use std::cmp;
use std::error::Error;
use std::sync::mpsc::{Sender, Receiver};

use types::*;
use util::StringResult;
use util::now;
use libcommon_proto::{game, control};
use libcommon_proto::ExtraArg;
use libcommon_proto::types::{LocalPos, LocalOffset, LocalTime};
use libphysics::TILE_SIZE;

use input::InputBits;
use tasks::{Input, Output};
use world::{self, Activity};

use self::clients::Clients;


mod clients;


pub struct Messages {
    send: Sender<Output>,
    recv: Receiver<Input>,
    clients: Clients,
    time_base: Time,
}

pub enum Event {
    Control(ControlEvent),
    Wire(WireId, WireEvent),
    Client(ClientId, ClientEvent),
}


pub enum ControlEvent {
    OpenWire(WireId, u32, String),
    CloseWire(WireId, Option<ClientId>),
    ReplCommand(u16, String),
    Shutdown,
    Restart(bool, bool),
}

pub enum WireEvent {
    Ready,
    BadRequest,
}

pub enum ClientEvent {
    Input(Time, InputBits),
    CloseDialog,
    MoveItem(InventoryId, SlotId, InventoryId, SlotId, u8),
    CraftRecipe(StructureId, InventoryId, RecipeId, u16),
    Chat(String),

    Interact(Time, Option<ExtraArg>),
    UseItem(Time, ItemId, Option<ExtraArg>),
    UseAbility(Time, ItemId, Option<ExtraArg>),

    CreateCharacter(u32),

    PathStart(LocalPos, u16),
    PathUpdate(LocalTime, V3, InputBits),
    PathBlocked(LocalTime),

    BadRequest,
}


#[derive(Debug, Clone)]
pub enum ControlResponse {
    WireClosed(WireId),
    ReplResult(u16, String),
}

#[derive(Debug, Clone)]
pub enum WireResponse {
    KickReason(String),
}

#[derive(Debug, Clone)]
pub enum SyncKind {
    Loading,
    Ok,
    Reset,
    Refresh,
}

#[derive(Debug, Clone)]
pub enum ClientResponse {
    Init(EntityId, Time, u32, u32),
    InitNoPawn(V3, Time, u32, u32),

    TerrainChunk(V2, Vec<u16>),
    UnloadChunk(V2),

    EntityAppear(EntityId, u32, String),
    EntityMotionStart(EntityId, V3, Time, V3, AnimId),
    EntityMotionStartEnd(EntityId, V3, Time, V3, AnimId, Time),
    EntityMotionEnd(EntityId, Time),
    EntityGone(EntityId, Time),
    EntityActivityIcon(EntityId, AnimId),
    ActivityChange(Activity),
    ResetMotion,

    StructureAppear(StructureId, TemplateId, V3),
    StructureGone(StructureId),
    StructureReplace(StructureId, TemplateId),

    InventoryAppear(InventoryId, Vec<world::Item>),
    InventoryUpdate(InventoryId, u8, world::Item),
    InventoryGone(InventoryId),

    PlaneFlags(u32),
    SyncStatus(SyncKind),

    GetInteractArgs(u32, ExtraArg),
    GetUseItemArgs(ItemId, u32, ExtraArg),
    GetUseAbilityArgs(ItemId, u32, ExtraArg),

    OpenDialog(Dialog),
    CancelDialog,
    MainInventory(InventoryId),
    AbilityInventory(InventoryId),
    EnergyUpdate(i32, i32, (i16, u16), Time),
    ProcessedInputs(Time, u16),
    ChatUpdate(String),
    KickReason(String),
}

#[derive(Debug, Clone)]
pub enum Dialog {
    Inventory(InventoryId),
    Container(InventoryId, InventoryId),
    Crafting(TemplateId, StructureId, InventoryId),
    PonyEdit(String),
}


pub type MessageEvent = Input;


impl Messages {
    pub fn new(recv: Receiver<Input>,
               send: Sender<Output>) -> Messages {
        Messages {
            send: send,
            recv: recv,
            clients: Clients::new(),
            time_base: 0,
        }
    }


    // Time adjustment

    // Regarding timestamps: All Time values within this module, as well as all Times passed
    // to/from the Engine or transmitted to/from clients, are "world times" (that is, adjusted
    // using `time_base`).

    fn world_time(&self, unix_time: Time) -> Time {
        unix_time - self.time_base
    }

    fn world_now(&self) -> Time {
        self.world_time(now())
    }

    // NB: This is designed to be called only once, near the beginning of server startup.  Calling
    // it while the server is running may have strange effects.
    pub fn set_world_time(&mut self, unix_time: Time, world_time: Time) {
        self.time_base = unix_time - world_time;
        debug!("new time_base: {:x} (world_time {:x})", self.time_base, world_time);
    }

    fn from_world_time(&self, world_time: Time) -> Time {
        world_time + self.time_base
    }


    // Client lifecycle

    pub fn add_client(&mut self, cid: ClientId, wire_id: WireId, name: &str) {
        self.clients.add(cid, wire_id, name);
    }

    pub fn remove_client(&mut self, cid: ClientId) {
        self.clients.remove(cid);
    }

    pub fn wire_to_client(&self, wire_id: WireId) -> Option<ClientId> {
        self.clients.wire_to_client(wire_id)
    }

    pub fn client_to_wire(&self, cid: ClientId) -> Option<WireId> {
        self.clients.get(cid).map(|c| c.wire_id())
    }

    pub fn name_to_client(&self, name: &str) -> Option<ClientId> {
        self.clients.name_to_client(name)
    }

    pub fn clients_len(&self) -> usize {
        self.clients.len()
    }


    // Event processing

    pub fn receiver(&self) -> &Receiver<MessageEvent> {
        &self.recv
    }

    pub fn process(&mut self, evt: MessageEvent) -> Option<(Event, Time)> {
        let now = self.world_now();
        self.handle_input(now, evt)
            .map(|evt| (evt, now))
    }

    fn handle_input(&mut self, now: Time, input: Input) -> Option<Event> {
        match input {
            Input::Control(req) => self.handle_control_req(now, req),
            Input::Game(wire_id, req) => {
                if let Some(cid) = self.clients.wire_to_client(wire_id) {
                    self.handle_client_req(now, wire_id, cid, req)
                } else {
                    self.handle_pre_login_req(now, wire_id, req)
                }
            },
        }
    }

    fn handle_control_req(&mut self, _now: Time, req: control::Request) -> Option<Event> {
        use libcommon_proto::control::Request;
        match req {
            Request::AddClient(raw_id, uid, name) =>
                // Let the caller decide when to actually add the client.
                Some(Event::Control(ControlEvent::OpenWire(WireId(raw_id), uid, name))),
            Request::RemoveClient(raw_id) => {
                let wire_id = WireId(raw_id);
                // Let the caller decide when to actually remove the client.
                let opt_cid = self.clients.wire_to_client(wire_id);
                Some(Event::Control(ControlEvent::CloseWire(wire_id, opt_cid)))
            },
            Request::ReplCommand(cookie, cmd) =>
                Some(Event::Control(ControlEvent::ReplCommand(cookie, cmd))),
            Request::Shutdown(()) =>
                Some(Event::Control(ControlEvent::Shutdown)),
            Request::RestartServer(()) =>
                Some(Event::Control(ControlEvent::Restart(true, false))),
            Request::RestartClient(()) =>
                Some(Event::Control(ControlEvent::Restart(false, true))),
            Request::RestartBoth(()) =>
                Some(Event::Control(ControlEvent::Restart(true, true))),
        }
    }

    fn handle_pre_login_req(&mut self,
                            now: Time,
                            wire_id: WireId,
                            req: game::Request) -> Option<Event> {
        use libcommon_proto::game::Request;
        match req {
            Request::Ping(cookie) => {
                self.send_raw(wire_id, game::Response::Pong(
                        cookie, LocalTime::from_global_64(now)));
                None
            },
            Request::Ready(()) =>
                Some(Event::Wire(wire_id, WireEvent::Ready)),
            _ => {
                warn!("bad pre-login request from {:?}: {:?}", wire_id, req);
                Some(Event::Wire(wire_id, WireEvent::BadRequest))
            },
        }
    }

    fn handle_client_req(&mut self,
                         now: Time,
                         wire_id: WireId,
                         cid: ClientId,
                         req: game::Request) -> Option<Event> {
        match self.try_handle_client_req(now, wire_id, req) {
            Ok(evt) => evt.map(|e| Event::Client(cid, e)),
            Err(e) => {
                warn!("bad request from {:?}: {}", cid, e.description());
                Some(Event::Client(cid, ClientEvent::BadRequest))
            },
        }
    }

    fn try_handle_client_req(&mut self,
                             now: Time,
                             wire_id: WireId,
                             req: game::Request) -> StringResult<Option<ClientEvent>> {
        use libcommon_proto::game::Request;

        let opt_client = self.clients.wire_to_client(wire_id)
                             .and_then(|cid| self.clients.get(cid));

        match req {
            Request::Ping(cookie) => {
                self.send_raw(wire_id, game::Response::Pong(
                        cookie, LocalTime::from_global_64(now)));
                Ok(None)
            },

            Request::Input(time, input) => {
                let time = cmp::max(time.to_global_64(now), now);
                let input = unwrap!(InputBits::from_bits(input));
                Ok(Some(ClientEvent::Input(time, input)))
            },

            Request::CloseDialog(()) =>
                Ok(Some(ClientEvent::CloseDialog)),

            Request::MoveItem(from_iid, from_slot, to_iid, to_slot, count) =>
                Ok(Some(ClientEvent::MoveItem(from_iid, from_slot, to_iid, to_slot, count))),

            Request::CraftRecipe(sid, iid, recipe_id, count) =>
                Ok(Some(ClientEvent::CraftRecipe(sid, iid, recipe_id, count))),

            Request::Chat(msg) =>
                Ok(Some(ClientEvent::Chat(msg))),


            Request::Interact(time) => {
                let time = cmp::max(time.to_global_64(now), now);
                Ok(Some(ClientEvent::Interact(time, None)))
            },

            Request::UseItem(time, item_id) => {
                let time = cmp::max(time.to_global_64(now), now);
                Ok(Some(ClientEvent::UseItem(time, item_id, None)))
            },

            Request::UseAbility(time, item_id) => {
                let time = cmp::max(time.to_global_64(now), now);
                Ok(Some(ClientEvent::UseAbility(time, item_id, None)))
            },


            Request::InteractWithArgs(time, args) => {
                let time = cmp::max(time.to_global_64(now), now);
                Ok(Some(ClientEvent::Interact(time, Some(args))))
            },

            Request::UseItemWithArgs(time, item_id, args) => {
                let time = cmp::max(time.to_global_64(now), now);
                Ok(Some(ClientEvent::UseItem(time, item_id, Some(args))))
            },

            Request::UseAbilityWithArgs(time, item_id, args) => {
                let time = cmp::max(time.to_global_64(now), now);
                Ok(Some(ClientEvent::UseAbility(time, item_id, Some(args))))
            },


            Request::CreateCharacter(appearance) =>
                Ok(Some(ClientEvent::CreateCharacter(appearance))),


            Request::PathStart(pos, delay) => {
                let pos = unwrap!(opt_client).unoffset_pos(pos);
                Ok(Some(ClientEvent::PathStart(pos, delay)))
            },

            Request::PathUpdate(rel_time, velocity, input) => {
                let input = unwrap!(InputBits::from_bits(input));
                Ok(Some(ClientEvent::PathUpdate(rel_time, velocity.to_global(), input)))
            },

            Request::PathBlocked(rel_time) =>
                Ok(Some(ClientEvent::PathBlocked(rel_time))),


            _ => fail!("bad request: {:?}", req),
        }
    }


    // Response sending

    fn send_raw(&self, wire_id: WireId, msg: game::Response) {
        trace!("{:?}: {:?}", wire_id, msg);
        self.send.send(Output::Game(wire_id, msg)).unwrap();
    }

    fn send_raw_control(&self, msg: control::Response) {
        trace!("[control]: {:?}", msg);
        self.send.send(Output::Control(msg)).unwrap();
    }

    pub fn send_control(&self, resp: ControlResponse) {
        match resp {
            ControlResponse::WireClosed(wire_id) =>
                self.send_raw_control(control::Response::ClientRemoved(wire_id.unwrap())),
            ControlResponse::ReplResult(cookie, msg) =>
                self.send_raw_control(control::Response::ReplResult(cookie, msg)),
        }
    }

    pub fn send_wire(&self, wire_id: WireId, resp: WireResponse) {
        match resp {
            WireResponse::KickReason(msg) =>
                self.send_raw(wire_id, game::Response::KickReason(msg)),
        }
    }

    pub fn send_client(&self, cid: ClientId, resp: ClientResponse) {
        use libcommon_proto::game::Response;

        let client = match self.clients.get(cid) {
            Some(x) => x,
            None => {
                debug!("can't send to client {:?} (no wire): {:?}", cid, resp);
                return;
            },
        };
        let wire_id = client.wire_id();

        match resp {
            ClientResponse::Init(eid, time, cycle_base, cycle_ms) => {
                self.send_raw(wire_id, Response::Init(eid,
                                                      LocalTime::from_global_64(time),
                                                      cycle_base,
                                                      cycle_ms));
            },

            ClientResponse::InitNoPawn(pos, time, cycle_base, cycle_ms) => {
                self.send_raw(wire_id, Response::InitNoPawn(client.local_pos(pos),
                                                            LocalTime::from_global_64(time),
                                                            cycle_base,
                                                            cycle_ms));
            },

            ClientResponse::TerrainChunk(cpos, data) => {
                let index = client.local_chunk_index(cpos);
                self.send_raw(wire_id, Response::TerrainChunk(index, data));
            },

            ClientResponse::UnloadChunk(cpos) => {
                let index = client.local_chunk_index(cpos);
                self.send_raw(wire_id, Response::UnloadChunk(index));
            },


            ClientResponse::EntityAppear(eid, appear, name) =>
                self.send_raw(wire_id, Response::EntityAppear(eid, appear, name)),

            ClientResponse::EntityMotionStart(eid, pos, time, velocity, anim) => {
                let pos16 = client.local_pos(pos);
                let v16 = LocalOffset::from_global(velocity);
                self.send_raw(wire_id, Response::EntityMotionStart(
                        eid, pos16, LocalTime::from_global_64(time), v16, anim));
            },

            ClientResponse::EntityMotionStartEnd(eid, pos, time, velocity, anim, end_time) => {
                let pos16 = client.local_pos(pos);
                let v16 = LocalOffset::from_global(velocity);
                self.send_raw(wire_id, Response::EntityMotionStartEnd(
                        eid, pos16, LocalTime::from_global_64(time), v16, anim,
                        LocalTime::from_global_64(end_time)));
            },

            ClientResponse::EntityMotionEnd(eid, time) =>
                self.send_raw(wire_id, Response::EntityMotionEnd(
                        eid, LocalTime::from_global_64(time))),

            ClientResponse::EntityGone(eid, time) => {
                self.send_raw(wire_id, Response::EntityGone(eid, LocalTime::from_global_64(time)));
            },

            ClientResponse::EntityActivityIcon(eid, anim_id) => {
                self.send_raw(wire_id, Response::EntityActivityIcon(eid, anim_id));
            },

            ClientResponse::ActivityChange(activity) => {
                let code = match activity {
                    Activity::Walk => 0,
                    // fly => 1
                    Activity::Emote(_) => 2,    // interruptible
                    Activity::Work(_, _) |
                    Activity::Teleport => 3,   // uninterruptible
                };
                self.send_raw(wire_id, Response::ActivityChange(code));
            },

            ClientResponse::ResetMotion =>
                self.send_raw(wire_id, Response::ResetMotion(())),


            ClientResponse::StructureAppear(sid, template_id, pos) => {
                let local_pos = client.local_pos(pos * scalar(TILE_SIZE));
                self.send_raw(wire_id, Response::StructureAppear(sid, template_id, local_pos));
            },

            ClientResponse::StructureGone(sid) => {
                self.send_raw(wire_id, Response::StructureGone(sid));
            },

            ClientResponse::StructureReplace(sid, template_id) => {
                self.send_raw(wire_id, Response::StructureReplace(sid, template_id));
            },


            ClientResponse::InventoryAppear(iid, ref all_items) => {
                let all_slot_data = all_items.iter().map(|&x| encode_item(x)).collect();
                self.send_raw(wire_id, Response::InventoryAppear(iid, all_slot_data));
            },

            ClientResponse::InventoryGone(iid) => {
                self.send_raw(wire_id, Response::InventoryGone(iid));
            },

            ClientResponse::InventoryUpdate(iid, slot_idx, item) => {
                let slot_data = encode_item(item);
                self.send_raw(wire_id, Response::InventoryUpdate(iid, slot_idx, slot_data));
            },


            ClientResponse::PlaneFlags(flags) =>
                self.send_raw(wire_id, Response::PlaneFlags(flags)),

            ClientResponse::SyncStatus(kind) => {
                let arg = match kind {
                    SyncKind::Loading => 0,
                    SyncKind::Ok => 1,
                    SyncKind::Reset => 2,
                    SyncKind::Refresh => 3,
                };
                self.send_raw(wire_id, Response::SyncStatus(arg))
            },


            ClientResponse::GetInteractArgs(dialog_id, parts) =>
                self.send_raw(wire_id, Response::GetInteractArgs(dialog_id, parts)),

            ClientResponse::GetUseItemArgs(item_id, dialog_id, parts) =>
                self.send_raw(wire_id, Response::GetUseItemArgs(item_id, dialog_id, parts)),

            ClientResponse::GetUseAbilityArgs(item_id, dialog_id, parts) =>
                self.send_raw(wire_id, Response::GetUseAbilityArgs(item_id, dialog_id, parts)),


            ClientResponse::OpenDialog(dialog) => {
                match dialog {
                    Dialog::Inventory(iid) => 
                        self.send_raw(wire_id, Response::OpenDialog(0, vec![iid.unwrap()])),
                    Dialog::Container(iid1, iid2) => 
                        self.send_raw(wire_id, Response::OpenDialog(1, vec![iid1.unwrap(),
                                                                            iid2.unwrap()])),
                    Dialog::Crafting(template_id, sid, iid) =>
                        self.send_raw(wire_id, Response::OpenCrafting(template_id, sid, iid)),

                    Dialog::PonyEdit(name) =>
                        self.send_raw(wire_id, Response::OpenPonyEdit(name)),
                }
            },

            ClientResponse::CancelDialog =>
                self.send_raw(wire_id, Response::CancelDialog(())),

            ClientResponse::MainInventory(iid) =>
                self.send_raw(wire_id, Response::MainInventory(iid)),

            ClientResponse::AbilityInventory(iid) =>
                self.send_raw(wire_id, Response::AbilityInventory(iid)),

            ClientResponse::EnergyUpdate(cur, max, rate, time) =>
                self.send_raw(wire_id, Response::EnergyUpdate(
                        cur as u16, max as u16, rate, LocalTime::from_global_64(time))),

            ClientResponse::ProcessedInputs(time, count) =>
                self.send_raw(wire_id, Response::ProcessedInputs(
                        LocalTime::from_global_64(time), count)),

            ClientResponse::ChatUpdate(msg) =>
                self.send_raw(wire_id, Response::ChatUpdate(msg)),

            ClientResponse::KickReason(msg) =>
                self.send_raw(wire_id, Response::KickReason(msg)),
        }
    }

    pub fn broadcast_clients(&self, resp: ClientResponse) {
        for (&cid, _) in self.clients.iter() {
            self.send_client(cid, resp.clone());
        }
    }
}

fn encode_item(i: world::Item) -> (u8, u8, ItemId) {
    // TODO: change protocol encoding of items
    const TAG_EMPTY: u8 = 0;
    const TAG_BULK: u8 = 1;

    if i.is_none() {
        (TAG_EMPTY, 0, 0)
    } else {
        (TAG_BULK, i.count, i.id)
    }
}

use std::boxed::FnBox;
use std::sync::mpsc::{Sender, Receiver};

use types::*;

use cache::TerrainCache;
use chat::Chat;
use chunks::Chunks;
use components::energy::Energy;
use data::Data;
use dialogs::Dialogs;
use input::{Input, Action};
use logic;
use logic::extra::Extra;
use messages::{Messages, MessageEvent};
use messages::{Event, ControlEvent, WireEvent, ClientEvent};
use messages::SyncKind;
use messages::{ControlResponse, WireResponse, ClientResponse};
use msg::{Request, Response};
use physics::Physics;
use script::ScriptHooks;
use storage::Storage;
use terrain_gen::{TerrainGen, TerrainGenEvent};
use timer::{Timer, TimerEvent};
use timing::*;
use vision::Vision;
use world::World;

use self::split::EngineRef;
use self::split2::Coded;


#[macro_use] pub mod split;
#[macro_use] pub mod split2;
pub mod glue;


pub struct Engine<'d> {
    pub data: &'d Data,
    pub storage: &'d Storage,
    pub script_hooks: &'d ScriptHooks,
    pub now: Time,
    last_tick: Time,

    pub world: World<'d>,

    pub extra: Extra,
    pub messages: Messages,
    pub timer: Timer,
    pub physics: Physics<'d>,
    pub vision: Vision,
    pub chunks: Chunks<'d>,
    pub cache: TerrainCache,
    pub terrain_gen: TerrainGen,
    pub chat: Chat,
    pub dialogs: Dialogs,
    pub input: Input,

    pub energy: Energy,
}

#[must_use]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum HandlerResult {
    Continue,
    Shutdown,
    Restart,
}

impl<'d> Engine<'d> {
    pub fn new(data: &'d Data,
               storage: &'d Storage,
               script_hooks: &'d ScriptHooks,
               receiver: Receiver<(WireId, Request)>,
               sender: Sender<(WireId, Response)>) -> Engine<'d> {
        Engine {
            data: data,
            storage: storage,
            script_hooks: script_hooks,
            now: TIME_MIN,
            last_tick: TIME_MIN,

            world: World::new(data),

            extra: Extra::new(),
            messages: Messages::new(receiver, sender),
            timer: Timer::new(),
            physics: Physics::new(data),
            vision: Vision::new(),
            chunks: Chunks::new(storage),
            cache: TerrainCache::new(),
            terrain_gen: TerrainGen::new(),
            chat: Chat::new(),
            dialogs: Dialogs::new(),
            input: Input::new(),

            energy: Energy::new(),
        }
    }

    pub fn run(&mut self) {
        use self::HandlerResult::*;
        logic::lifecycle::start_up(self);
        if let Some(file) = self.storage.open_restart_file() {
            logic::lifecycle::post_restart(self, file);
            self.storage.remove_restart_file();
        }

        self.timer.schedule(next_tick(self.now), |eng| eng.unwrap().tick());

        loop {
            enum Event {
                FromTimer(TimerEvent),
                FromMessage(MessageEvent),
                FromTerrainGen(TerrainGenEvent),
            }

            let evt = {
                let recv_timer = self.timer.receiver();
                let recv_message = self.messages.receiver();
                let recv_terrain_gen = self.terrain_gen.receiver();
                select! {
                    evt = recv_timer.recv() => Event::FromTimer(evt.unwrap()),
                    evt = recv_message.recv() => Event::FromMessage(evt.unwrap()),
                    evt = recv_terrain_gen.recv() => Event::FromTerrainGen(evt.unwrap())
                }
            };

            match evt {
                Event::FromTimer(evt) => {
                    let (cb, now) = unwrap_or!(self.timer.process(evt), continue);
                    self.now = now;
                    cb.call_box((self.as_ref(),));
                },
                Event::FromMessage(evt) => {
                    let (evt, now) = unwrap_or!(self.messages.process(evt), continue);
                    match self.handle(now, evt) {
                        Continue => {},
                        Shutdown => break,
                        Restart => {
                            logic::lifecycle::pre_restart(self);
                            break;
                        },
                    }
                },
                Event::FromTerrainGen(evt) => {
                    logic::terrain_gen::process(self, evt);
                },
            }
        }

        logic::lifecycle::shut_down(self);
    }


    fn handle(&mut self,
              now: Time,
              evt: Event) -> HandlerResult {
        use messages::Event::*;
        self.now = now;
        match evt {
            Control(e) => self.handle_control(e),
            Wire(wire_id, e) => self.handle_wire(wire_id, e),
            Client(cid, e) => self.handle_client(cid, e),
        }
    }

    fn handle_control(&mut self,
                      evt: ControlEvent) -> HandlerResult {
        use messages::ControlEvent::*;
        use messages::ControlResponse::*;
        match evt {
            OpenWire(wire_id, uid, name) => {
                info!("OpenWire: {:?}, {:x}, {}", wire_id, uid, name);
                self.extra.wire_info.insert(wire_id, (uid, name));
            },

            CloseWire(wire_id, opt_cid) => {
                if let Some(cid) = opt_cid {
                    self.cleanup_client(cid);
                }
                self.messages.send_control(WireClosed(wire_id));
                self.extra.wire_info.remove(&wire_id);
            },

            ReplCommand(cookie, msg) => {
                let result = self.script_hooks.call_eval(self, &msg);
                let result_str = match result {
                    Ok(s) => s + "\n",
                    Err(e) => format!("[exception: {}]\n", e),
                };
                self.messages.send_control(ReplResult(cookie, result_str));
            },

            Shutdown => {
                return HandlerResult::Shutdown;
            },

            Restart(server, client) => {
                if client {
                    self.messages.broadcast_clients(ClientResponse::SyncStatus(SyncKind::Refresh));
                }
                if server {
                    return HandlerResult::Restart;
                }
            },
        }
        HandlerResult::Continue
    }

    fn handle_wire(&mut self,
                   wire_id: WireId,
                   evt: WireEvent) -> HandlerResult {
        use messages::WireEvent::*;
        match evt {
            Ready => {
                info!("wire ready: {:?}", wire_id);
                warn_on_err!(logic::client::ready(self, wire_id));
            },

            BadRequest => {
                self.kick_wire(wire_id, "bad request");
            },
        }
        HandlerResult::Continue
    }

    fn handle_client(&mut self,
                     cid: ClientId,
                     evt: ClientEvent) -> HandlerResult {
        use messages::ClientEvent::*;
        match evt {
            Input(time, input) => {
                let tick = (time - next_tick(self.last_tick)) / TICK_MS;
                self.input.schedule_input(cid, tick as i32, input);
            },

            CloseDialog => {
                logic::dialogs::close_dialog(self.refine(), cid);
            },

            MoveItem(from_iid, from_slot, to_iid, to_slot, count) => {
                warn_on_err!(logic::items::move_items2(self,
                                                       from_iid,
                                                       from_slot,
                                                       to_iid,
                                                       to_slot,
                                                       count));
            },

            CraftRecipe(station_sid, iid, recipe_id, count) => {
                warn_on_err!(logic::items::craft_recipe(self,
                                                        station_sid, iid, recipe_id, count));
            },

            Chat(msg) => {
                logic::input::chat(self, cid, msg);
            },

            Interact(_time, args) => {
                self.input.schedule_action(cid, Action::Interact, args);
            },

            UseItem(_time, item_id, args) => {
                self.input.schedule_action(cid, Action::UseItem(item_id), args);
            },

            UseAbility(_time, item_id, args) => {
                self.input.schedule_action(cid, Action::UseAbility(item_id), args);
            },

            CreateCharacter(appearance) => {
                warn_on_err!(logic::client::create_character(self, cid, appearance));
            },

            BadRequest => {
                self.kick_client(cid, "bad request");
            },
        }
        HandlerResult::Continue
    }


    pub fn tick(&mut self) {
        self.last_tick = self.now;
        logic::tick::tick(self);
    }


    fn cleanup_client(&mut self, cid: ClientId) {
        warn_on_err!(logic::client::logout(self, cid));
    }

    fn cleanup_wire(&mut self, wire_id: WireId) {
        if let Some(cid) = self.messages.wire_to_client(wire_id) {
            self.cleanup_client(cid);
        }
    }

    pub fn kick_client<'a, S: Into<String>>(&mut self, cid: ClientId, msg: S) {
        let wire_id = self.messages.client_to_wire(cid)
                .expect("missing WireId for existing client");

        self.messages.send_client(cid, ClientResponse::KickReason(msg.into()));
        self.cleanup_client(cid);
        self.messages.send_control(ControlResponse::WireClosed(wire_id));
        // If there's a Client object, the wire_info should already have been removed.
    }

    pub fn kick_wire<'a, S: Into<String>>(&mut self, wire_id: WireId, msg: S) {
        self.messages.send_wire(wire_id, WireResponse::KickReason(msg.into()));
        self.cleanup_wire(wire_id);
        self.messages.send_control(ControlResponse::WireClosed(wire_id));
        self.extra.wire_info.remove(&wire_id);
    }


    pub fn as_ref<'b>(&'b mut self) -> EngineRef<'b, 'd> {
        EngineRef::new(self)
    }

    pub fn now(&self) -> Time {
        self.now
    }
}

fn name_valid(name: &str) -> Result<(), &'static str> {
    if name.len() == 0 {
        return Err("Please enter a name.");
    }

    if name.len() > 16 {
        return Err("Name is too long (must not exceed 16 characters).");
    }

    let chars_ok = name.chars().all(|c| {
        (c >= 'a' && c <= 'z') ||
        (c >= 'A' && c <= 'Z') ||
        (c >= '0' && c <= '9') ||
        c == ' ' ||
        c == '-'
    });
    if !chars_ok {
        return Err("Names may only contain letters, numbers, spaces, and hyphens.");
    }

    let has_alnum = name.chars().any(|c| {
        (c >= 'a' && c <= 'z') ||
        (c >= 'A' && c <= 'Z') ||
        (c >= '0' && c <= '9')
    });
    if !has_alnum {
        return Err("Names must contain at least one letter or digit.");
    }
    if name.contains("  ") {
        return Err("Names must not have more than one space in a row.");
    }
    if name.starts_with(" ") || name.ends_with(" ") {
        return Err("Names must not start or end with a space.");
    }

    Ok(())
}

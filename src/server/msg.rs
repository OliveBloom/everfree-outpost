use std::collections::HashMap;
use std::io::{self, Read, Write};

use wire::{self, WireReader, WireWriter};
use types::*;

pub use self::Request::*;
pub use self::Response::*;
use self::op::Opcode;


mod op {
    use wire::{self, WireWriter};
    use std::io::{self, Write};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Opcode(pub u16);

    impl Opcode {
        pub fn unwrap(self) -> u16 {
            let Opcode(v) = self;
            v
        }
    }

    impl wire::WriteTo for Opcode {
        fn write_to<W: Write>(&self, w: &mut WireWriter<W>) -> io::Result<()> {
            self.unwrap().write_to(w)
        }

        fn size(&self) -> usize { self.unwrap().size() }

        fn size_is_fixed() -> bool { true }
    }


    macro_rules! opcodes {
        ($($name:ident = $value:expr,)*) => {
            $(
                #[allow(non_upper_case_globals, dead_code)]
                pub const $name: Opcode = Opcode($value);
            )*
        }
    }

    opcodes! {
        // Requests
        Ping = 0x0003,
        Input = 0x0004,
        CraftRecipe = 0x0009,
        Chat = 0x000a,
        Interact = 0x000c,
        UseItem = 0x000d,
        UseAbility = 0x000e,
        InteractWithArgs = 0x0010,
        UseItemWithArgs = 0x0011,
        UseAbilityWithArgs = 0x0012,
        MoveItem = 0x0013,
        AuthResponse = 0x0014,      // For auth verifier only
        CreateCharacter = 0x0015,
        Ready = 0x0016,
        CloseDialog = 0x0017,

        // Deprecated requests
        GetTerrain = 0x0001,
        UpdateMotion = 0x0002,
        Login = 0x0005,
        Action = 0x0006,
        UnsubscribeInventory = 0x0007,
        old_MoveItem = 0x0008,
        Register = 0x000b,
        OpenInventory = 0x000f,

        // Responses
        TerrainChunk = 0x8001,
        Pong = 0x8003,
        Init = 0x8005,
        KickReason = 0x8006,
        UnloadChunk = 0x8007,
        OpenDialog = 0x8008,
        OpenCrafting = 0x800a,
        ChatUpdate = 0x800b,
        EntityAppear = 0x800c,
        EntityGone = 0x800d,
        StructureAppear = 0x800f,
        StructureGone = 0x8010,
        MainInventory = 0x8011,
        AbilityInventory = 0x8012,
        PlaneFlags = 0x8013,
        GetInteractArgs = 0x8014,
        GetUseItemArgs = 0x8015,
        GetUseAbilityArgs = 0x8016,
        SyncStatus = 0x8017,
        StructureReplace = 0x8018,
        InventoryUpdate = 0x8019,
        InventoryAppear = 0x801a,
        InventoryGone = 0x801b,
        EntityMotionStart = 0x801c,
        EntityMotionEnd = 0x801d,
        EntityMotionStartEnd = 0x801e,
        ProcessedInputs = 0x801f,
        ActivityChange = 0x8020,
        AuthChallenge = 0x8021,     // For auth verifier only
        AuthResult = 0x8022,        // For auth verifier only
        InitNoPawn = 0x8023,
        OpenPonyEdit = 0x8024,
        EntityActivityIcon = 0x8025,
        CancelDialog = 0x8026,

        // Deprecated responses
        PlayerMotion = 0x8002,
        old_EntityUpdate = 0x8004,
        old_InventoryUpdate = 0x8009,
        RegisterResult = 0x800e,

        // Control messages
        AddClient = 0xff00,
        RemoveClient = 0xff01,
        ClientRemoved = 0xff02,
        ReplCommand = 0xff03,
        ReplResult = 0xff04,
        Shutdown = 0xff05,
        RestartServer = 0xff06,
        RestartClient = 0xff07,
        RestartBoth = 0xff08,
        AuthStart = 0xff09,         // For auth verifier only
        AuthCancel = 0xff0a,        // For auth verifier only
        AuthFinish = 0xff0b,        // For auth verifier only
    }
}


#[allow(dead_code)]
#[derive(Debug)]
pub enum Request {
    // Ordinary requests
    Ping(u16),
    Input(LocalTime, u16),
    CraftRecipe(StructureId, InventoryId, RecipeId, u16),
    Chat(String),
    Interact(LocalTime),
    UseItem(LocalTime, ItemId),
    UseAbility(LocalTime, ItemId),
    InteractWithArgs(LocalTime, ExtraArg),
    UseItemWithArgs(LocalTime, ItemId, ExtraArg),
    UseAbilityWithArgs(LocalTime, ItemId, ExtraArg),
    MoveItem(InventoryId, SlotId, InventoryId, SlotId, u8),
    CreateCharacter(u32),
    Ready,
    CloseDialog,

    // Control messages
    AddClient(WireId, u32, String),
    RemoveClient(WireId),
    ReplCommand(u16, String),
    Shutdown,
    Restart(bool, bool),

    // Server-internal messages
    BadMessage(Opcode),
}

impl Request {
    pub fn read_from<R: Read>(wr: &mut WireReader<R>) -> io::Result<(WireId, Request)> {
        let id = try!(wr.read_header());
        let opcode = Opcode(try!(wr.read()));

        let req = match opcode {
            op::Ping => Ping(try!(wr.read())),
            op::Input => {
                let (a, b): (LocalTime, u16) = try!(wr.read());
                Input(a, b)
            },
            op::CraftRecipe => {
                let (a, b, c, d) = try!(wr.read());
                CraftRecipe(a, b, c, d)
            },
            op::Chat => {
                let a = try!(wr.read());
                Chat(a)
            },
            op::Interact => {
                let a = try!(wr.read());
                Interact(a)
            },
            op::UseItem => {
                let (a, b) = try!(wr.read());
                UseItem(a, b)
            },
            op::UseAbility => {
                let (a, b) = try!(wr.read());
                UseAbility(a, b)
            },
            op::InteractWithArgs => {
                let (a, b) = try!(wr.read());
                InteractWithArgs(a, b)
            },
            op::UseItemWithArgs => {
                let (a, b, c) = try!(wr.read());
                UseItemWithArgs(a, b, c)
            },
            op::UseAbilityWithArgs => {
                let (a, b, c) = try!(wr.read());
                UseAbilityWithArgs(a, b, c)
            },
            op::MoveItem => {
                let (a, b, c, d, e) = try!(wr.read());
                MoveItem(a, b, c, d, e)
            },
            op::CreateCharacter => {
                let a = try!(wr.read());
                CreateCharacter(a)
            },
            op::Ready => {
                Ready
            },
            op::CloseDialog => {
                CloseDialog
            },

            op::AddClient => {
                let (a, b, c) = try!(wr.read());
                AddClient(a, b, c)
            },
            op::RemoveClient => {
                let a = try!(wr.read());
                RemoveClient(a)
            },
            op::ReplCommand => {
                let (a, b) = try!(wr.read());
                ReplCommand(a, b)
            },
            op::Shutdown => {
                Shutdown
            },
            op::RestartServer => {
                Restart(true, false)
            },
            op::RestartClient => {
                Restart(false, true)
            },
            op::RestartBoth => {
                Restart(true, true)
            },
            _ => BadMessage(opcode),
        };

        if !wr.done() {
            Ok((id, BadMessage(opcode)))
        } else {
            Ok((id, req))
        }
    }
}


#[allow(dead_code)]
#[derive(Debug)]
pub enum Response {
    TerrainChunk(u16, Vec<u16>),
    Pong(u16, LocalTime),
    Init(EntityId, LocalTime, u32, u32),
    KickReason(String),
    UnloadChunk(u16),
    OpenDialog(u32, Vec<u32>),
    OpenCrafting(TemplateId, StructureId, InventoryId),
    ChatUpdate(String),
    EntityAppear(EntityId, u32, String),
    EntityGone(EntityId, LocalTime),
    RegisterResult(u32, String),
    StructureAppear(StructureId, TemplateId, (u16, u16, u16)),
    StructureGone(StructureId),
    MainInventory(InventoryId),
    AbilityInventory(InventoryId),
    PlaneFlags(u32),
    GetInteractArgs(u32, ExtraArg),
    GetUseItemArgs(ItemId, u32, ExtraArg),
    GetUseAbilityArgs(ItemId, u32, ExtraArg),
    SyncStatus(u8),
    StructureReplace(StructureId, TemplateId),
    InventoryUpdate(InventoryId, u8, (u8, u8, ItemId)),
    InventoryAppear(InventoryId, Vec<(u8, u8, ItemId)>),
    InventoryGone(InventoryId),
    EntityMotionStart(EntityId, (u16, u16, u16), LocalTime, (i16, i16, i16), AnimId),
    EntityMotionEnd(EntityId, LocalTime),
    EntityMotionStartEnd(EntityId, (u16, u16, u16), LocalTime, (i16, i16, i16), AnimId, LocalTime),
    ProcessedInputs(LocalTime, u16),
    ActivityChange(u8),
    InitNoPawn((u16, u16, u16), LocalTime, u32, u32),
    OpenPonyEdit(String),
    EntityActivityIcon(EntityId, AnimId),
    CancelDialog,

    ClientRemoved(WireId),
    ReplResult(u16, String),
}

impl Response {
    pub fn write_to<W: Write>(&self, id: WireId, ww: &mut WireWriter<W>) -> io::Result<()> {
        try!(match *self {
            TerrainChunk(idx, ref data) =>
                ww.write_msg(id, (op::TerrainChunk, idx, data)),
            Pong(data, time) =>
                ww.write_msg(id, (op::Pong, data, time)),
            Init(pawn_id, now, cycle_base, cycle_ms) =>
                ww.write_msg(id, (op::Init, pawn_id, now, cycle_base, cycle_ms)),
            KickReason(ref msg) =>
                ww.write_msg(id, (op::KickReason, msg)),
            UnloadChunk(idx) =>
                ww.write_msg(id, (op::UnloadChunk, idx)),
            OpenDialog(dialog_id, ref params) =>
                ww.write_msg(id, (op::OpenDialog, dialog_id, params)),
            OpenCrafting(station_type, station_id, inventory_id) =>
                ww.write_msg(id, (op::OpenCrafting, station_type, station_id, inventory_id)),
            ChatUpdate(ref msg) =>
                ww.write_msg(id, (op::ChatUpdate, msg)),
            EntityAppear(entity_id, appearance, ref name) =>
                ww.write_msg(id, (op::EntityAppear, entity_id, appearance, name)),
            EntityGone(entity_id, time) =>
                ww.write_msg(id, (op::EntityGone, entity_id, time)),
            RegisterResult(code, ref msg) =>
                ww.write_msg(id, (op::RegisterResult, code, msg)),
            StructureAppear(sid, template_id, pos) =>
                ww.write_msg(id, (op::StructureAppear, sid, template_id, pos)),
            StructureGone(sid) =>
                ww.write_msg(id, (op::StructureGone, sid)),
            MainInventory(iid) =>
                ww.write_msg(id, (op::MainInventory, iid)),
            AbilityInventory(iid) =>
                ww.write_msg(id, (op::AbilityInventory, iid)),
            PlaneFlags(flags) =>
                ww.write_msg(id, (op::PlaneFlags, flags)),
            GetInteractArgs(dialog_id, ref args) =>
                ww.write_msg(id, (op::GetInteractArgs, dialog_id, args)),
            GetUseItemArgs(item_id, dialog_id, ref args) =>
                ww.write_msg(id, (op::GetUseItemArgs, item_id, dialog_id, args)),
            GetUseAbilityArgs(item_id, dialog_id, ref args) =>
                ww.write_msg(id, (op::GetUseAbilityArgs, item_id, dialog_id, args)),
            SyncStatus(kind) =>
                ww.write_msg(id, (op::SyncStatus, kind)),
            StructureReplace(sid, template_id) =>
                ww.write_msg(id, (op::StructureReplace, sid, template_id)),
            InventoryUpdate(inventory_id, slot_idx, slot_data) =>
                ww.write_msg(id, (op::InventoryUpdate, inventory_id, slot_idx, slot_data)),
            InventoryAppear(inventory_id, ref all_slot_data) =>
                ww.write_msg(id, (op::InventoryAppear, inventory_id, all_slot_data)),
            InventoryGone(inventory_id) =>
                ww.write_msg(id, (op::InventoryGone, inventory_id)),
            EntityMotionStart(entity_id, pos, time, velocity, anim_id) =>
                ww.write_msg(id, (op::EntityMotionStart, entity_id,
                                  pos, time, velocity, anim_id)),
            EntityMotionEnd(entity_id, time) =>
                ww.write_msg(id, (op::EntityMotionEnd, entity_id, time)),
            EntityMotionStartEnd(entity_id, pos, time, velocity, end_time, anim_id) =>
                ww.write_msg(id, (op::EntityMotionStartEnd, entity_id,
                                  pos, time, velocity, end_time, anim_id)),
            ProcessedInputs(time, count) =>
                ww.write_msg(id, (op::ProcessedInputs, time, count)),
            ActivityChange(activity) =>
                ww.write_msg(id, (op::ActivityChange, activity)),
            InitNoPawn(pos, now, cycle_base, cycle_ms) =>
                ww.write_msg(id, (op::InitNoPawn, pos, now, cycle_base, cycle_ms)),
            OpenPonyEdit(ref name) =>
                ww.write_msg(id, (op::OpenPonyEdit, name)),
            EntityActivityIcon(eid, anim) =>
                ww.write_msg(id, (op::EntityActivityIcon, eid, anim)),
            CancelDialog =>
                ww.write_msg(id, op::CancelDialog),

            ClientRemoved(wire_id) =>
                ww.write_msg(id, (op::ClientRemoved, wire_id)),
            ReplResult(cookie, ref msg) =>
                ww.write_msg(id, (op::ReplResult, cookie, msg)),
        });
        ww.flush()
    }
}


#[derive(Debug, Clone)]
pub struct Motion {
    pub start_pos: (u16, u16, u16),
    pub start_time: LocalTime,
    pub end_pos: (u16, u16, u16),
    pub end_time: LocalTime,
}

impl wire::ReadFrom for Motion {
    fn read_from<R: Read>(r: &mut WireReader<R>) -> io::Result<Motion> {
        let (a, b, c, d): ((u16, u16, u16), LocalTime, (u16, u16, u16), LocalTime) =
                            try!(wire::ReadFrom::read_from(r));
        Ok(Motion {
            start_pos: a,
            start_time: b,
            end_pos: c,
            end_time: d,
        })
    }
}

impl wire::WriteTo for Motion {
    fn write_to<W: Write>(&self, w: &mut WireWriter<W>) -> io::Result<()> {
        try!(self.start_pos.write_to(w));
        try!(self.start_time.write_to(w));
        try!(self.end_pos.write_to(w));
        try!(self.end_time.write_to(w));
        Ok(())
    }

    fn size(&self) -> usize {
        self.start_pos.size() +
        self.start_time.size() +
        self.end_pos.size() +
        self.end_time.size()
    }

    fn size_is_fixed() -> bool { true }
}


#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum SimpleArg {
    Int(i32),
    Str(String),
}

impl SimpleArg {
    fn into_extra_arg(self) -> ExtraArg {
        match self {
            SimpleArg::Int(x) => ExtraArg::Int(x),
            SimpleArg::Str(x) => ExtraArg::Str(x),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExtraArg {
    Int(i32),
    Str(String),
    List(Vec<ExtraArg>),
    Map(HashMap<SimpleArg, ExtraArg>),
}

impl ExtraArg {
    fn into_simple_arg(self) -> Result<SimpleArg, ExtraArg> {
        match self {
            ExtraArg::Int(x) => Ok(SimpleArg::Int(x)),
            ExtraArg::Str(x) => Ok(SimpleArg::Str(x)),
            e => Err(e),
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ArgTag {
    Int = 0,
    Str = 1,
    List = 2,
    Map = 3,
}

impl wire::ReadFrom for ArgTag {
    fn read_from<R: Read>(r: &mut WireReader<R>) -> io::Result<ArgTag> {
        let tag = match try!(r.read::<u8>()) {
            // TODO: figure out how to use `ArgTag::Int as u8` constants
            0 => ArgTag::Int,
            1 => ArgTag::Str,
            2 => ArgTag::List,
            3 => ArgTag::Map,
            x => return Err(io::Error::new(io::ErrorKind::Other,
                                           format!("bad ArgTag variant: {}", x))),
        };
        Ok(tag)
    }
}

impl wire::WriteTo for ArgTag {
    fn write_to<W: Write>(&self, w: &mut WireWriter<W>) -> io::Result<()> {
        w.write(*self as u8)
    }

    fn size(&self) -> usize { (*self as u8).size() }

    fn size_is_fixed() -> bool { true }
}


impl wire::ReadFrom for SimpleArg {
    fn read_from<R: Read>(r: &mut WireReader<R>) -> io::Result<SimpleArg> {
        let arg = try!(r.read::<ExtraArg>());
        let simple = unwrap_or!(arg.into_simple_arg().ok(),
                                return Err(io::Error::new(io::ErrorKind::Other,
                                                          "non-simple SimpleArg")));
        Ok(simple)
    }
}

impl wire::WriteTo for SimpleArg {
    fn write_to<W: Write>(&self, w: &mut WireWriter<W>) -> io::Result<()> {
        use self::SimpleArg::*;
        match *self {
            Int(i) => w.write((ArgTag::Int, i)),
            Str(ref s) => w.write((ArgTag::Str, s)),
        }
    }

    fn size(&self) -> usize {
        use self::SimpleArg::*;
        let inner_size = match *self {
            Int(i) => i.size(),
            Str(ref s) => s.size(),
        };
        1 + inner_size
    }

    fn size_is_fixed() -> bool { false }
}


impl wire::ReadFrom for ExtraArg {
    fn read_from<R: Read>(r: &mut WireReader<R>) -> io::Result<ExtraArg> {
        use self::ExtraArg::*;
        let arg = match try!(r.read()) {
            ArgTag::Int => Int(try!(r.read())),
            ArgTag::Str => Str(try!(r.read())),
            ArgTag::List => List(try!(r.read())),
            ArgTag::Map => Map(try!(r.read())),
        };
        Ok(arg)
    }
}

impl wire::WriteTo for ExtraArg {
    fn write_to<W: Write>(&self, w: &mut WireWriter<W>) -> io::Result<()> {
        use self::ExtraArg::*;
        match *self {
            Int(i) => w.write((ArgTag::Int, i)),
            Str(ref s) => w.write((ArgTag::Str, s)),
            List(ref l) => w.write((ArgTag::List, l)),
            Map(ref m) => w.write((ArgTag::Map, m)),
        }
    }

    fn size(&self) -> usize {
        use self::ExtraArg::*;
        let inner_size = match *self {
            Int(i) => i.size(),
            Str(ref s) => s.size(),
            List(ref l) => l.size(),
            Map(ref m) => m.size(),
        };
        1 + inner_size
    }

    fn size_is_fixed() -> bool { false }
}

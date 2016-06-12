use std::collections::{HashMap, hash_map};
use rand::{self, Rng};

use libphysics::{CHUNK_SIZE, CHUNK_BITS, TILE_SIZE, TILE_BITS};

use types::*;

use msg;
use world;


pub struct Clients {
    clients: HashMap<ClientId, ClientInfo>,
    wire_map: HashMap<WireId, ClientId>,
    name_map: HashMap<String, ClientId>,
}

pub struct ClientInfo {
    wire_id: WireId,
    name: String,
    chunk_offset: (u8, u8),
}

impl Clients {
    pub fn new() -> Clients {
        Clients {
            clients: HashMap::new(),
            wire_map: HashMap::new(),
            name_map: HashMap::new(),
        }
    }

    pub fn add(&mut self, cid: ClientId, wire_id: WireId, name: &str) {
        let old_client = self.clients.insert(cid, ClientInfo::new(wire_id, name));
        let old_wire = self.wire_map.insert(wire_id, cid);
        let old_name = self.name_map.insert(String::from(name), cid);
        debug_assert!(old_client.is_none());
        debug_assert!(old_wire.is_none());
        debug_assert!(old_name.is_none());
    }

    pub fn remove(&mut self, cid: ClientId) {
        let info = self.clients.remove(&cid).expect("client does not exist");
        self.wire_map.remove(&info.wire_id).expect("client was not in wire_map");
        self.name_map.remove(&info.name).expect("client was not in name_map");
    }

    pub fn wire_to_client(&self, wire_id: WireId) -> Option<ClientId> {
        self.wire_map.get(&wire_id).map(|&x| x)
    }

    pub fn name_to_client(&self, name: &str) -> Option<ClientId> {
        self.name_map.get(name).map(|&x| x)
    }

    pub fn get(&self, cid: ClientId) -> Option<&ClientInfo> {
        self.clients.get(&cid)
    }

    pub fn get_mut(&mut self, cid: ClientId) -> Option<&mut ClientInfo> {
        self.clients.get_mut(&cid)
    }

    pub fn iter(&self) -> hash_map::Iter<ClientId, ClientInfo> {
        self.clients.iter()
    }

    pub fn len(&self) -> usize {
        self.clients.len()
    }
}

const LOCAL_BITS: usize = 3;
const LOCAL_SIZE: i32 = 1 << LOCAL_BITS;
const LOCAL_MASK: i32 = LOCAL_SIZE - 1;

impl ClientInfo {
    pub fn new(wire_id: WireId, name: &str) -> ClientInfo {
        let mut rng = rand::thread_rng();
        let offset_x = rng.gen_range(0, 8);
        let offset_y = rng.gen_range(0, 8);
        ClientInfo {
            wire_id: wire_id,
            name: String::from(name),
            chunk_offset: (offset_x, offset_y),
        }
    }

    pub fn wire_id(&self) -> WireId {
        self.wire_id
    }

    pub fn local_chunk_index(&self, cpos: V2) -> u16 {
        let cx = (cpos.x + self.chunk_offset.0 as i32) & LOCAL_MASK;
        let cy = (cpos.y + self.chunk_offset.1 as i32) & LOCAL_MASK;
        (cy * LOCAL_SIZE + cx) as u16
    }

    pub fn local_pos(&self, pos: V3) -> V3 {
        const BASE: i32 = TILE_SIZE * CHUNK_SIZE * (LOCAL_SIZE / 2);
        const MASK: i32 = (1 << (TILE_BITS + CHUNK_BITS + LOCAL_BITS)) - 1;
        let chunk_off = V2::new(self.chunk_offset.0 as i32,
                                self.chunk_offset.1 as i32);
        let adj_pos = pos.reduce() + chunk_off * scalar(CHUNK_SIZE * TILE_SIZE);

        let delta = adj_pos - scalar(BASE);
        let masked = delta & scalar(MASK);
        let local_pos = masked + scalar(BASE);

        local_pos.extend(pos.z)
    }

    pub fn local_pos_tuple(&self, pos: V3) -> (u16, u16, u16) {
        let local = self.local_pos(pos);
        (local.x as u16,
         local.y as u16,
         local.z as u16)
    }
}

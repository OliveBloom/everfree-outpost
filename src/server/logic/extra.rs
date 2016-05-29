use std::collections::HashMap;

use types::*;

use timer;


pub struct Extra {
    pub client_view_update_timer: HashMap<ClientId, timer::Cookie>,
    pub entity_physics_update_timer: HashMap<EntityId, timer::Cookie>,


    /// Info about wires (connections) that have not yet indicated readiness.  The Client object is
    /// loaded only once the wire sends a Ready message, and we need somewhere to store this
    /// information in the meantime.
    ///
    /// Once the wire indicates readiness, its wire_info entry will be removed.
    pub wire_info: HashMap<WireId, (u16, String)>,
}

impl Extra {
    pub fn new() -> Extra {
        Extra {
            client_view_update_timer: HashMap::new(),
            entity_physics_update_timer: HashMap::new(),
            wire_info: HashMap::new(),
        }
    }
}

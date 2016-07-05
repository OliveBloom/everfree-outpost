use std::collections::HashMap;

use types::*;

use timer;


pub struct Extra {
    /// Info about wires (connections) that have not yet indicated readiness.  The Client object is
    /// loaded only once the wire sends a Ready message, and we need somewhere to store this
    /// information in the meantime.
    ///
    /// Once the wire indicates readiness, its wire_info entry will be removed.
    pub wire_info: HashMap<WireId, (u32, String)>,

    /// Map each client to its user ID.  This is used for saving and loading .client files.
    pub client_uid: HashMap<ClientId, u32>,
    pub uid_client: HashMap<u32, ClientId>,
}

impl Extra {
    pub fn new() -> Extra {
        Extra {
            wire_info: HashMap::new(),
            client_uid: HashMap::new(),
            uid_client: HashMap::new(),
        }
    }
}

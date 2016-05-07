use types::*;

use messages::{Messages, ClientResponse};
use pubsub::PubSub;
use world::World;


fn region(cpos: V2) -> Region<V2> {
    Region::new(cpos - scalar(2),
                cpos + scalar(3))
}


pub struct Chat {
    local: PubSub<ClientId, (PlaneId, V2), ClientId>,
}

impl Chat {
    pub fn new() -> Chat {
        Chat {
            local: PubSub::new(),
        }
    }

    pub fn add_client(&mut self, cid: ClientId, pid: PlaneId, cpos: V2) {
        self.local.publish(cid, (pid, cpos), |_,_,_| ());

        for p in region(cpos).points() {
            self.local.subscribe(cid, (pid, p), |_,_,_| ());
        }
    }

    pub fn set_client_location(&mut self,
                               cid: ClientId,
                               old_pid: PlaneId,
                               old_cpos: V2,
                               new_pid: PlaneId,
                               new_cpos: V2) {
        let plane_change = new_pid != old_pid;

        let old_area = region(old_cpos);
        let new_area = region(new_cpos);

        for p in old_area.points().filter(|&p| !new_area.contains(p) || plane_change) {
            self.local.unsubscribe(cid, (old_pid, p), |_,_,_| ());
        }

        for p in new_area.points().filter(|&p| !old_area.contains(p) || plane_change) {
            self.local.subscribe(cid, (new_pid, p), |_,_,_| ());
        }

        self.local.unpublish(cid, (old_pid, old_cpos), |_,_,_| ());
        self.local.publish(cid, (new_pid, new_cpos), |_,_,_| ());
    }

    pub fn remove_client(&mut self, cid: ClientId, pid: PlaneId, cpos: V2) {
        self.local.unpublish(cid, (pid, cpos), |_,_,_| ());

        for p in region(cpos).points() {
            self.local.unsubscribe(cid, (pid, p), |_,_,_| ());
        }
    }


    pub fn send_system(&mut self,
                       messages: &mut Messages,
                       to: ClientId,
                       msg: &str) {
        let msg_out = format!("&s\t***\t{}", msg);
        messages.send_client(to, ClientResponse::ChatUpdate(msg_out));
    }

    pub fn send_global(&mut self,
                       world: &World,
                       messages: &mut Messages,
                       from: ClientId,
                       msg: &str) {
        let msg_out = format!("&g\t<{}>\t{}", world.client(from).name(), msg);
        messages.broadcast_clients(ClientResponse::ChatUpdate(msg_out));
    }

    pub fn send_local(&mut self,
                      world: &World,
                      messages: &mut Messages,
                      from: ClientId,
                      msg: &str) {
        let msg_out = format!("&l\t<{}>\t{}", world.client(from).name(), msg);
        let resp = ClientResponse::ChatUpdate(msg_out);
        self.local.message(&from, |_, &to| {
            messages.send_client(to, resp.clone());
        });
    }
}

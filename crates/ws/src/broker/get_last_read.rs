use actix::{Context, Handler};
use crate::broker::phoenix_manager::{GetLastRead, PhoenixManager};

impl Handler<GetLastRead> for PhoenixManager {
    type Result = Option<String>;

    fn handle(&mut self, msg: GetLastRead, _ctx: &mut Context<Self>) -> Self::Result {
        self
            .last_read
            .get(&(msg.room_id.clone(), msg.local_user_id.clone()))
            .cloned()
    }
}

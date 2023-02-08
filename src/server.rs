use std::sync::Arc;
use tokio::time::Duration;

use crate::room::{ChatRoom, ChatOptions};

pub struct Server{
    port:u16,
    room:Arc<ChatRoom>,
}

impl Server{
    pub fn new(port:u16) -> Self{
        Server{
            port,
            room: Arc::new(
                ChatRoom::new(ChatOptions{
                    heartbeat: Some(Duration::from_secs(5)),
                })
            )
        }
    }
}
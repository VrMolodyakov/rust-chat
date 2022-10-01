use std::collections::HashMap;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use crate::model::user::User;
use std::time::Duration;
use crate::proto::{ UserOutputMessage, Output, OutputError,};
use crate::model::feed::Feed;

const ROOM_SIZE: usize = 32;

#[derive(Clone, Copy, Default)]
pub struct ChatOptions{
    pub alive: Option<Duration>
    
}

pub struct Chat{
    alive:Option<Duration>,
    sender:broadcast::Sender<UserOutputMessage>,
    users: RwLock<HashMap<Uuid,User>>,
    feed:RwLock<Feed>

}

impl Chat {

    pub fn new(options:ChatOptions) -> Self{
        let (sender,_) = broadcast::channel(ROOM_SIZE);
        Chat { 
            alive: options.alive, 
            sender, 
            users: Default::default(), 
            feed: Default::default() 
        }
    }

    async fn send(&self, output: Output) {
        if self.sender.receiver_count() == 0 {
            return;
        }
        self.users.read().await.keys().for_each(|user_id| {
            if let Err(e) = self.sender.send(UserOutputMessage::new(*user_id, output.clone())){
                println!("Could not send message due to: {}", e);
            }
               
        });
    }

    fn target_send(&self,user_id:Uuid,output: Output){
        if self.sender.receiver_count() >= 1{
            if let Err(e) = self.sender.send(UserOutputMessage::new(user_id, output)){
                println!("Could not send message due to: {}", e);
            }
        }
    }

    async fn ignore_send(&self,ignored_id:Uuid,output: Output){
        if self.sender.receiver_count() == 0{
            return;
        }
        self
            .users
            .read()
            .await
            .values()
            .filter(|user| user.id != ignored_id)
            .for_each(|user| {
                if let Err(e) = self.sender.send(UserOutputMessage::new(user.id, output.clone())){
                    println!("Could not send message due to: {}", e);
                }
            })
    }

    fn error(&self,user_id:Uuid,error:OutputError){
        self.target_send(user_id, output)
    }


    
}


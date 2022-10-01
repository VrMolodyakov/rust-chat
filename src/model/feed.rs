use crate::model::message::Message;
use std::collections::BinaryHeap;

#[derive(Default)]
pub struct Feed{
    messages:BinaryHeap<Message>
}

impl Feed{
    pub fn add_message(&mut self,message:Message){
        self.messages.push(message);
    }

    pub fn message_iter(&self) -> impl Iterator<Item = &Message>{
        self.messages.iter()
    }
}
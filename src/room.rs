use std::collections::HashMap;
use tokio::sync::{broadcast, RwLock};
use tokio::time;
use uuid::Uuid;
use chrono::Utc;
use crate::model::message::Message;
use crate::model::user::User;
use tokio::sync::mpsc::UnboundedReceiver;
use std::time::Duration;
use crate::proto::{ UserOutputMessage, Output, OutputError, UserLeftOutput, UserInputMessage, 
                    Input, JoinInput, PostInput,UserOutput, MessageOutput, UserJoinedOutput, 
                    JoinedOutput,PostedOutput, UserPostedOutput};
use crate::model::feed::Feed;
use tokio_stream::wrappers::UnboundedReceiverStream;
use once_cell::sync::Lazy;

const ROOM_SIZE: usize = 32;
const MAX_MESSAGE_LENGTH: usize = 512;

static USERNAME_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new("[A-Za-z\\s]{4,24}").unwrap());

#[derive(Clone, Copy, Default)]
pub struct ChatOptions{
    pub heartbeat: Option<Duration>
    
}

pub struct ChatRoom{
    heartbeat:Option<Duration>,
    sender:broadcast::Sender<UserOutputMessage>,
    users: RwLock<HashMap<Uuid,User>>,
    feed:RwLock<Feed>

}

impl ChatRoom {

    pub fn new(options:ChatOptions) -> Self{
        let (sender,_) = broadcast::channel(ROOM_SIZE);
        ChatRoom { 
            heartbeat: options.heartbeat, 
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
        self.target_send(user_id, Output::Error(error));
    }

    pub fn subscribe(&self) ->broadcast::Receiver<UserOutputMessage>{
        self.sender.subscribe()
    }

    pub async fn on_disconnect(&self,user_id:Uuid){
        if self.users.write().await.remove(&user_id).is_some() {
            self.ignore_send(user_id, Output::UserLeft(UserLeftOutput::new(user_id))).await;
        }
    }

    async fn alive(&self){
        let interval = if let Some(interval) = self.heartbeat{
            interval
        }else{
            return;
        };
        loop{
            time::sleep( interval).await;
            self.send(Output::Alive).await;
        }


    }

    pub async fn run(&self,receiver:UnboundedReceiver<UserInputMessage>){
        let ticking_alive = self.alive();
        let receiver = UnboundedReceiverStream::new(receiver);
        
        tokio::select! {
            _ = ticking_alive => {},
        }
    }
    
    async fn process(&self,input_message:UserInputMessage){
        match  input_message.input {
            Input::Join(input) => self.join_process(input_message.client_id, input).await,
            Input::Post(input) => self.post_process(input_message.client_id, input).await,
        }
    }

    async fn join_process(&self,user_id:Uuid,input:JoinInput){
        let username = input.name.trim();

        if self
            .users
            .read()
            .await
            .values()
            .any(|user| user.nickname == username){
            self.error(user_id, OutputError::NameTaken)
        } 

        if !USERNAME_REGEX.is_match(username){
            self.error(user_id, OutputError::InvalidName);
            return;
        }
        let new_user = User::new(user_id, username);
        self.users.write().await.insert(user_id, new_user);
        
        let user_output = UserOutput::new(user_id, username);
        let other_users = self
            .users
            .read()
            .await
            .values()
            .filter_map(|user| {
                if user.id != user_id {
                    Some(UserOutput::new(user.id, &user.nickname))
                } else {
                    None
                }
            })
            .collect();

        let messages = 
            self
            .feed
            .read()
            .await
            .message_iter()
            .map(|message| {
                MessageOutput::new(
                    message.id,
                    UserOutput::new(message.user.id,&message.user.nickname),
                    &message.content,
                    message.published_at
                )
            })
            .collect();

        self.target_send(
            user_id, 
            Output::Joined(
                JoinedOutput::new(user_output.clone(),other_users,messages)
            )
        );

        self.ignore_send(
            user_id, 
            Output::UserJoined(
                UserJoinedOutput::new(user_output)
            )
        ).await;
        



    }

    async fn post_process(&self,user_id:Uuid,input:PostInput){

        let user = if let Some(user) = self.users.read().await.get(&user_id){
            user.clone()
        }else{
            self.error(user_id, OutputError::NotJoined);
            return;
        };

        if input.body.is_empty() || input.body.len() > MAX_MESSAGE_LENGTH{
            self.error(user_id, OutputError::InvalidMessageBody);
            return;
        }

        let message = Message::new(Uuid::new_v4(),user.clone(),&input.body,Utc::now());
        self.feed.write().await.add_message(message.clone());

        let message_output = MessageOutput::new(
            message.id,
      UserOutput::new(user.id, &user.nickname),
            &message.content,
            message.published_at
        );

        self.target_send(
            user_id,
            Output::Posted(PostedOutput::new(message_output.clone())),
        );

        self.ignore_send(
            user_id,
             Output::UserPosted(UserPostedOutput::new(message_output))
        ).await;
    }

}


use chrono::prelude::*;
use uuid::Uuid;
use crate::model::user::User;
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq,Eq)]
pub struct Message{
    pub id:Uuid,
    pub user:User,
    pub content:String,
    pub published_at:DateTime<Utc>,
}

impl Message {
    pub fn new(id:Uuid,user:User,content:&str,published_at:DateTime<Utc>) -> Self{
        Message { id, user, content:String::from(content), published_at }
    }
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> Ordering {
        other.published_at.cmp(&self.published_at)
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.published_at.partial_cmp(&other.published_at)
    }
}


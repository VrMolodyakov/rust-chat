use uuid::Uuid;

#[derive(Debug, Clone, PartialEq,Eq)]
pub struct User{
    pub nickname: String,
    pub id: Uuid,
}

impl User{
    pub fn new(id:Uuid,nickname:&str) -> Self{
        User{
            id,
            nickname:String::from(nickname),
        }
    }
}
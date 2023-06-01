use std::fmt::Debug;
use std::error::Error;
use std::collections::HashMap;
use teloxide::prelude::*;

#[derive(Debug)]
#[derive(Clone)]
#[derive(serde::Serialize)]
#[derive(PartialEq)]
pub struct ChatMember {
    pub id: UserId,
    pub name: Option<String>,
}

impl ChatMember {
    pub fn try_from(_update: &Update) -> Result<ChatMember, Box<dyn Error>> { // TODO: change error type, use tryFrom trait
        todo!()
    }
}

pub struct ChatData {
    id: ChatId,
    users: HashMap<UserId, ChatMember>,
}

impl ChatData {
    pub fn new(chat_id: ChatId) -> Self {
        Self {
            id: chat_id,
            users: HashMap::new(),
        }
    }

    pub fn update_user(&mut self, user: ChatMember) -> ChannelUserUpdateResult {
        let user_id = user.id;
        match self.users.insert(user_id, user)
        {
            None => ChannelUserUpdateResult::NewEntry,
            Some(old) => {
                if Some(&old) == self.users.get(&user_id) {
                    ChannelUserUpdateResult::NoChanges
                } else {
                    ChannelUserUpdateResult::Update(old)
                }
            }
        }
    }

}


#[derive(Debug)]
#[derive(PartialEq)]
pub enum ChannelUserUpdateResult {
    NewEntry,

    // contains old user state
    Update(ChatMember),

    NoChanges,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_member() {
        let id = UserId(1);
        let name = Some("Alice".to_string());
        let chat_member = ChatMember { id, name: name.clone() };

        assert_eq!(chat_member.id, id);
        assert_eq!(chat_member.name, name);
    }

    #[test]
    fn test_chat_data_new() {
        let id = ChatId(1);
        let chat_data = ChatData::new(id);

        assert_eq!(chat_data.id, id);
        assert_eq!(chat_data.users.len(), 0);
    }

    #[test]
    fn test_chat_data_update_new_entry() {
        let id = ChatId(1);
        let mut chat_data = ChatData::new(id);

        let user_id = UserId(2);
        let user_name = Some("Bob".to_string());
        let user = ChatMember { id: user_id, name: user_name };

        let result = chat_data.update_user(user);

        assert_eq!(result, ChannelUserUpdateResult::NewEntry);
        assert_eq!(chat_data.users.len(), 1);
        assert!(chat_data.users.contains_key(&user_id));
    }

    #[test]
    fn test_chat_data_update_no_changes() {
        let id = ChatId(1);
        let mut chat_data = ChatData::new(id);

        let user_id = UserId(2);
        let user_name = Some("Bob".to_string());
        let user = ChatMember { id: user_id, name: user_name.clone() };

        chat_data.update_user(user.clone());
        let result = chat_data.update_user(user);

        assert_eq!(result, ChannelUserUpdateResult::NoChanges);
        assert_eq!(chat_data.users.len(), 1);
        assert_eq!(chat_data.users.get(&user_id).unwrap().name, user_name);
    }

    #[test]
    fn test_chat_data_update_update() {
        let id = ChatId(1);
        let mut chat_data = ChatData::new(id);

        let user_id = UserId(2);
        let user_name = Some("Bob".to_string());
        let user = ChatMember { id: user_id, name: user_name.clone() };

        chat_data.update_user(user);

        let new_user_name = Some("Alice".to_string());
        let updated_user = ChatMember { id: user_id, name: new_user_name.clone() };
        let result = chat_data.update_user(updated_user);

        match result {
            ChannelUserUpdateResult::Update(old_user) => {
                assert_eq!(old_user.name, user_name);
            },
            _ => panic!("Expected Update, got something else"),
        }

        assert_eq!(chat_data.users.len(), 1);
        assert_eq!(chat_data.users.get(&user_id).unwrap().name, new_user_name);
    }
}

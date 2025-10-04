// #[serde(untagged)]
use std::sync::Arc;
// #[serde(untagged)]

use dashmap::DashMap;
use nanoid::nanoid;
use serde::Deserialize;
#[derive(Deserialize, serde::Serialize, Clone)]
pub struct User {
    pub name: String,
    pub userid: String,
}
impl User {
    fn new(name: String) -> User {
        let user = User {
            name: name,
            userid: nanoid!(),
        };
        return user;
    }
}
#[derive(Clone)]
pub struct Users {
    pub users_map: Arc<DashMap<String, User>>,
}
impl Users {
    pub fn new() -> Users {
        Users {
            users_map: Arc::new(DashMap::new()),
        }
    }
    pub fn add_new_user(&self, name: String) -> User {
        let user = User::new(name);
        let user_id = user.userid.clone();
        self.users_map.insert(user_id, user.clone());
        return user;
    }
}

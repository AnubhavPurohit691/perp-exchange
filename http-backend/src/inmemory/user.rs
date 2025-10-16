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
    // #[serde(untagged)]
}
impl User {
    fn new(name: String) -> User {
        User {
            name,
            userid: nanoid!(),
        }
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
    pub fn add_new_user(&self, name: String) {
        let user = User::new(name);
        let user_id = user.userid.clone();
        self.users_map.insert(user_id, user.clone());
    }
    pub fn getuser(&self, userid: &str) -> Option<User> {
        self.users_map.get(userid).map(|u| u.clone())
    }
}

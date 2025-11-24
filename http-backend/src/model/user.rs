use std::collections::HashMap;

use nanoid::nanoid;
use rust_decimal::Decimal;
use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
pub struct User {
    pub userid: String,
    pub name: String,
    pub balance: Decimal,
    pub quantity: Decimal,
}

impl User {
    pub fn new(name: String) -> User {
        User {
            name,
            userid: nanoid!(),
            balance: Decimal::new(100000000, 3),
            quantity: Decimal::new(10, 0),
        }
    }
}

pub struct Users {
    pub users: HashMap<String, User>,
}
impl Users {
    pub fn new() -> Self {
        Users {
            users: HashMap::new(),
        }
    }
    pub fn getusermut(&mut self, userid: &str) -> Option<&mut User> {
        self.users.get_mut(userid)
    }
    pub fn add_new_user(&mut self, name: String) -> User {
        let user = User::new(name);
        self.users.insert(user.userid.clone(), user.clone());
        user
    }
    pub fn getuser(&self, userid: &str) -> Option<&User> {
        self.users.get(userid)
    }
}

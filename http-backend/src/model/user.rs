use std::collections::HashMap;

use nanoid::nanoid;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::model::password::hashed_password;
#[derive(Clone, Debug, Serialize)]
pub struct User {
    pub userid: String,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub balance: Decimal,
    pub quantity: Decimal,
}

impl User {
    pub fn new(name: String, email: String, password: String) -> User {
        User {
            name,
            email,
            password_hash: hashed_password(&password),
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
    pub fn add_new_user(
        &mut self,
        name: String,
        email: String,
        password: String,
    ) -> Result<User, String> {
        if self.users.values().any(|user| user.email == email) {
            return Err(String::from("email already registered"));
        }
        let user = User::new(name, email, password);
        println!("{}", user.password_hash);
        self.users.insert(user.userid.clone(), user.clone());
        Ok(user)
    }
    pub fn getuser(&self, userid: &str) -> Option<&User> {
        self.users.get(userid)
    }
    pub fn find_by_email(&self, email: &str) -> Option<&User> {
        self.users.values().find(|user| user.email == email)
    }
}

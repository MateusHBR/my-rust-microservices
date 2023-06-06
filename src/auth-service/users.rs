use std::collections::HashMap;

use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};
use rand_core::OsRng;
use uuid::Uuid;

pub trait Users {
    fn create_user(&mut self, username: String, password: String) -> Result<(), String>;
    fn get_user_uuid(&self, username: String, password: String) -> Option<String>;
    fn delete_user(&mut self, user_uuid: String);
}

#[derive(Clone)]
pub struct User {
    user_uuid: String,
    username: String,
    password: String,
}

impl User {
    fn new(user_uuid: String, username: String, password: String) -> Self {
        Self {
            user_uuid,
            username,
            password,
        }
    }
}

#[derive(Default)]
pub struct UsersImpl {
    uuid_to_user: HashMap<String, User>,
    username_to_user: HashMap<String, User>,
}

impl Users for UsersImpl {
    fn create_user(&mut self, username: String, password: String) -> Result<(), String> {
        if self.username_to_user.contains_key(&username) {
            return Err(format!("User already exists"));
        }

        let salt = SaltString::generate(&mut OsRng);
        let hashed_password = Pbkdf2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|err| format!("Failed to hash password. \n{err:?}"))?
            .to_string();

        let user_uuid = Uuid::new_v4().to_string();
        let user = User::new(user_uuid, username, hashed_password);

        self.uuid_to_user
            .insert(user.user_uuid.clone(), user.clone());
        self.username_to_user.insert(user.username.clone(), user);

        Ok(())
    }

    fn get_user_uuid(&self, username: String, password: String) -> Option<String> {
        let user: &User = self.username_to_user.get(&username)?;

        let hashed_password = user.password.clone();
        let parsed_hash = PasswordHash::new(&hashed_password).ok()?;

        let result = Pbkdf2.verify_password(password.as_bytes(), &parsed_hash);

        if result.is_ok() {
            return Some(user.user_uuid.clone());
        };

        None
    }

    fn delete_user(&mut self, user_uuid: String) {
        let user = self.uuid_to_user.get(&user_uuid);
        if user.is_none() {
            return;
        }

        let user = user.unwrap();
        self.username_to_user.remove(&user.username.clone());
        self.uuid_to_user.remove(&user_uuid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_user() {
        let mut user_service = UsersImpl::default();
        user_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("Should create user");

        assert_eq!(user_service.uuid_to_user.len(), 1);
        assert_eq!(user_service.username_to_user.len(), 1);
    }

    #[test]
    fn should_fail_creating_user_with_existing_username() {
        let mut user_service = UsersImpl::default();
        user_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("Should create user");

        let result = user_service.create_user("username".to_owned(), "password".to_owned());

        assert!(result.is_err());
    }

    #[test]
    fn should_retrieve_user_uuid() {
        let mut user_service = UsersImpl::default();
        user_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("Should create user");

        assert!(user_service
            .get_user_uuid("username".to_owned(), "password".to_owned())
            .is_some())
    }

    #[test]
    fn should_fail_to_retrieve_on_inesistent_user_or_incorrect_password() {
        let mut user_service = UsersImpl::default();
        assert!(user_service
            .get_user_uuid("username".to_owned(), "password".to_owned())
            .is_none());

        user_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("Should create user");

        assert!(user_service
            .get_user_uuid("username".to_owned(), "incorrect_password".to_owned())
            .is_none())
    }

    #[test]
    fn should_delete_user() {
        let mut user_service = UsersImpl::default();

        user_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("Should create user");
        assert_eq!(user_service.username_to_user.len(), 1);
        assert_eq!(user_service.uuid_to_user.len(), 1);

        let user_uuid = user_service
            .get_user_uuid("username".to_owned(), "password".to_owned())
            .expect("User not found");

        user_service.delete_user(user_uuid);
        assert_eq!(user_service.username_to_user.len(), 0);
        assert_eq!(user_service.uuid_to_user.len(), 0);
    }
}

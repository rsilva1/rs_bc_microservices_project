use std::collections::HashMap;

use pbkdf2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Pbkdf2
};
use rand_core::OsRng;
use uuid::Uuid;

pub trait Users {
    fn get_user_uuid(&self, username: String, password: String) -> Option<String>;
    fn create_user(&mut self, username: String, password: String) -> Result<(), String>;
    fn delete_user(&mut self, user_uuid: String);
}

#[derive(Clone)]
pub struct User {
    user_uuid: String,
    username: String,
    password: String,
}

#[derive(Default)]
pub struct UsersImpl {
    user_uuid_to_user: HashMap<String, User>,
    username_to_user: HashMap<String, User>,
}

impl Users for UsersImpl {
    fn get_user_uuid(&self, username: String, password: String) -> Option<String> {
        let user = self.username_to_user.get(&username)?;

        let hashed_password = user.password.clone();
        let parsed_hash = PasswordHash::new(&hashed_password).ok()?;

        let result = Pbkdf2.verify_password(password.as_bytes(), &parsed_hash);

        match result {
            Ok(_) => Some(user.user_uuid.clone()),
            _ => None
        }
    }

    fn create_user(&mut self, username: String, password: String) -> Result<(), String> {
        if let Some(_) = self.username_to_user.get(&username) {
            return Err(format!("User with {} username already exists!", username));
        }

        let salt = SaltString::generate(&mut OsRng);

        let hashed_password = Pbkdf2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Failed to hash password.\n{e:?}"))?
            .to_string();

        let user = User {
            user_uuid: Uuid::new_v4().to_string(),
            username,
            password: hashed_password,
        };

        self.user_uuid_to_user.insert(user.user_uuid.clone(), user.clone());
        self.username_to_user.insert(user.username.clone(), user);

        Ok(())
    }

    fn delete_user(&mut self, user_uuid: String) {
        if let Some(user) = self.user_uuid_to_user.get(&user_uuid) {
            let username = user.username.clone();
            self.user_uuid_to_user.remove(&user_uuid);
            self.username_to_user.remove(&username);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Users, UsersImpl};

    #[test]
    fn should_create_user() {
        let mut users_service = UsersImpl::default();
        users_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("should create user");

        assert_eq!(users_service.user_uuid_to_user.len(), 1);
        assert_eq!(users_service.username_to_user.len(), 1);
    }

    #[test]
    fn should_fail_creating_user_with_existing_username() {
        let mut users_service = UsersImpl::default();

        users_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("should create user");

        let result = users_service
            .create_user("username".to_owned(), "password".to_owned());

        assert!(result.is_err());
    }

    #[test]
    fn should_retrieve_user_uuid() {
        let mut users_service = UsersImpl::default();

        users_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("should create user");

        let option = users_service
            .get_user_uuid("username".to_owned(), "password".to_owned());

        assert!(option.is_some());
    }

    #[test]
    fn should_fail_to_retrieve_user_uuid_with_false_password() {
        let mut users_service = UsersImpl::default();

        users_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("should create user");

        assert!(users_service
            .get_user_uuid("username".to_owned(), "false".to_owned())
            .is_none());
    }

    #[test]
    fn should_delete_user() {
        let mut users_service = UsersImpl::default();

        users_service
            .create_user("username".to_owned(), "password".to_owned())
            .expect("should create user");

        let user_uuid = users_service
            .get_user_uuid("username".to_owned(), "password".to_owned())
            .expect("should get user uuid");

        users_service.delete_user(user_uuid);

        assert_eq!(users_service.username_to_user.len(), 0);
        assert_eq!(users_service.user_uuid_to_user.len(), 0);
    }
}

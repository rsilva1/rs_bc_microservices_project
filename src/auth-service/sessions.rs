use std::collections::HashMap;

use uuid::Uuid;

pub trait Sessions {
    fn create_session(&mut self, user_uuid: &str) -> String;
    fn delete_session(&mut self, session_token: &str);
}

#[derive(Default)]
pub struct SessionsImpl {
    uuid_to_session: HashMap<String, String>,
    session_to_uuid: HashMap<String, String>
}

impl Sessions for SessionsImpl {
    fn create_session(&mut self, user_uuid: &str) -> String {
        let session = Uuid::new_v4().to_string();
        self.uuid_to_session.insert(user_uuid.to_string(), session.clone());
        self.session_to_uuid.insert(session.clone(), user_uuid.to_string());
        session
    }

    fn delete_session(&mut self, session_token: &str) {
        if let Some(user_uuid) = self.session_to_uuid.get(session_token) {
            self.uuid_to_session.remove(user_uuid);
            self.session_to_uuid.remove(session_token);
        }
        ()
    }
}

#[cfg(test)]
mod tests {
    use super::{Sessions, SessionsImpl};

    #[test]
    fn should_create_session() {
        let mut sessions_service = SessionsImpl::default();
        assert_eq!(sessions_service.uuid_to_session.len(), 0);

        let session = sessions_service.create_session("123");
        assert_eq!(sessions_service.uuid_to_session.len(), 1);
        assert_eq!(sessions_service.uuid_to_session.get("123").unwrap(), &session);
    }

    #[test]
    fn should_delete_session() {
        let mut sessions_service = SessionsImpl::default();

        let session_token = sessions_service.create_session("123");
        assert_eq!(sessions_service.uuid_to_session.len(), 1);

        sessions_service.delete_session(&session_token);
        assert_eq!(sessions_service.uuid_to_session.len(), 0);
    }
}

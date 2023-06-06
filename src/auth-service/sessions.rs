use std::collections::HashMap;
use uuid::Uuid;

pub trait Sessions {
    fn create_session(&mut self, user_uuid: &str) -> String;
    fn delete_session(&mut self, user_uuid: &str);
}

#[derive(Default)]
pub struct SessionsImpl {
    uuid_to_session: HashMap<String, String>,
}

impl Sessions for SessionsImpl {
    fn create_session(&mut self, user_uuid: &str) -> String {
        let session = Uuid::new_v4().to_string();
        self.uuid_to_session
            .insert(user_uuid.to_owned(), session.clone());
        session
    }

    fn delete_session(&mut self, user_uuid: &str) {
        self.uuid_to_session.remove(user_uuid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_session() {
        let mut sessions = SessionsImpl::default();
        assert_eq!(sessions.uuid_to_session.len(), 0);

        let session = sessions.create_session("user_uuid");
        let result = sessions.uuid_to_session.get("user_uuid").unwrap();
        assert_eq!(sessions.uuid_to_session.len(), 1);
        assert_eq!(&session, result);
    }

    #[test]
    fn should_delete_session() {
        let mut sessions = SessionsImpl::default();
        sessions.create_session("user_uuid");
        assert_eq!(sessions.uuid_to_session.len(), 1);

        sessions.delete_session("user_uuid");
        assert_eq!(sessions.uuid_to_session.len(), 0);
    }
}

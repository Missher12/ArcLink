use arclink_common::RemoteSession;
use std::sync::{Arc, Mutex};

pub struct ViewerSessionManager {
    active_session: Arc<Mutex<Option<RemoteSession>>>,
}

impl ViewerSessionManager {
    pub fn new() -> Self {
        Self {
            active_session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_session(&self, session: RemoteSession) {
        let mut s = self.active_session.lock().unwrap();
        *s = Some(session);
    }

    pub fn end_session(&self) {
        let mut s = self.active_session.lock().unwrap();
        *s = None;
    }

    pub fn get_active_session(&self) -> Option<RemoteSession> {
        self.active_session.lock().unwrap().clone()
    }
}

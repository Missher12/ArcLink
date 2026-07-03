use arclink_common::RemoteSession;
use std::sync::{Arc, Mutex};

pub struct HostSessionManager {
    active_session: Arc<Mutex<Option<RemoteSession>>>,
}

impl HostSessionManager {
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

    pub fn is_session_authorized(&self, session_id: &str, viewer_ip: &str) -> bool {
        if let Some(ref active) = *self.active_session.lock().unwrap() {
            active.session_id == session_id && active.viewer_ip == viewer_ip && active.allow_control
        } else {
            false
        }
    }
}

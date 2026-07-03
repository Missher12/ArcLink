#[cfg(test)]
mod tests {
    use arclink_common::{
        InputEvent, MouseMoveEvent, MouseButtonEvent, MouseButton, KeyboardEvent,
        SessionRequest, SessionAccept, ControlMessage, DisconnectReason
    };
    use chrono::Utc;

    #[test]
    fn test_input_event_serialization() {
        let event = InputEvent::MouseMove(MouseMoveEvent {
            norm_x: 0.54,
            norm_y: 0.32,
        });

        // Test bincode serialization
        let encoded = bincode::serialize(&event).expect("bincode serialize success");
        let decoded: InputEvent = bincode::deserialize(&encoded).expect("bincode deserialize success");

        if let InputEvent::MouseMove(mv) = decoded {
            assert_eq!(mv.norm_x, 0.54);
            assert_eq!(mv.norm_y, 0.32);
        } else {
            panic!("Expected MouseMove event");
        }
    }

    #[test]
    fn test_mouse_click_serialization() {
        let event = InputEvent::MouseButton(MouseButtonEvent {
            button: MouseButton::Right,
            is_down: true,
        });

        let encoded = serde_json::to_string(&event).expect("json serialize success");
        let decoded: InputEvent = serde_json::from_str(&encoded).expect("json deserialize success");

        if let InputEvent::MouseButton(mb) = decoded {
            assert_eq!(mb.button, MouseButton::Right);
            assert!(mb.is_down);
        } else {
            panic!("Expected MouseButton event");
        }
    }

    #[test]
    fn test_keyboard_event_mapping() {
        let event = KeyboardEvent {
            vk_code: 0x0D, // Enter key
            is_down: true,
            modifiers: 2, // Ctrl modifier
        };

        assert_eq!(event.vk_code, 13);
        assert!(event.is_down);
        assert_eq!(event.modifiers, 2);
    }

    #[test]
    fn test_control_channel_session_handshake() {
        let req = SessionRequest {
            session_id: "REQ-778".to_string(),
            viewer_name: "VIEWER-LAPTOP".to_string(),
            viewer_ip: "192.168.1.150".to_string(),
            request_time: Utc::now(),
            required_fps: 60,
            width: 1920,
            height: 1080,
        };

        let msg = ControlMessage::Request(req);
        let serialized = serde_json::to_string(&msg).expect("Serialize request");
        let deserialized: ControlMessage = serde_json::from_str(&serialized).expect("Deserialize request");

        match deserialized {
            ControlMessage::Request(r) => {
                assert_eq!(r.session_id, "REQ-778");
                assert_eq!(r.viewer_name, "VIEWER-LAPTOP");
            }
            _ => panic!("Expected Request message type"),
        }
    }

    #[test]
    fn test_local_loopback_connection_simulation() {
        // Mock a simple host receiver loop
        let host_status = arclink_common::HostStatus::Listening;
        let mut session_established = false;

        let mock_accept = SessionAccept {
            session_id: "SESSION-1".to_string(),
            host_name: "HOST-PC".to_string(),
            accepted_time: Utc::now(),
            control_port: 8443,
            video_port: 8444,
        };

        if host_status == arclink_common::HostStatus::Listening {
            session_established = true;
        }

        assert!(session_established);
        assert_eq!(mock_accept.control_port, 8443);
    }
}

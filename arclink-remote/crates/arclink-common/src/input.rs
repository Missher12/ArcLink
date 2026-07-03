use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseMoveEvent {
    /// Normalized coordinate X (0.0 to 1.0)
    pub norm_x: f32,
    /// Normalized coordinate Y (0.0 to 1.0)
    pub norm_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub is_down: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseWheelEvent {
    pub delta_x: f32,
    pub delta_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEvent {
    pub vk_code: u16,  // Virtual-key code (e.g. standard Windows Virtual Keys)
    pub is_down: bool,
    pub modifiers: u8, // Flag bits: 1=Shift, 2=Ctrl, 4=Alt, 8=Win
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    MouseMove(MouseMoveEvent),
    MouseButton(MouseButtonEvent),
    MouseWheel(MouseWheelEvent),
    Keyboard(KeyboardEvent),
}

impl InputEvent {
    /// Validates normalized coordinates
    pub fn is_valid_mouse_pos(&self) -> bool {
        if let InputEvent::MouseMove(mv) = self {
            mv.norm_x >= 0.0 && mv.norm_x <= 1.0 && mv.norm_y >= 0.0 && mv.norm_y <= 1.0
        } else {
            true
        }
    }
}

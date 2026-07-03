use arclink_common::{InputEvent, KeyboardEvent, MouseButton, MouseButtonEvent, MouseMoveEvent, MouseWheelEvent};

pub fn map_key_to_vk(key: eframe::egui::Key) -> u16 {
    match key {
        eframe::egui::Key::Enter => 0x0D,      // VK_RETURN
        eframe::egui::Key::Backspace => 0x08,  // VK_BACK
        eframe::egui::Key::Tab => 0x09,        // VK_TAB
        eframe::egui::Key::Escape => 0x1B,     // VK_ESCAPE
        eframe::egui::Key::Space => 0x20,      // VK_SPACE
        eframe::egui::Key::ArrowLeft => 0x25,  // VK_LEFT
        eframe::egui::Key::ArrowUp => 0x26,    // VK_UP
        eframe::egui::Key::ArrowRight => 0x27, // VK_RIGHT
        eframe::egui::Key::ArrowDown => 0x28,  // VK_DOWN
        eframe::egui::Key::A => 0x41,
        eframe::egui::Key::B => 0x42,
        eframe::egui::Key::C => 0x43,
        eframe::egui::Key::D => 0x44,
        eframe::egui::Key::E => 0x45,
        eframe::egui::Key::F => 0x46,
        eframe::egui::Key::G => 0x47,
        eframe::egui::Key::H => 0x48,
        eframe::egui::Key::I => 0x49,
        eframe::egui::Key::J => 0x4A,
        eframe::egui::Key::K => 0x4B,
        eframe::egui::Key::L => 0x4C,
        eframe::egui::Key::M => 0x4D,
        eframe::egui::Key::N => 0x4E,
        eframe::egui::Key::O => 0x4F,
        eframe::egui::Key::P => 0x50,
        eframe::egui::Key::Q => 0x51,
        eframe::egui::Key::R => 0x52,
        eframe::egui::Key::S => 0x53,
        eframe::egui::Key::T => 0x54,
        eframe::egui::Key::U => 0x55,
        eframe::egui::Key::V => 0x56,
        eframe::egui::Key::W => 0x57,
        eframe::egui::Key::X => 0x58,
        eframe::egui::Key::Y => 0x59,
        eframe::egui::Key::Z => 0x5A,
        eframe::egui::Key::Num0 => 0x30,
        eframe::egui::Key::Num1 => 0x31,
        eframe::egui::Key::Num2 => 0x32,
        eframe::egui::Key::Num3 => 0x33,
        eframe::egui::Key::Num4 => 0x34,
        eframe::egui::Key::Num5 => 0x35,
        eframe::egui::Key::Num6 => 0x36,
        eframe::egui::Key::Num7 => 0x37,
        eframe::egui::Key::Num8 => 0x38,
        eframe::egui::Key::Num9 => 0x39,
        _ => 0,
    }
}

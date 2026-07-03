use arclink_common::{InputEvent, MouseButton};

pub struct InputInjector;

impl InputInjector {
    pub fn new() -> Self {
        Self
    }

    pub fn inject_event(&self, event: &InputEvent) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::Input::KeyboardAndMouse::*;
            match event {
                InputEvent::MouseMove(mv) => {
                    // Map 0.0-1.0 to Windows absolute input coordinates (0-65535)
                    let x = (mv.norm_x * 65535.0) as i32;
                    let y = (mv.norm_y * 65535.0) as i32;
                    let input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: x,
                                dy: y,
                                mouseData: 0,
                                dwFlags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
                }
                InputEvent::MouseButton(mb) => {
                    let flags = match (mb.button, mb.is_down) {
                        (MouseButton::Left, true) => MOUSEEVENTF_LEFTDOWN,
                        (MouseButton::Left, false) => MOUSEEVENTF_LEFTUP,
                        (MouseButton::Right, true) => MOUSEEVENTF_RIGHTDOWN,
                        (MouseButton::Right, false) => MOUSEEVENTF_RIGHTUP,
                        (MouseButton::Middle, true) => MOUSEEVENTF_MIDDLEDOWN,
                        (MouseButton::Middle, false) => MOUSEEVENTF_MIDDLEUP,
                    };
                    let input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: 0,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
                }
                InputEvent::MouseWheel(mw) => {
                    let flags = MOUSEEVENTF_WHEEL;
                    let mouse_data = (mw.delta_y * 120.0) as i32; // Standard wheel click delta
                    let input = INPUT {
                        r#type: INPUT_MOUSE,
                        Anonymous: INPUT_0 {
                            mi: MOUSEINPUT {
                                dx: 0,
                                dy: 0,
                                mouseData: mouse_data,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
                }
                InputEvent::Keyboard(kb) => {
                    let flags = if kb.is_down { KEYBD_EVENT_FLAGS(0) } else { KEYEVENTF_KEYUP };
                    let input = INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VIRTUAL_KEY(kb.vk_code),
                                wScan: 0,
                                dwFlags: flags,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    };
                    unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
                }
            }
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        {
            // Emulate logs for debugging and platform compatibility
            match event {
                InputEvent::MouseMove(mv) => {
                    println!("[MOCK INPUT] MouseMove: ({:.3}, {:.3})", mv.norm_x, mv.norm_y);
                }
                InputEvent::MouseButton(mb) => {
                    println!("[MOCK INPUT] MouseButton: {:?} down={}", mb.button, mb.is_down);
                }
                InputEvent::MouseWheel(mw) => {
                    println!("[MOCK INPUT] MouseWheel: dy={:.1}", mw.delta_y);
                }
                InputEvent::Keyboard(kb) => {
                    println!("[MOCK INPUT] Keyboard: vk=0x{:X} down={}", kb.vk_code, kb.is_down);
                }
            }
            Ok(())
        }
    }
}

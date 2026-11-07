use gilrs::{Button, Event, EventType, GamepadId, Gilrs};
use std::collections::HashSet;

pub struct GamepadManager {
    gilrs: Gilrs,
    pub active_gamepad: Option<GamepadId>,
    pub active_name: String,
    pub is_xbox_layout: bool,
    pub is_playstation_layout: bool,
    pub is_nintendo_layout: bool,
    pub pressed_buttons: HashSet<Button>,
    pub just_pressed_buttons: HashSet<Button>,
    pub left_stick: (f32, f32),
    pub right_stick: (f32, f32),
    stick_nav_cooldown: u32,
}

impl GamepadManager {
    pub fn new() -> Self {
        let gilrs = Gilrs::new().unwrap_or_else(|_| {
            // Fallback if gilrs fails to init
            panic!("Failed to initialize gamepad subsystem");
        });

        let mut mgr = Self {
            gilrs,
            active_gamepad: None,
            active_name: "No Controller Detected".to_string(),
            is_xbox_layout: false,
            is_playstation_layout: false,
            is_nintendo_layout: false,
            pressed_buttons: HashSet::new(),
            just_pressed_buttons: HashSet::new(),
            left_stick: (0.0, 0.0),
            right_stick: (0.0, 0.0),
            stick_nav_cooldown: 0,
        };

        mgr.refresh_connected_devices();
        mgr
    }

    pub fn refresh_connected_devices(&mut self) {
        if let Some((id, gamepad)) = self.gilrs.gamepads().next() {
            self.active_gamepad = Some(id);
            let name = gamepad.name().to_string();
            self.active_name = name.clone();

            let lower = name.to_lowercase();
            if lower.contains("xbox") || lower.contains("xinput") {
                self.is_xbox_layout = true;
                self.is_playstation_layout = false;
                self.is_nintendo_layout = false;
            } else if lower.contains("dualshock") || lower.contains("dualsense") || lower.contains("sony") || lower.contains("playstation") {
                self.is_playstation_layout = true;
                self.is_xbox_layout = false;
                self.is_nintendo_layout = false;
            } else if lower.contains("switch") || lower.contains("nintendo") || lower.contains("joy-con") {
                self.is_nintendo_layout = true;
                self.is_xbox_layout = false;
                self.is_playstation_layout = false;
            } else {
                // Default to Xbox for generic PC XInput gamepads
                self.is_xbox_layout = true;
                self.is_playstation_layout = false;
                self.is_nintendo_layout = false;
            }
        } else {
            self.active_gamepad = None;
            self.active_name = "No Controller Detected".to_string();
            self.is_xbox_layout = false;
            self.is_playstation_layout = false;
            self.is_nintendo_layout = false;
        }
    }

    pub fn update(&mut self) {
        self.just_pressed_buttons.clear();
        if self.stick_nav_cooldown > 0 {
            self.stick_nav_cooldown -= 1;
        }

        while let Some(Event { id, event, .. }) = self.gilrs.next_event() {
            self.active_gamepad = Some(id);
            match event {
                EventType::ButtonPressed(btn, _) => {
                    self.pressed_buttons.insert(btn);
                    self.just_pressed_buttons.insert(btn);
                }
                EventType::ButtonReleased(btn, _) => {
                    self.pressed_buttons.remove(&btn);
                }
                EventType::AxisChanged(axis, val, _) => match axis {
                    gilrs::Axis::LeftStickX => self.left_stick.0 = val,
                    gilrs::Axis::LeftStickY => self.left_stick.1 = val,
                    gilrs::Axis::RightStickX => self.right_stick.0 = val,
                    gilrs::Axis::RightStickY => self.right_stick.1 = val,
                    _ => {}
                },
                EventType::Connected => {
                    self.refresh_connected_devices();
                }
                EventType::Disconnected => {
                    self.pressed_buttons.clear();
                    self.just_pressed_buttons.clear();
                    self.refresh_connected_devices();
                }
                _ => {}
            }
        }
    }

    pub fn is_button_pressed(&self, button: Button) -> bool {
        self.pressed_buttons.contains(&button)
    }

    pub fn is_button_just_pressed(&self, button: Button) -> bool {
        self.just_pressed_buttons.contains(&button)
    }

    pub fn check_stick_nav_up(&mut self) -> bool {
        if self.left_stick.1 > 0.6 && self.stick_nav_cooldown == 0 {
            self.stick_nav_cooldown = 15;
            true
        } else {
            false
        }
    }

    pub fn check_stick_nav_down(&mut self) -> bool {
        if self.left_stick.1 < -0.6 && self.stick_nav_cooldown == 0 {
            self.stick_nav_cooldown = 15;
            true
        } else {
            false
        }
    }

    pub fn check_stick_nav_left(&mut self) -> bool {
        if self.left_stick.0 < -0.6 && self.stick_nav_cooldown == 0 {
            self.stick_nav_cooldown = 15;
            true
        } else {
            false
        }
    }

    pub fn check_stick_nav_right(&mut self) -> bool {
        if self.left_stick.0 > 0.6 && self.stick_nav_cooldown == 0 {
            self.stick_nav_cooldown = 15;
            true
        } else {
            false
        }
    }
}

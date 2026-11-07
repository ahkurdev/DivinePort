use crate::config::GlyphStyle;
use eframe::epaint::Color32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ps2Button {
    Cross,
    Circle,
    Square,
    Triangle,
    L1,
    R1,
    L2,
    R2,
    L3,
    R3,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    Select,
    Start,
}

#[derive(Debug, Clone)]
pub struct ButtonPrompt {
    pub label: String,
    pub sub_label: String,
    pub badge_color: Color32,
    pub text_color: Color32,
}

impl Ps2Button {
    pub const ALL: [Ps2Button; 16] = [
        Ps2Button::Cross,
        Ps2Button::Circle,
        Ps2Button::Square,
        Ps2Button::Triangle,
        Ps2Button::L1,
        Ps2Button::R1,
        Ps2Button::L2,
        Ps2Button::R2,
        Ps2Button::L3,
        Ps2Button::R3,
        Ps2Button::DpadUp,
        Ps2Button::DpadDown,
        Ps2Button::DpadLeft,
        Ps2Button::DpadRight,
        Ps2Button::Select,
        Ps2Button::Start,
    ];

    pub fn canonical_ps_name(&self) -> &'static str {
        match self {
            Ps2Button::Cross => "Cross (✕)",
            Ps2Button::Circle => "Circle (○)",
            Ps2Button::Square => "Square (□)",
            Ps2Button::Triangle => "Triangle (△)",
            Ps2Button::L1 => "L1 (Left Bumper)",
            Ps2Button::R1 => "R1 (Right Bumper)",
            Ps2Button::L2 => "L2 (Left Trigger)",
            Ps2Button::R2 => "R2 (Right Trigger)",
            Ps2Button::L3 => "L3 (Left Stick Click)",
            Ps2Button::R3 => "R3 (Right Stick Click)",
            Ps2Button::DpadUp => "D-Pad Up",
            Ps2Button::DpadDown => "D-Pad Down",
            Ps2Button::DpadLeft => "D-Pad Left",
            Ps2Button::DpadRight => "D-Pad Right",
            Ps2Button::Select => "Select",
            Ps2Button::Start => "Start",
        }
    }

    pub fn prompt_info(&self, style: GlyphStyle, detected_is_xbox: bool) -> ButtonPrompt {
        let active_style = match style {
            GlyphStyle::Auto => {
                if detected_is_xbox {
                    GlyphStyle::Xbox
                } else {
                    GlyphStyle::PlayStation
                }
            }
            other => other,
        };

        match active_style {
            GlyphStyle::PlayStation | GlyphStyle::Auto => self.ps_prompt(),
            GlyphStyle::Xbox => self.xbox_prompt(),
            GlyphStyle::Nintendo => self.nintendo_prompt(),
            GlyphStyle::Keyboard => self.keyboard_prompt(),
        }
    }

    fn ps_prompt(&self) -> ButtonPrompt {
        match self {
            Ps2Button::Cross => ButtonPrompt {
                label: "✕".to_string(),
                sub_label: "Confirm / Jump".to_string(),
                badge_color: Color32::from_rgb(41, 128, 185), // PS Blue
                text_color: Color32::WHITE,
            },
            Ps2Button::Circle => ButtonPrompt {
                label: "○".to_string(),
                sub_label: "Cancel / Grab".to_string(),
                badge_color: Color32::from_rgb(231, 76, 60), // PS Red
                text_color: Color32::WHITE,
            },
            Ps2Button::Square => ButtonPrompt {
                label: "□".to_string(),
                sub_label: "Light Attack".to_string(),
                badge_color: Color32::from_rgb(233, 30, 99), // PS Pink
                text_color: Color32::WHITE,
            },
            Ps2Button::Triangle => ButtonPrompt {
                label: "△".to_string(),
                sub_label: "Heavy Attack".to_string(),
                badge_color: Color32::from_rgb(46, 204, 113), // PS Green
                text_color: Color32::WHITE,
            },
            Ps2Button::L1 => ButtonPrompt {
                label: "L1".to_string(),
                sub_label: "Block / Guard".to_string(),
                badge_color: Color32::from_rgb(52, 73, 94),
                text_color: Color32::WHITE,
            },
            Ps2Button::R1 => ButtonPrompt {
                label: "R1".to_string(),
                sub_label: "Special / Lock".to_string(),
                badge_color: Color32::from_rgb(52, 73, 94),
                text_color: Color32::WHITE,
            },
            Ps2Button::L2 => ButtonPrompt {
                label: "L2".to_string(),
                sub_label: "Aim / Magic".to_string(),
                badge_color: Color32::from_rgb(44, 62, 80),
                text_color: Color32::WHITE,
            },
            Ps2Button::R2 => ButtonPrompt {
                label: "R2".to_string(),
                sub_label: "Shoot / Power".to_string(),
                badge_color: Color32::from_rgb(44, 62, 80),
                text_color: Color32::WHITE,
            },
            Ps2Button::L3 => ButtonPrompt {
                label: "L3".to_string(),
                sub_label: "Sprint".to_string(),
                badge_color: Color32::from_rgb(70, 70, 70),
                text_color: Color32::WHITE,
            },
            Ps2Button::R3 => ButtonPrompt {
                label: "R3".to_string(),
                sub_label: "Camera Reset".to_string(),
                badge_color: Color32::from_rgb(70, 70, 70),
                text_color: Color32::WHITE,
            },
            Ps2Button::DpadUp => ButtonPrompt {
                label: "▲".to_string(),
                sub_label: "D-Pad Up".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::DpadDown => ButtonPrompt {
                label: "▼".to_string(),
                sub_label: "D-Pad Down".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::DpadLeft => ButtonPrompt {
                label: "◀".to_string(),
                sub_label: "D-Pad Left".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::DpadRight => ButtonPrompt {
                label: "▶".to_string(),
                sub_label: "D-Pad Right".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::Select => ButtonPrompt {
                label: "SELECT".to_string(),
                sub_label: "Map / Menu".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::Start => ButtonPrompt {
                label: "START".to_string(),
                sub_label: "Pause".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
        }
    }

    fn xbox_prompt(&self) -> ButtonPrompt {
        match self {
            Ps2Button::Cross => ButtonPrompt {
                label: "A".to_string(),
                sub_label: "Cross (PS2)".to_string(),
                badge_color: Color32::from_rgb(46, 204, 113), // Xbox Green
                text_color: Color32::BLACK,
            },
            Ps2Button::Circle => ButtonPrompt {
                label: "B".to_string(),
                sub_label: "Circle (PS2)".to_string(),
                badge_color: Color32::from_rgb(231, 76, 60), // Xbox Red
                text_color: Color32::WHITE,
            },
            Ps2Button::Square => ButtonPrompt {
                label: "X".to_string(),
                sub_label: "Square (PS2)".to_string(),
                badge_color: Color32::from_rgb(41, 128, 185), // Xbox Blue
                text_color: Color32::WHITE,
            },
            Ps2Button::Triangle => ButtonPrompt {
                label: "Y".to_string(),
                sub_label: "Triangle (PS2)".to_string(),
                badge_color: Color32::from_rgb(241, 196, 15), // Xbox Yellow
                text_color: Color32::BLACK,
            },
            Ps2Button::L1 => ButtonPrompt {
                label: "LB".to_string(),
                sub_label: "Left Bumper (L1)".to_string(),
                badge_color: Color32::from_rgb(52, 73, 94),
                text_color: Color32::WHITE,
            },
            Ps2Button::R1 => ButtonPrompt {
                label: "RB".to_string(),
                sub_label: "Right Bumper (R1)".to_string(),
                badge_color: Color32::from_rgb(52, 73, 94),
                text_color: Color32::WHITE,
            },
            Ps2Button::L2 => ButtonPrompt {
                label: "LT".to_string(),
                sub_label: "Left Trigger (L2)".to_string(),
                badge_color: Color32::from_rgb(44, 62, 80),
                text_color: Color32::WHITE,
            },
            Ps2Button::R2 => ButtonPrompt {
                label: "RT".to_string(),
                sub_label: "Right Trigger (R2)".to_string(),
                badge_color: Color32::from_rgb(44, 62, 80),
                text_color: Color32::WHITE,
            },
            Ps2Button::L3 => ButtonPrompt {
                label: "LS".to_string(),
                sub_label: "Left Stick Click".to_string(),
                badge_color: Color32::from_rgb(70, 70, 70),
                text_color: Color32::WHITE,
            },
            Ps2Button::R3 => ButtonPrompt {
                label: "RS".to_string(),
                sub_label: "Right Stick Click".to_string(),
                badge_color: Color32::from_rgb(70, 70, 70),
                text_color: Color32::WHITE,
            },
            Ps2Button::DpadUp => ButtonPrompt {
                label: "D-Pad ↑".to_string(),
                sub_label: "Up".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::DpadDown => ButtonPrompt {
                label: "D-Pad ↓".to_string(),
                sub_label: "Down".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::DpadLeft => ButtonPrompt {
                label: "D-Pad ←".to_string(),
                sub_label: "Left".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::DpadRight => ButtonPrompt {
                label: "D-Pad →".to_string(),
                sub_label: "Right".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::Select => ButtonPrompt {
                label: "VIEW".to_string(),
                sub_label: "Select (PS2)".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::Start => ButtonPrompt {
                label: "MENU".to_string(),
                sub_label: "Start (PS2)".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
        }
    }

    fn nintendo_prompt(&self) -> ButtonPrompt {
        match self {
            Ps2Button::Cross => ButtonPrompt {
                label: "B".to_string(),
                sub_label: "Cross (PS2)".to_string(),
                badge_color: Color32::from_rgb(230, 126, 34),
                text_color: Color32::WHITE,
            },
            Ps2Button::Circle => ButtonPrompt {
                label: "A".to_string(),
                sub_label: "Circle (PS2)".to_string(),
                badge_color: Color32::from_rgb(231, 76, 60),
                text_color: Color32::WHITE,
            },
            Ps2Button::Square => ButtonPrompt {
                label: "Y".to_string(),
                sub_label: "Square (PS2)".to_string(),
                badge_color: Color32::from_rgb(52, 152, 219),
                text_color: Color32::WHITE,
            },
            Ps2Button::Triangle => ButtonPrompt {
                label: "X".to_string(),
                sub_label: "Triangle (PS2)".to_string(),
                badge_color: Color32::from_rgb(46, 204, 113),
                text_color: Color32::WHITE,
            },
            Ps2Button::L1 => ButtonPrompt {
                label: "L".to_string(),
                sub_label: "L (L1)".to_string(),
                badge_color: Color32::from_rgb(52, 73, 94),
                text_color: Color32::WHITE,
            },
            Ps2Button::R1 => ButtonPrompt {
                label: "R".to_string(),
                sub_label: "R (R1)".to_string(),
                badge_color: Color32::from_rgb(52, 73, 94),
                text_color: Color32::WHITE,
            },
            Ps2Button::L2 => ButtonPrompt {
                label: "ZL".to_string(),
                sub_label: "ZL (L2)".to_string(),
                badge_color: Color32::from_rgb(44, 62, 80),
                text_color: Color32::WHITE,
            },
            Ps2Button::R2 => ButtonPrompt {
                label: "ZR".to_string(),
                sub_label: "ZR (R2)".to_string(),
                badge_color: Color32::from_rgb(44, 62, 80),
                text_color: Color32::WHITE,
            },
            Ps2Button::L3 => ButtonPrompt {
                label: "LS".to_string(),
                sub_label: "Left Stick".to_string(),
                badge_color: Color32::from_rgb(70, 70, 70),
                text_color: Color32::WHITE,
            },
            Ps2Button::R3 => ButtonPrompt {
                label: "RS".to_string(),
                sub_label: "Right Stick".to_string(),
                badge_color: Color32::from_rgb(70, 70, 70),
                text_color: Color32::WHITE,
            },
            Ps2Button::DpadUp => ButtonPrompt {
                label: "D-Pad ↑".to_string(),
                sub_label: "Up".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::DpadDown => ButtonPrompt {
                label: "D-Pad ↓".to_string(),
                sub_label: "Down".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::DpadLeft => ButtonPrompt {
                label: "D-Pad ←".to_string(),
                sub_label: "Left".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::DpadRight => ButtonPrompt {
                label: "D-Pad →".to_string(),
                sub_label: "Right".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::Select => ButtonPrompt {
                label: "—".to_string(),
                sub_label: "Minus (-)".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
            Ps2Button::Start => ButtonPrompt {
                label: "+".to_string(),
                sub_label: "Plus (+)".to_string(),
                badge_color: Color32::from_rgb(50, 50, 50),
                text_color: Color32::LIGHT_GRAY,
            },
        }
    }

    fn keyboard_prompt(&self) -> ButtonPrompt {
        match self {
            Ps2Button::Cross => ButtonPrompt {
                label: "Space".to_string(),
                sub_label: "Cross (PS2)".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::Circle => ButtonPrompt {
                label: "Shift".to_string(),
                sub_label: "Circle (PS2)".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::Square => ButtonPrompt {
                label: "Left Click".to_string(),
                sub_label: "Square (PS2)".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::Triangle => ButtonPrompt {
                label: "Right Click".to_string(),
                sub_label: "Triangle (PS2)".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::L1 => ButtonPrompt {
                label: "Q".to_string(),
                sub_label: "L1 (PS2)".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::R1 => ButtonPrompt {
                label: "E".to_string(),
                sub_label: "R1 (PS2)".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::L2 => ButtonPrompt {
                label: "R".to_string(),
                sub_label: "L2 (PS2)".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::R2 => ButtonPrompt {
                label: "Mouse Fwd".to_string(),
                sub_label: "R2 (PS2)".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::L3 => ButtonPrompt {
                label: "Z".to_string(),
                sub_label: "L3 (PS2)".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::R3 => ButtonPrompt {
                label: "X".to_string(),
                sub_label: "R3 (PS2)".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::DpadUp => ButtonPrompt {
                label: "Up / 1".to_string(),
                sub_label: "Dpad Up".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::DpadDown => ButtonPrompt {
                label: "Down / 3".to_string(),
                sub_label: "Dpad Down".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::DpadLeft => ButtonPrompt {
                label: "Left / 4".to_string(),
                sub_label: "Dpad Left".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::DpadRight => ButtonPrompt {
                label: "Right / 2".to_string(),
                sub_label: "Dpad Right".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::Select => ButtonPrompt {
                label: "Tab".to_string(),
                sub_label: "Select".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
            Ps2Button::Start => ButtonPrompt {
                label: "Esc".to_string(),
                sub_label: "Start".to_string(),
                badge_color: Color32::from_rgb(33, 33, 33),
                text_color: Color32::WHITE,
            },
        }
    }
}

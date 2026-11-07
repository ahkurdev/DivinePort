use crate::config::GlyphStyle;
use crate::gamepad::GamepadManager;
use crate::glyphs::Ps2Button;
use eframe::egui::{Color32, FontId, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use gilrs::Button;

pub fn render_interactive_gamepad(
    ui: &mut Ui,
    glyph_style: GlyphStyle,
    gamepad: &GamepadManager,
    size: Vec2,
) {
    let (response, painter) = ui.allocate_painter(size, Sense::hover());
    let rect = response.rect;

    // Background panel for controller
    painter.rect_filled(
        rect,
        12.0,
        Color32::from_rgb(15, 23, 42), // Slate 900
    );
    painter.rect_stroke(
        rect,
        12.0,
        Stroke::new(1.5_f32, Color32::from_rgb(51, 65, 85)), // Slate 700
    );

    let center = rect.center();
    let pad_w = rect.width() * 0.75;
    let pad_h = rect.height() * 0.65;
    let pad_rect = Rect::from_center_size(center, Vec2::new(pad_w, pad_h));

    // Draw Controller Body (Sleek aerodynamic gamepad shape)
    painter.rect_filled(
        pad_rect,
        36.0,
        Color32::from_rgb(30, 41, 59), // Slate 800
    );
    painter.rect_stroke(
        pad_rect,
        36.0,
        Stroke::new(2.0_f32, Color32::from_rgb(71, 85, 105)),
    );

    // Left Handle Grips
    let left_grip = Rect::from_min_size(
        Pos2::new(pad_rect.left() - 15.0, pad_rect.top() + 30.0),
        Vec2::new(50.0, pad_rect.height() * 0.75),
    );
    painter.rect_filled(left_grip, 20.0, Color32::from_rgb(24, 32, 47));

    // Right Handle Grips
    let right_grip = Rect::from_min_size(
        Pos2::new(pad_rect.right() - 35.0, pad_rect.top() + 30.0),
        Vec2::new(50.0, pad_rect.height() * 0.75),
    );
    painter.rect_filled(right_grip, 20.0, Color32::from_rgb(24, 32, 47));

    // -------------------------------------------------------------
    // Shoulder Buttons (L1, L2, R1, R2)
    // -------------------------------------------------------------
    let l1_rect = Rect::from_center_size(
        Pos2::new(pad_rect.left() + 70.0, pad_rect.top() - 8.0),
        Vec2::new(60.0, 16.0),
    );
    let l1_pressed = gamepad.is_button_pressed(Button::LeftTrigger);
    let l1_color = if l1_pressed {
        Color32::from_rgb(56, 189, 248) // Sky 400
    } else {
        Color32::from_rgb(51, 65, 85)
    };
    painter.rect_filled(l1_rect, 6.0, l1_color);
    let l1_prompt = Ps2Button::L1.prompt_info(glyph_style, gamepad.is_xbox_layout);
    painter.text(
        l1_rect.center(),
        eframe::egui::Align2::CENTER_CENTER,
        &l1_prompt.label,
        FontId::proportional(11.0),
        Color32::WHITE,
    );

    let r1_rect = Rect::from_center_size(
        Pos2::new(pad_rect.right() - 70.0, pad_rect.top() - 8.0),
        Vec2::new(60.0, 16.0),
    );
    let r1_pressed = gamepad.is_button_pressed(Button::RightTrigger);
    let r1_color = if r1_pressed {
        Color32::from_rgb(56, 189, 248)
    } else {
        Color32::from_rgb(51, 65, 85)
    };
    painter.rect_filled(r1_rect, 6.0, r1_color);
    let r1_prompt = Ps2Button::R1.prompt_info(glyph_style, gamepad.is_xbox_layout);
    painter.text(
        r1_rect.center(),
        eframe::egui::Align2::CENTER_CENTER,
        &r1_prompt.label,
        FontId::proportional(11.0),
        Color32::WHITE,
    );

    // -------------------------------------------------------------
    // D-PAD (Left Side)
    // -------------------------------------------------------------
    let dpad_center = Pos2::new(pad_rect.left() + 85.0, pad_rect.center().y - 20.0);
    let dpad_bg = Rect::from_center_size(dpad_center, Vec2::new(54.0, 54.0));
    painter.rect_filled(dpad_bg, 8.0, Color32::from_rgb(15, 23, 42));

    let draw_dpad_dir = |ui_pos: Pos2, btn: Button, icon: &str| {
        let is_down = gamepad.is_button_pressed(btn);
        let col = if is_down {
            Color32::from_rgb(56, 189, 248)
        } else {
            Color32::from_rgb(71, 85, 105)
        };
        painter.circle_filled(ui_pos, 9.0, col);
        painter.text(
            ui_pos,
            eframe::egui::Align2::CENTER_CENTER,
            icon,
            FontId::proportional(10.0),
            Color32::WHITE,
        );
    };

    draw_dpad_dir(Pos2::new(dpad_center.x, dpad_center.y - 17.0), Button::DPadUp, "▲");
    draw_dpad_dir(Pos2::new(dpad_center.x, dpad_center.y + 17.0), Button::DPadDown, "▼");
    draw_dpad_dir(Pos2::new(dpad_center.x - 17.0, dpad_center.y), Button::DPadLeft, "◀");
    draw_dpad_dir(Pos2::new(dpad_center.x + 17.0, dpad_center.y), Button::DPadRight, "▶");

    // -------------------------------------------------------------
    // ACTION BUTTONS (Right Side Diamond - The Glyphs!)
    // -------------------------------------------------------------
    let action_center = Pos2::new(pad_rect.right() - 85.0, pad_rect.center().y - 20.0);
    let action_bg = Rect::from_center_size(action_center, Vec2::new(60.0, 60.0));
    painter.circle_filled(action_bg.center(), 30.0, Color32::from_rgb(15, 23, 42));

    let draw_glyph_btn = |ui_pos: Pos2, ps_btn: Ps2Button, gilrs_btn: Button| {
        let prompt = ps_btn.prompt_info(glyph_style, gamepad.is_xbox_layout);
        let is_pressed = gamepad.is_button_pressed(gilrs_btn);

        let (fill_col, stroke_col, radius) = if is_pressed {
            (prompt.badge_color, Color32::WHITE, 14.0)
        } else {
            (
                Color32::from_rgb(26, 36, 52),
                prompt.badge_color,
                12.0,
            )
        };

        painter.circle_filled(ui_pos, radius, fill_col);
        painter.circle_stroke(ui_pos, radius, Stroke::new(2.0_f32, stroke_col));
        painter.text(
            ui_pos,
            eframe::egui::Align2::CENTER_CENTER,
            &prompt.label,
            FontId::proportional(12.0),
            if is_pressed { Color32::WHITE } else { prompt.badge_color },
        );
    };

    // South Button: Cross (PS) / A (Xbox) / B (Nintendo)
    draw_glyph_btn(
        Pos2::new(action_center.x, action_center.y + 19.0),
        Ps2Button::Cross,
        Button::South,
    );
    // East Button: Circle (PS) / B (Xbox) / A (Nintendo)
    draw_glyph_btn(
        Pos2::new(action_center.x + 19.0, action_center.y),
        Ps2Button::Circle,
        Button::East,
    );
    // West Button: Square (PS) / X (Xbox) / Y (Nintendo)
    draw_glyph_btn(
        Pos2::new(action_center.x - 19.0, action_center.y),
        Ps2Button::Square,
        Button::West,
    );
    // North Button: Triangle (PS) / Y (Xbox) / X (Nintendo)
    draw_glyph_btn(
        Pos2::new(action_center.x, action_center.y - 19.0),
        Ps2Button::Triangle,
        Button::North,
    );

    // -------------------------------------------------------------
    // ANALOG STICKS (Interactive Live Movement)
    // -------------------------------------------------------------
    let left_stick_base = Pos2::new(pad_rect.center().x - 55.0, pad_rect.center().y + 35.0);
    painter.circle_filled(left_stick_base, 22.0, Color32::from_rgb(15, 23, 42));
    painter.circle_stroke(left_stick_base, 22.0, Stroke::new(1.5_f32, Color32::from_rgb(51, 65, 85)));

    // Stick offset based on real gamepad input
    let left_offset = Vec2::new(gamepad.left_stick.0 * 10.0, -gamepad.left_stick.1 * 10.0);
    let left_knob = left_stick_base + left_offset;
    let l3_pressed = gamepad.is_button_pressed(Button::LeftThumb);
    let left_knob_col = if l3_pressed {
        Color32::from_rgb(56, 189, 248)
    } else {
        Color32::from_rgb(71, 85, 105)
    };
    painter.circle_filled(left_knob, 14.0, left_knob_col);
    painter.text(left_knob, eframe::egui::Align2::CENTER_CENTER, "L3", FontId::proportional(9.0), Color32::WHITE);

    let right_stick_base = Pos2::new(pad_rect.center().x + 55.0, pad_rect.center().y + 35.0);
    painter.circle_filled(right_stick_base, 22.0, Color32::from_rgb(15, 23, 42));
    painter.circle_stroke(right_stick_base, 22.0, Stroke::new(1.5_f32, Color32::from_rgb(51, 65, 85)));

    let right_offset = Vec2::new(gamepad.right_stick.0 * 10.0, -gamepad.right_stick.1 * 10.0);
    let right_knob = right_stick_base + right_offset;
    let r3_pressed = gamepad.is_button_pressed(Button::RightThumb);
    let right_knob_col = if r3_pressed {
        Color32::from_rgb(56, 189, 248)
    } else {
        Color32::from_rgb(71, 85, 105)
    };
    painter.circle_filled(right_knob, 14.0, right_knob_col);
    painter.text(right_knob, eframe::egui::Align2::CENTER_CENTER, "R3", FontId::proportional(9.0), Color32::WHITE);

    // -------------------------------------------------------------
    // Center Buttons (Select / Back & Start / Menu)
    // -------------------------------------------------------------
    let select_pos = Pos2::new(pad_rect.center().x - 25.0, pad_rect.center().y - 15.0);
    let select_down = gamepad.is_button_pressed(Button::Select);
    painter.rect_filled(
        Rect::from_center_size(select_pos, Vec2::new(16.0, 8.0)),
        3.0,
        if select_down { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(51, 65, 85) },
    );

    let start_pos = Pos2::new(pad_rect.center().x + 25.0, pad_rect.center().y - 15.0);
    let start_down = gamepad.is_button_pressed(Button::Start);
    painter.rect_filled(
        Rect::from_center_size(start_pos, Vec2::new(16.0, 8.0)),
        3.0,
        if start_down { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(51, 65, 85) },
    );

    // Center Logo Badge
    painter.text(
        Pos2::new(pad_rect.center().x, pad_rect.center().y - 30.0),
        eframe::egui::Align2::CENTER_CENTER,
        "PS2 RECOMP",
        FontId::proportional(11.0),
        Color32::from_rgb(148, 163, 184),
    );
}

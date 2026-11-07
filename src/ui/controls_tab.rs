use crate::config::{GameRuntimeConfig, GlyphStyle, LauncherSettings};
use crate::gamepad::GamepadManager;
use crate::glyphs::{ButtonPrompt, Ps2Button};
use crate::ui::controller_canvas::render_interactive_gamepad;
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use gilrs::Button;

pub fn render_controls_tab(
    ui: &mut egui::Ui,
    settings: &mut LauncherSettings,
    runtime_config: &mut GameRuntimeConfig,
    gamepad: &mut GamepadManager,
) {
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.heading(RichText::new("KONTROL & GLYPH DUALSHOCK 2 (PAD)").strong().size(18.0).color(Color32::from_rgb(241, 245, 249)));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if gamepad.active_gamepad.is_some() {
                ui.colored_label(Color32::from_rgb(52, 211, 153), format!("[Terhubung: {}]", gamepad.active_name));
            } else {
                ui.colored_label(Color32::from_rgb(248, 113, 113), "[Controller Terputus]");
            }
        });
    });
    ui.label(RichText::new("Pilih simbol tombol visual (Glyph) di layar. Sinyal tombol ke engine game tetap 100% PS2 DualShock 2 asli.").color(Color32::from_rgb(139, 148, 158)));
    ui.add_space(8.0);

    // -----------------------------------------------------------------
    // 1. Interactive 2D Gamepad Canvas (Real-Time Visualizer)
    // -----------------------------------------------------------------
    ui.vertical_centered(|ui| {
        render_interactive_gamepad(ui, settings.glyph_style, gamepad, Vec2::new(600.0, 230.0));
    });

    ui.add_space(10.0);

    // -------------------------------------------------------------
    // 2. Glyph Selector Cards (PS / Xbox / Nintendo / Keyboard)
    // -------------------------------------------------------------
    ui.heading(RichText::new("Pilih Tampilan Ikon Tombol (Glyph Prompt)").size(15.0).color(Color32::from_rgb(226, 232, 240)));
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let styles = [
            (GlyphStyle::Auto, "Auto (Deteksi Hardware)", "Deteksi otomatis berdasarkan hardware controller yang terhubung"),
            (GlyphStyle::PlayStation, "PlayStation [X, O, Kotak, Segitiga]", "Simbol klasik PlayStation dengan warna biru, merah, pink, hijau"),
            (GlyphStyle::Xbox, "Xbox [A, B, X, Y]", "Simbol gaya Xbox XInput dengan tombol A hijau, B merah, X biru, Y kuning"),
            (GlyphStyle::Nintendo, "Nintendo [B, A, Y, X]", "Simbol tata letak Switch Pro Controller / Nintendo"),
            (GlyphStyle::Keyboard, "Keyboard & Mouse", "Tampilkan shortcut keyboard (Space, Shift, LMB, RMB, dll.)"),
        ];

        for (style, title, _desc) in styles {
            let is_selected = settings.glyph_style == style;
            let (bg, border) = if is_selected {
                (Color32::from_rgb(30, 58, 95), Color32::from_rgb(56, 189, 248))
            } else {
                (Color32::from_rgb(18, 24, 38), Color32::from_rgb(48, 54, 61))
            };

            egui::Frame::none()
                .fill(bg)
                .stroke(Stroke::new(1.2_f32, border))
                .inner_margin(8.0)
                .rounding(4.0)
                .show(ui, |ui| {
                    if ui.selectable_label(is_selected, RichText::new(title).strong()).clicked() {
                        settings.glyph_style = style;
                        runtime_config.glyph_style = style;
                    }
                });
        }
    });

    ui.add_space(12.0);

    // -------------------------------------------------------------
    // 3. Button Mapping & Diagnostic Grid
    // -------------------------------------------------------------
    ui.heading(RichText::new("Tabel Pemetaan & Diagnostik Input").size(15.0).color(Color32::from_rgb(226, 232, 240)));
    ui.add_space(4.0);

    egui::Frame::none()
        .fill(Color32::from_rgb(18, 24, 38))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 54, 61)))
        .inner_margin(10.0)
        .rounding(6.0)
        .show(ui, |ui| {
            egui::Grid::new("modern_button_glyph_grid")
                .striped(true)
                .spacing([24.0, 6.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Target PS2").strong().color(Color32::from_rgb(139, 148, 158)));
                    ui.label(RichText::new("Ikon Tampilan Layar").strong().color(Color32::from_rgb(139, 148, 158)));
                    ui.label(RichText::new("Deskripsi Fungsi").strong().color(Color32::from_rgb(139, 148, 158)));
                    ui.label(RichText::new("Status Tombol Fisik").strong().color(Color32::from_rgb(139, 148, 158)));
                    ui.end_row();

                    let buttons = Ps2Button::ALL;
                    let is_xbox = gamepad.active_name.contains("Xbox") || gamepad.active_name.contains("XInput");
                    for btn in buttons {
                        ui.label(RichText::new(btn.canonical_ps_name()).color(Color32::from_rgb(241, 245, 249)));

                        let prompt: ButtonPrompt = btn.prompt_info(settings.glyph_style, is_xbox);
                        ui.label(
                            RichText::new(format!("[ {} ]", prompt.label))
                                .strong()
                                .color(prompt.badge_color),
                        );

                        ui.label(RichText::new(&prompt.sub_label).color(Color32::from_rgb(148, 163, 184)));

                        let gilrs_btn = match btn {
                            Ps2Button::Cross => Button::South,
                            Ps2Button::Circle => Button::East,
                            Ps2Button::Square => Button::West,
                            Ps2Button::Triangle => Button::North,
                            Ps2Button::L1 => Button::LeftTrigger,
                            Ps2Button::R1 => Button::RightTrigger,
                            Ps2Button::L2 => Button::LeftTrigger2,
                            Ps2Button::R2 => Button::RightTrigger2,
                            Ps2Button::L3 => Button::LeftThumb,
                            Ps2Button::R3 => Button::RightThumb,
                            Ps2Button::Select => Button::Select,
                            Ps2Button::Start => Button::Start,
                            Ps2Button::DpadUp => Button::DPadUp,
                            Ps2Button::DpadDown => Button::DPadDown,
                            Ps2Button::DpadLeft => Button::DPadLeft,
                            Ps2Button::DpadRight => Button::DPadRight,
                        };

                        let is_pressed = gamepad.is_button_pressed(gilrs_btn);
                        if is_pressed {
                            ui.colored_label(Color32::from_rgb(52, 211, 153), "[ DITEKAN ]");
                        } else {
                            ui.colored_label(Color32::from_rgb(100, 116, 139), "-");
                        }

                        ui.end_row();
                    }
                });
        });
}

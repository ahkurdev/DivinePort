use crate::config::{GameRuntimeConfig, LauncherSettings};
use eframe::egui::{self, Color32, RichText, Stroke};
use std::path::Path;

pub fn render_settings_tab(
    ui: &mut egui::Ui,
    settings: &mut LauncherSettings,
    runtime_config: &mut GameRuntimeConfig,
) {
    ui.heading(RichText::new("PENGATURAN UMUM & KONFIGURASI SISTEM").strong().size(18.0));
    ui.label(RichText::new("Konfigurasi aplikasi peluncur, path folder kompiler, dan preferensi antarmuka.").color(Color32::from_rgb(139, 148, 158)));
    ui.add_space(8.0);

    egui::Frame::none()
        .fill(Color32::from_rgb(18, 24, 38))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 54, 61)))
        .inner_margin(12.0)
        .rounding(6.0)
        .show(ui, |ui| {
            ui.heading(RichText::new("Tampilan & UI Scale").size(15.0));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("GUI Scale:");
                let scales = [("auto", "Auto (Fit Screen)"), ("1.0", "1x"), ("1.5", "1.5x")];
                for (val, desc) in scales {
                    if ui.selectable_label(settings.gui_scale == val, desc).clicked() {
                        settings.gui_scale = val.to_string();
                    }
                }
            });

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Bahasa Antarmuka (Launcher Language):");
                let langs = [("id", "Bahasa Indonesia"), ("en", "English"), ("zh", "Simplified Chinese")];
                for (val, desc) in langs {
                    if ui.selectable_label(settings.language == val, desc).clicked() {
                        settings.language = val.to_string();
                    }
                }
            });

            ui.add_space(6.0);
            ui.checkbox(&mut settings.discord_rpc, "Aktifkan Discord Rich Presence (Status Game & Waktu Bermain)");
        });

    ui.add_space(10.0);

    egui::Frame::none()
        .fill(Color32::from_rgb(18, 24, 38))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 54, 61)))
        .inner_margin(12.0)
        .rounding(6.0)
        .show(ui, |ui| {
            ui.heading(RichText::new("Direktori Source ps2-native-recomp").size(15.0));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                let mut path_str = settings.recomp_dir.to_string_lossy().to_string();
                if ui.text_edit_singleline(&mut path_str).changed() {
                    settings.recomp_dir = std::path::PathBuf::from(path_str);
                }

                if ui.button("Pilih Folder...").clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        settings.recomp_dir = folder;
                    }
                }
            });
        });

    ui.add_space(12.0);

    if ui.button(RichText::new("[ Simpan Pengaturan (Save settings.json & spartanport.ini) ]").strong().size(14.0)).clicked() {
        let settings_path = Path::new("settings.json");
        let ini_path = Path::new("spartanport.ini");
        let _ = settings.save(settings_path);
        let _ = runtime_config.save_to_ini(ini_path);
    }
}

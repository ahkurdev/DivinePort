use crate::config::{GameRuntimeConfig, LauncherSettings};
use crate::native::NativeManager;
use eframe::egui::{self, Color32, ProgressBar, RichText, Stroke};

pub fn render_library_tab(
    ui: &mut egui::Ui,
    settings: &mut LauncherSettings,
    runtime_config: &mut GameRuntimeConfig,
    native_mgr: &mut NativeManager,
    search_query: &str,
    table_view_mode: bool,
    selected_index: &mut usize,
) -> Option<(String, std::process::Child)> {
    ui.add_space(2.0);

    // If currently building a game, show Live Terminal Console
    if native_mgr.is_building {
        egui::Frame::none()
            .fill(Color32::from_rgb(10, 15, 26))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(56, 189, 248)))
            .inner_margin(12.0)
            .rounding(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new(format!(" Status: {}", native_mgr.current_stage)).strong().color(Color32::from_rgb(56, 189, 248)));
                });

                ui.add_space(4.0);
                ui.add(ProgressBar::new(native_mgr.build_progress).animate(true));

                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .max_height(100.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in &native_mgr.log_lines {
                            let col = if line.contains("[STDERR]") {
                                Color32::from_rgb(248, 113, 113)
                            } else if line.contains("===>") {
                                Color32::from_rgb(52, 211, 153)
                            } else {
                                Color32::from_rgb(148, 163, 184)
                            };
                            ui.label(RichText::new(line).monospace().size(11.0).color(col));
                        }
                    });
            });

        ui.add_space(8.0);
    }

    let mut action_launch = None;
    let mut action_build = None;

    // Filter games by search query
    let filtered_games: Vec<_> = native_mgr
        .games
        .iter()
        .filter(|g| {
            if search_query.is_empty() {
                true
            } else {
                let q = search_query.to_lowercase();
                g.name.to_lowercase().contains(&q) || g.source.to_lowercase().contains(&q)
            }
        })
        .cloned()
        .collect();

    if filtered_games.is_empty() {
        egui::Frame::none()
            .fill(Color32::from_rgb(15, 23, 42))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(51, 65, 85)))
            .inner_margin(24.0)
            .rounding(6.0)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("Tidak ada disc image PS2 ditemukan.").italics().size(15.0).color(Color32::from_rgb(203, 213, 225)));
                    ui.add_space(4.0);
                    ui.label(RichText::new("Letakkan file ISO/BIN/CHD di folder games/ atau klik [+] Tambah Disc ISO di toolbar.").color(Color32::from_rgb(148, 163, 184)));
                });
            });
    } else if table_view_mode {
        // -------------------------------------------------------------
        // PCSX2 DETAILED MULTI-COLUMN TABLE VIEW
        // -------------------------------------------------------------
        egui::Frame::none()
            .fill(Color32::from_rgb(18, 24, 38))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 54, 61)))
            .inner_margin(6.0)
            .rounding(4.0)
            .show(ui, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    egui::Grid::new("pcsx2_game_table")
                        .striped(true)
                        .min_col_width(90.0)
                        .spacing([18.0, 10.0])
                        .show(ui, |ui| {
                            // Table Headers (Iconic PCSX2 Table)
                            ui.label(RichText::new("Judul Game").strong().color(Color32::from_rgb(148, 163, 184)));
                            ui.label(RichText::new("Serial / ELF ID").strong().color(Color32::from_rgb(148, 163, 184)));
                            ui.label(RichText::new("Region").strong().color(Color32::from_rgb(148, 163, 184)));
                            ui.label(RichText::new("Format").strong().color(Color32::from_rgb(148, 163, 184)));
                            ui.label(RichText::new("Status Biner Native").strong().color(Color32::from_rgb(148, 163, 184)));
                            ui.label(RichText::new("Waktu Main").strong().color(Color32::from_rgb(148, 163, 184)));
                            ui.label(RichText::new("Widescreen 16:9").strong().color(Color32::from_rgb(148, 163, 184)));
                            ui.label(RichText::new("Aksi").strong().color(Color32::from_rgb(148, 163, 184)));
                            ui.end_row();

                            for (idx, game) in filtered_games.iter().enumerate() {
                                let is_selected = *selected_index == idx;
                                // 1. Title
                                let display_title = game.name.replace('-', " ").to_uppercase();
                                let title_text = if is_selected {
                                    RichText::new(format!("> {}", display_title)).strong().color(Color32::from_rgb(56, 189, 248))
                                } else {
                                    RichText::new(display_title).strong().color(Color32::WHITE)
                                };
                                if ui.selectable_label(is_selected, title_text).clicked() {
                                    *selected_index = idx;
                                }

                                // 2. Serial / ELF ID
                                let serial_display = if game.elf.contains("SLUS_209.64") {
                                    "SLUS-20964"
                                } else if game.elf.contains("SLES_544.90") {
                                    "SLES-54490"
                                } else if !game.elf.is_empty() {
                                    &game.elf
                                } else {
                                    "PS2-DVD"
                                };
                                ui.label(RichText::new(serial_display).monospace().color(Color32::from_rgb(203, 213, 225)));

                                // 3. Region
                                let region = if game.source.contains("USA") || game.elf.contains("SLUS") {
                                    "NTSC-U"
                                } else if game.source.contains("Europe") || game.elf.contains("SLES") {
                                    "PAL-E"
                                } else {
                                    "UNIVERSAL"
                                };
                                ui.label(RichText::new(region).color(Color32::from_rgb(148, 163, 184)));

                                // 4. Format
                                let format_label = if game.source.ends_with(".iso") || game.source.ends_with(".ISO") {
                                    "ISO Image"
                                } else if game.source.ends_with(".bin") {
                                    "BIN Image"
                                } else {
                                    "DVD-ROM"
                                };
                                ui.label(RichText::new(format_label).color(Color32::from_rgb(148, 163, 184)));

                                // 5. Status
                                if game.is_ready {
                                    ui.colored_label(
                                        Color32::from_rgb(52, 211, 153),
                                        RichText::new("[ SIAP (AOT x86_64) ]").strong().small(),
                                    );
                                } else {
                                    ui.colored_label(
                                        Color32::from_rgb(251, 191, 36),
                                        RichText::new("[ BELUM DIKOMPILASI ]").small(),
                                    );
                                }

                                // 6. Waktu Main
                                let hours = game.playtime_seconds as f64 / 3600.0;
                                let playtime_str = if game.playtime_seconds == 0 {
                                    "-".to_string()
                                } else if hours < 0.1 {
                                    format!("{} dtk", game.playtime_seconds)
                                } else {
                                    format!("{:.1} jam", hours)
                                };
                                ui.label(RichText::new(playtime_str).small().color(Color32::from_rgb(203, 213, 225)));

                                // 7. Widescreen 16:9
                                if !game.widescreen.is_empty() {
                                    ui.colored_label(
                                        Color32::from_rgb(56, 189, 248),
                                        RichText::new("Patch Aktif (16:9)").small(),
                                    );
                                } else {
                                    ui.label(RichText::new("Standar (4:3)").color(Color32::from_rgb(100, 116, 139)).small());
                                }

                                // 7. Action Button
                                ui.horizontal(|ui| {
                                    if game.is_ready {
                                        if ui.button(RichText::new("JALANKAN").strong().color(Color32::from_rgb(52, 211, 153))).clicked() {
                                            action_launch = Some((
                                                game.clone(),
                                                settings.glyph_style.as_str().to_string(),
                                            ));
                                        }
                                    } else if ui.button(RichText::new("KOMPILASI").strong().color(Color32::from_rgb(251, 191, 36))).clicked() {
                                        let iso_path = std::path::PathBuf::from(&game.source);
                                        action_build = Some((
                                            game.name.clone(),
                                            iso_path,
                                            runtime_config.widescreen_16_9,
                                        ));
                                    }
                                });

                                ui.end_row();
                            }
                        });
                });
            });
    } else {
        // -------------------------------------------------------------
        // SPARTANPORT CARD GRID VIEW
        // -------------------------------------------------------------
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (idx, game) in filtered_games.iter().enumerate() {
                let is_selected = *selected_index == idx;
                let bg_color = if is_selected {
                    Color32::from_rgb(26, 36, 56)
                } else {
                    Color32::from_rgb(18, 24, 38)
                };
                let stroke_color = if is_selected {
                    Color32::from_rgb(56, 189, 248)
                } else {
                    Color32::from_rgb(48, 54, 61)
                };

                egui::Frame::none()
                    .fill(bg_color)
                    .stroke(Stroke::new(if is_selected { 2.0_f32 } else { 1.0_f32 }, stroke_color))
                    .inner_margin(12.0)
                    .rounding(6.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let cover_path = format!("covers/{}.jpg", game.name);
                            let mut loaded_cover = false;
                            if std::path::Path::new(&cover_path).exists() {
                                if let Ok(bytes) = std::fs::read(&cover_path) {
                                    if let Ok(img) = image::load_from_memory(&bytes) {
                                        let rgba = img.to_rgba8();
                                        let size = [rgba.width() as usize, rgba.height() as usize];
                                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                                        let texture = ui.ctx().load_texture(&cover_path, color_image, Default::default());
                                        ui.image((texture.id(), egui::Vec2::new(54.0, 76.0)));
                                        loaded_cover = true;
                                    }
                                }
                            }

                            if !loaded_cover {
                                let (badge_rect, _) = ui.allocate_exact_size(eframe::egui::Vec2::new(54.0, 76.0), egui::Sense::hover());
                                ui.painter().rect_filled(badge_rect, 6.0, Color32::from_rgb(33, 38, 45));
                                ui.painter().text(
                                    badge_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "PS2",
                                    egui::FontId::monospace(14.0),
                                    Color32::from_rgb(56, 189, 248),
                                );
                            }

                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                let display_title = game.name.replace('-', " ").to_uppercase();
                                ui.label(RichText::new(display_title).strong().size(16.0).color(Color32::WHITE));
                                ui.label(RichText::new(format!("Path: {}", game.source)).small().color(Color32::from_rgb(139, 148, 158)));

                                if game.is_ready {
                                    ui.colored_label(Color32::from_rgb(52, 211, 153), RichText::new("Status: Native Binary Siap Dimainkan").strong().small());
                                } else {
                                    ui.colored_label(Color32::from_rgb(251, 191, 36), RichText::new("Status: Belum Dikompilasi").small());
                                }

                                let hours = game.playtime_seconds as f64 / 3600.0;
                                let pt_str = if game.playtime_seconds == 0 {
                                    "Belum pernah dimainkan".to_string()
                                } else {
                                    format!("Dimainkan: {:.1} Jam | Terakhir: {}", hours, if game.last_played.is_empty() { "-" } else { &game.last_played })
                                };
                                ui.label(RichText::new(pt_str).small().color(Color32::from_rgb(148, 163, 184)));
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if game.is_ready {
                                    if ui.button(RichText::new(" [ JALANKAN ] ").strong().size(14.0).color(Color32::WHITE)).clicked() {
                                        action_launch = Some((
                                            game.clone(),
                                            settings.glyph_style.as_str().to_string(),
                                        ));
                                    }
                                } else if ui.button(RichText::new(" [ KOMPILASI ] ").strong().size(13.0)).clicked() {
                                    let iso_path = std::path::PathBuf::from(&game.source);
                                    action_build = Some((
                                        game.name.clone(),
                                        iso_path,
                                        runtime_config.widescreen_16_9,
                                    ));
                                }
                            });
                        });
                    });
                ui.add_space(8.0);
            }
        });
    }

    let mut launched = None;
    if let Some((game, glyph)) = action_launch {
        if let Ok(child) = native_mgr.launch_native_game(&game, &glyph, runtime_config) {
            launched = Some((game.name.clone(), child));
        }
    }

    if let Some((name, iso, wide)) = action_build {
        native_mgr.enqueue_build(&name, &iso, wide);
    }

    launched
}

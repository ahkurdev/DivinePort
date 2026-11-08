mod config;
mod gamepad;
mod games;
mod glyphs;
mod native;
mod ui;

use config::{GameRuntimeConfig, LauncherSettings};
use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use gamepad::GamepadManager;
use native::NativeManager;
use std::path::{Path, PathBuf};
use ui::ActiveTab;

struct App {
    settings: LauncherSettings,
    runtime_config: GameRuntimeConfig,
    active_tab: ActiveTab,
    gamepad: GamepadManager,
    native_mgr: NativeManager,
    search_query: String,
    table_view_mode: bool, // true: PCSX2 detailed list table, false: SpartanPort cards
    show_about_dialog: bool,
    show_hotkey_dialog: bool,
    logo_texture: Option<egui::TextureHandle>,
    selected_game_index: usize,
    running_game: Option<(String, std::time::Instant, std::process::Child)>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings_path = Path::new("settings.json");
        let settings = LauncherSettings::load_or_default(settings_path);
        let runtime_config = GameRuntimeConfig::load_or_default(Path::new("runtime_config.json"));
        let gamepad = GamepadManager::new();

        let recomp_root = if Path::new("../ps2-native-recomp").exists() {
            PathBuf::from("../ps2-native-recomp")
        } else if Path::new("ps2-native-recomp").exists() {
            PathBuf::from("ps2-native-recomp")
        } else {
            settings.recomp_dir.clone()
        };

        let native_mgr = NativeManager::new(recomp_root);
        fetch_missing_box_arts(&native_mgr.games);

        let logo_texture = if let Ok(img) = image::load_from_memory(include_bytes!("../assets/logo.jpg")) {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let pixels = rgba.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            Some(cc.egui_ctx.load_texture("divineport_logo", color_image, Default::default()))
        } else {
            None
        };

        Self {
            settings,
            runtime_config,
            active_tab: ActiveTab::Library,
            gamepad,
            native_mgr,
            search_query: String::new(),
            table_view_mode: true,
            show_about_dialog: false,
            show_hotkey_dialog: false,
            logo_texture,
            selected_game_index: 0,
            running_game: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let prev_settings = self.settings.clone();
        let prev_runtime = self.runtime_config.clone();

        self.gamepad.update();
        self.native_mgr.update_build_logs();

        // -------------------------------------------------------------
        // RUNNING GAME LIFECYCLE (PLAYTIME & AUTO RESTORE WINDOW)
        // -------------------------------------------------------------
        if let Some((ref game_name, start_time, ref mut child)) = self.running_game {
            match child.try_wait() {
                Ok(Some(_status)) => {
                    let elapsed = start_time.elapsed().as_secs();
                    let g_name = game_name.clone();
                    self.native_mgr.update_playtime(&g_name, elapsed);
                    self.running_game = None;
                    // Restore and bring launcher window to front
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                Ok(None) => {
                    // Still running, wake up every 500ms to poll
                    ctx.request_repaint_after(std::time::Duration::from_millis(500));
                }
                Err(_) => {
                    self.running_game = None;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
            }
        }

        // -------------------------------------------------------------
        // GAMEPAD MENU NAVIGATION (D-Pad, Stick, Face Buttons)
        // -------------------------------------------------------------
        let total_games = self.native_mgr.games.len();
        if total_games > 0 {
            if self.selected_game_index >= total_games {
                self.selected_game_index = total_games - 1;
            }

            if self.gamepad.is_button_just_pressed(gilrs::Button::DPadUp) || self.gamepad.check_stick_nav_up() {
                if self.selected_game_index > 0 {
                    self.selected_game_index -= 1;
                }
            }
            if self.gamepad.is_button_just_pressed(gilrs::Button::DPadDown) || self.gamepad.check_stick_nav_down() {
                if self.selected_game_index + 1 < total_games {
                    self.selected_game_index += 1;
                }
            }
        }

        // Tab Switching via L1 / R1 or Stick Left / Right
        if self.gamepad.is_button_just_pressed(gilrs::Button::LeftTrigger)
            || self.gamepad.is_button_just_pressed(gilrs::Button::LeftTrigger2)
            || self.gamepad.check_stick_nav_left()
        {
            self.active_tab = match self.active_tab {
                ActiveTab::Library => ActiveTab::Logs,
                ActiveTab::Graphics => ActiveTab::Library,
                ActiveTab::Controls => ActiveTab::Graphics,
                ActiveTab::Settings => ActiveTab::Controls,
                ActiveTab::Logs => ActiveTab::Settings,
            };
        }
        if self.gamepad.is_button_just_pressed(gilrs::Button::RightTrigger)
            || self.gamepad.is_button_just_pressed(gilrs::Button::RightTrigger2)
            || self.gamepad.check_stick_nav_right()
        {
            self.active_tab = match self.active_tab {
                ActiveTab::Library => ActiveTab::Graphics,
                ActiveTab::Graphics => ActiveTab::Controls,
                ActiveTab::Controls => ActiveTab::Settings,
                ActiveTab::Settings => ActiveTab::Logs,
                ActiveTab::Logs => ActiveTab::Library,
            };
        }

        // Toggle Table / Card view with West button (X on Xbox, Square on PS)
        if self.gamepad.is_button_just_pressed(gilrs::Button::West) {
            self.table_view_mode = !self.table_view_mode;
        }

        // Launch / Recompile selected game with South button (A on Xbox, Cross on PS) or Start
        let to_action = if self.active_tab == ActiveTab::Library && !self.show_about_dialog && !self.show_hotkey_dialog {
            if self.gamepad.is_button_just_pressed(gilrs::Button::South)
                || self.gamepad.is_button_just_pressed(gilrs::Button::Start)
            {
                self.native_mgr.games.get(self.selected_game_index).cloned()
            } else {
                None
            }
        } else {
            None
        };

        if let Some(game) = to_action {
            if game.is_ready {
                if let Ok(child) = self.native_mgr.launch_native_game(
                    &game,
                    self.settings.glyph_style.as_str(),
                    &self.runtime_config,
                ) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    self.running_game = Some((game.name.clone(), std::time::Instant::now(), child));
                }
            } else {
                let iso_path = PathBuf::from(&game.source);
                self.native_mgr.enqueue_build(
                    &game.name,
                    &iso_path,
                    self.runtime_config.widescreen_16_9,
                );
            }
        }

        // Toggle Hotkeys help dialog with Select / Back button
        if self.gamepad.is_button_just_pressed(gilrs::Button::Select) {
            self.show_hotkey_dialog = !self.show_hotkey_dialog;
        }

        // SpartanPort Dark Slate Theme
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(13, 17, 23); // #0d1117 (GitHub / SpartanPort graphite)
        visuals.window_fill = Color32::from_rgb(22, 27, 34); // #161b22
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(22, 27, 34);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(33, 38, 45);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(48, 54, 61);
        visuals.widgets.active.bg_fill = Color32::from_rgb(31, 111, 235);
        ctx.set_visuals(visuals);

        // -------------------------------------------------------------
        // 1. PCSX2 TOP MENUBAR (System, CDVD, Config, View, Tools, Help)
        // -------------------------------------------------------------
        egui::TopBottomPanel::top("pcsx2_menubar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("System", |ui| {
                    if ui.button("Start Disc (Fast)...").clicked() {
                        self.active_tab = ActiveTab::Library;
                        ui.close_menu();
                    }
                    if ui.button("Open Disc ISO...").clicked() {
                        if let Some(file) = rfd::FileDialog::new()
                            .add_filter("PS2 Disc Image", &["iso", "bin", "chd"])
                            .pick_file()
                        {
                            let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("game");
                            self.native_mgr.start_native_build(stem, &file, self.runtime_config.widescreen_16_9);
                            self.active_tab = ActiveTab::Library;
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit Launcher").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("CDVD", |ui| {
                    if ui.button("Scan Game Directory (games/)...").clicked() {
                        self.native_mgr.scan_games();
                        ui.close_menu();
                    }
                    if ui.button("Open games/ Folder...").clicked() {
                        let _ = std::process::Command::new("explorer.exe").arg("games").spawn();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Config", |ui| {
                    if ui.button("Graphics (GS & Upscaling)...").clicked() {
                        self.active_tab = ActiveTab::Graphics;
                        ui.close_menu();
                    }
                    if ui.button("Controllers (PAD & Glyphs)...").clicked() {
                        self.active_tab = ActiveTab::Controls;
                        ui.close_menu();
                    }
                    if ui.button("General Settings...").clicked() {
                        self.active_tab = ActiveTab::Settings;
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.selectable_label(self.table_view_mode, "Game List (PCSX2 Table)").clicked() {
                        self.table_view_mode = true;
                        ui.close_menu();
                    }
                    if ui.selectable_label(!self.table_view_mode, "Game Grid (SpartanPort Cards)").clicked() {
                        self.table_view_mode = false;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.checkbox(&mut self.runtime_config.fullscreen, "Fullscreen Mode (F11)").clicked() {
                        ui.close_menu();
                    }
                });

                ui.menu_button("Tools", |ui| {
                    if ui.button("Log Console (AOT Build Terminal)...").clicked() {
                        self.active_tab = ActiveTab::Logs;
                        ui.close_menu();
                    }
                    if ui.button("Clear Log History").clicked() {
                        self.native_mgr.log_lines.clear();
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Check for Updates (GitHub)...").clicked() {
                        let _ = std::process::Command::new("cmd")
                            .args(["/c", "start", "https://github.com/ahkurdev/DivinePort/releases"])
                            .spawn();
                        ui.close_menu();
                    }
                    if ui.button("Hotkeys & Shortcuts...").clicked() {
                        self.show_hotkey_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("About DivinePort...").clicked() {
                        self.show_about_dialog = true;
                        ui.close_menu();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.gamepad.active_gamepad.is_some() {
                        ui.colored_label(
                            Color32::from_rgb(52, 211, 153),
                            format!("[PAD 1: {}]", self.gamepad.active_name),
                        );
                    } else {
                        ui.colored_label(Color32::from_rgb(148, 163, 184), "[PAD 1: Disconnected]");
                    }
                });
            });
        });

        // -------------------------------------------------------------
        // 2. PCSX2 TOOLBAR & SPARTANPORT TAB BAR (ZERO EMOJIS)
        // -------------------------------------------------------------
        egui::TopBottomPanel::top("pcsx2_toolbar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                // Brand logo + label
                if let Some(ref logo) = self.logo_texture {
                    ui.image((logo.id(), Vec2::new(26.0, 26.0)));
                    ui.add_space(2.0);
                }

                ui.label(
                    RichText::new("DIVINEPORT")
                        .strong()
                        .size(15.0)
                        .color(Color32::from_rgb(56, 189, 248)),
                );
                ui.label(
                    RichText::new("| DivinePort Native Engine (SpartanPort x PCSX2 Unified)")
                        .size(11.0)
                        .color(Color32::from_rgb(139, 148, 158)),
                );

                ui.separator();

                // Quick buttons
                if ui.button(RichText::new("[+] Tambah Disc ISO").strong()).clicked() {
                    if let Some(file) = rfd::FileDialog::new()
                        .add_filter("PS2 Disc Image", &["iso", "bin", "chd"])
                        .pick_file()
                    {
                        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("game");
                        self.native_mgr.start_native_build(stem, &file, self.runtime_config.widescreen_16_9);
                        self.active_tab = ActiveTab::Library;
                    }
                }

                if ui.button("[*] Pindai games/").clicked() {
                    self.native_mgr.scan_games();
                    fetch_missing_box_arts(&self.native_mgr.games);
                }

                if ui.button("[+] Kompilasi Semua (Batch)").clicked() {
                    let enqueued = self.native_mgr.enqueue_all_unbuilt(self.runtime_config.widescreen_16_9);
                    if enqueued > 0 {
                        self.active_tab = ActiveTab::Logs;
                    }
                }

                if ui.button("[^] Unduh Box Art").clicked() {
                    fetch_missing_box_arts(&self.native_mgr.games);
                }

                // Search box
                ui.separator();
                ui.label(RichText::new("Cari:").color(Color32::from_rgb(139, 148, 158)));
                ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("Filter judul atau serial...")
                        .desired_width(180.0),
                );

                // View Switch
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.selectable_label(!self.table_view_mode, "[Card Grid]").clicked() {
                        self.table_view_mode = false;
                    }
                    if ui.selectable_label(self.table_view_mode, "[List Table]").clicked() {
                        self.table_view_mode = true;
                    }
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(2.0);

            // Technical Navigation Tabs
            ui.horizontal(|ui| {
                let tabs = [
                    (ActiveTab::Library, "PUSTAKA GAME (LIBRARY)"),
                    (ActiveTab::Graphics, "GRAFIS & UPSCALING (GS)"),
                    (ActiveTab::Controls, "KONTROL & GLYPH (PAD)"),
                    (ActiveTab::Settings, "PENGATURAN SISTEM"),
                    (ActiveTab::Logs, "LOG KONSOL KOMPILASI"),
                ];

                for (tab, name) in tabs {
                    let is_active = self.active_tab == tab;
                    let text = if is_active {
                        RichText::new(format!("[ {} ]", name))
                            .strong()
                            .size(13.0)
                            .color(Color32::WHITE)
                    } else {
                        RichText::new(name)
                            .size(12.0)
                            .color(Color32::from_rgb(139, 148, 158))
                    };

                    if ui.selectable_label(is_active, text).clicked() {
                        self.active_tab = tab;
                    }
                }
            });
            ui.add_space(4.0);
        });

        // -------------------------------------------------------------
        // 3. PCSX2 BOTTOM STATUS BAR
        // -------------------------------------------------------------
        egui::TopBottomPanel::bottom("pcsx2_statusbar").show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                let ready_count = self.native_mgr.games.iter().filter(|g| g.is_ready).count();
                let total_count = self.native_mgr.games.len();
                ui.label(
                    RichText::new(format!("Game terdaftar: {} (Siap: {})", total_count, ready_count))
                        .size(11.0)
                        .color(Color32::from_rgb(139, 148, 158)),
                );

                ui.separator();
                ui.label(
                    RichText::new("Engine: LLVM MinGW Clang 23.1 AOT (x86_64 Native)")
                        .size(11.0)
                        .color(Color32::from_rgb(52, 211, 153)),
                );

                ui.separator();
                ui.label(
                    RichText::new("Renderer: WebGPU / Vulkan Core")
                        .size(11.0)
                        .color(Color32::from_rgb(56, 189, 248)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Target: 60.00 FPS (100.0%) | 1280x720 (16:9)")
                            .monospace()
                            .size(11.0)
                            .color(Color32::from_rgb(203, 213, 225)),
                    );
                });
            });
            ui.add_space(2.0);
        });

        // -------------------------------------------------------------
        // 4. MAIN CENTRAL CONTENT AREA
        // -------------------------------------------------------------
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                ActiveTab::Library => {
                    if let Some((g_name, child)) = ui::library_tab::render_library_tab(
                        ui,
                        &mut self.settings,
                        &mut self.runtime_config,
                        &mut self.native_mgr,
                        &self.search_query,
                        self.table_view_mode,
                        &mut self.selected_game_index,
                    ) {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        self.running_game = Some((g_name, std::time::Instant::now(), child));
                    }
                }
                ActiveTab::Controls => {
                    ui::controls_tab::render_controls_tab(
                        ui,
                        &mut self.settings,
                        &mut self.runtime_config,
                        &mut self.gamepad,
                    );
                }
                ActiveTab::Graphics => {
                    ui::graphics_tab::render_graphics_tab(ui, &mut self.runtime_config);
                }
                ActiveTab::Settings => {
                    ui::settings_tab::render_settings_tab(
                        ui,
                        &mut self.settings,
                        &mut self.runtime_config,
                    );
                }
                ActiveTab::Logs => {
                    render_logs_tab(ui, &mut self.native_mgr);
                }
            }
        });

        // Hotkeys help dialog
        if self.show_hotkey_dialog {
            let mut open = true;
            let mut close_dialog = false;
            egui::Window::new("Hotkey & Pintasan Tombol")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label(RichText::new("Pintasan Keyboard di Dalam Game:").strong());
                    ui.add_space(6.0);
                    ui.label("F11       : Toggle Borderless Fullscreen");
                    ui.label("F2        : Ganti Filter Gambar (Crisp / Soft / Sharp / Bilinear)");
                    ui.label("Insert    : Buka Menu In-Game Port (Display, Upscaler, Audio)");
                    ui.label("R3 + L2   : Buka Menu In-Game menggunakan Controller");
                    ui.label("F5 / F8   : Simpan Status (Save State) / Muat Status (Load State)");
                    ui.label("Esc       : Buka Menu Jeda / Kembalikan Kursor Mouse");
                    ui.add_space(10.0);
                    if ui.button("Tutup").clicked() {
                        close_dialog = true;
                    }
                });
            if !open || close_dialog {
                self.show_hotkey_dialog = false;
            }
        }

        // About dialog
        if self.show_about_dialog {
            let mut open = true;
            let mut close_dialog = false;
            egui::Window::new("Tentang DivinePort")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .show(ctx, |ui| {
                    if let Some(ref logo) = self.logo_texture {
                        ui.vertical_centered(|ui| {
                            ui.image((logo.id(), Vec2::new(180.0, 180.0)));
                        });
                        ui.add_space(8.0);
                    }
                    ui.label(RichText::new("DivinePort - Native Static Recompiler").strong().size(15.0));
                    ui.label(RichText::new(format!("Versi {} (Release)", env!("CARGO_PKG_VERSION"))).color(Color32::from_rgb(56, 189, 248)));
                    ui.add_space(6.0);
                    ui.label("Kompilasi statis Ahead-Of-Time (AOT) dari instruksi biner PS2 MIPS R5900 langsung ke kode C++20 dan mesin x86_64 Windows.");
                    ui.label("Menggabungkan arsitektur grafis SpartanPort (AMD FSR 3.1 / 4.1, DLSS, TAA, Frame Generation) dengan sistem kontrol dan pustaka game terstruktur PCSX2.");
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(6.0);
                    ui.label(RichText::new("Dukungan & Kontribusi:").strong());
                    ui.label("Jika Anda menyukai proyek ini dan ingin memberikan dukungan:");
                    if ui.button(RichText::new("Kunjungi Website Support (supportzhou.vercel.app)").color(Color32::from_rgb(59, 130, 246)).strong()).clicked() {
                        let _ = std::process::Command::new("cmd")
                            .args(["/c", "start", "https://supportzhou.vercel.app"])
                            .spawn();
                    }
                    ui.add_space(10.0);
                    if ui.button("Tutup").clicked() {
                        close_dialog = true;
                    }
                });
            if !open || close_dialog {
                self.show_about_dialog = false;
            }
        }

        // -------------------------------------------------------------
        // INSTANT AUTO-SAVE: If any setting changed, save immediately to disk
        // -------------------------------------------------------------
        if self.settings != prev_settings || self.runtime_config != prev_runtime {
            let _ = self.settings.save(Path::new("settings.json"));
            let _ = self.runtime_config.save_json(Path::new("runtime_config.json"));
            let _ = self.runtime_config.save_to_ini(Path::new("spartanport.ini"));
        }

        ctx.request_repaint();
    }
}

fn render_logs_tab(ui: &mut egui::Ui, native_mgr: &mut NativeManager) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.heading(RichText::new("Terminal Log Kompilasi AOT (Clang & Ninja)").strong().size(18.0));
            ui.label(RichText::new("Output log real-time proses translasi mesin dan kompilasi biner C++20.").color(Color32::from_rgb(139, 148, 158)));
            ui.add_space(8.0);

    ui.horizontal(|ui| {
        if native_mgr.is_building {
            ui.spinner();
            ui.label(RichText::new(format!(" Status: {}", native_mgr.current_stage)).strong().color(Color32::from_rgb(56, 189, 248)));
        } else {
            ui.label(RichText::new(format!(" Status: {}", native_mgr.current_stage)).strong().color(Color32::from_rgb(52, 211, 153)));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Bersihkan Log").clicked() {
                native_mgr.log_lines.clear();
            }
        });
    });

    ui.add_space(8.0);

    egui::Frame::none()
        .fill(Color32::from_rgb(10, 15, 26))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 54, 61)))
        .inner_margin(12.0)
        .rounding(6.0)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if native_mgr.log_lines.is_empty() {
                        ui.label(RichText::new("Tidak ada aktivitas log saat ini.").italics().color(Color32::from_rgb(100, 116, 139)));
                    } else {
                        for line in &native_mgr.log_lines {
                            let col = if line.contains("[STDERR]") {
                                Color32::from_rgb(248, 113, 113)
                            } else if line.contains("===>") || line.contains("successfully") {
                                Color32::from_rgb(52, 211, 153)
                            } else {
                                Color32::from_rgb(148, 163, 184)
                            };
                            ui.label(RichText::new(line).monospace().size(11.0).color(col));
                        }
                    }
                });
        });

        ui.add_space(16.0);
    });
}

fn fetch_missing_box_arts(games: &[crate::native::NativeGameInfo]) {
    let games_clone = games.to_vec();
    std::thread::spawn(move || {
        let _ = std::fs::create_dir_all("covers");
        for g in games_clone {
            let cover_path = format!("covers/{}.jpg", g.name);
            if !std::path::Path::new(&cover_path).exists() {
                let serial = if g.elf.contains("SLUS_209.64") || g.name.contains("devil-may-cry-3") {
                    "SLUS-20964".to_string()
                } else if g.elf.contains("SLES_544.90") || g.name.contains("god-hand") {
                    "SLES-54490".to_string()
                } else if g.elf.contains("SLUS_212.15") || g.name.contains("warriors") {
                    "SLUS-21215".to_string()
                } else {
                    let mut found = String::new();
                    let full_txt = format!("{} {}", g.elf, g.source).to_uppercase();
                    let prefixes = ["SLUS_", "SLES_", "SCUS_", "SCES_", "SLPM_", "SLPS_", "SCPS_", "SLAJ_", "SCAJ_", "SLKA_"];
                    for p in prefixes {
                        if let Some(pos) = full_txt.find(p) {
                            let s = &full_txt[pos..];
                            let candidate = s.chars().take(11).collect::<String>().replace('_', "-").replace('.', "");
                            if candidate.len() >= 9 {
                                found = candidate;
                                break;
                            }
                        }
                    }
                    found
                };

                if !serial.is_empty() {
                    let url = format!(
                        "https://raw.githubusercontent.com/xlenore/ps2-covers/main/covers/default/{}.jpg",
                        serial
                    );
                    let _ = std::process::Command::new("powershell")
                        .args([
                            "-NoProfile",
                            "-Command",
                            &format!(
                                "Invoke-WebRequest -Uri '{}' -OutFile '{}' -TimeoutSec 10",
                                url, cover_path
                            ),
                        ])
                        .status();
                }
            }
        }
    });
}

fn load_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../assets/logo.jpg");
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(Vec2::new(1040.0, 720.0))
        .with_min_inner_size(Vec2::new(880.0, 580.0))
        .with_title("DivinePort - Native Static Recompiler");

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "DivinePort",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

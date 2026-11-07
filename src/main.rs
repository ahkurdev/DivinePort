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
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings_path = Path::new("settings.json");
        let settings = LauncherSettings::load_or_default(settings_path);
        let runtime_config = GameRuntimeConfig::default();
        let gamepad = GamepadManager::new();

        let recomp_root = if Path::new("../ps2-native-recomp").exists() {
            PathBuf::from("../ps2-native-recomp")
        } else if Path::new("ps2-native-recomp").exists() {
            PathBuf::from("ps2-native-recomp")
        } else {
            settings.recomp_dir.clone()
        };

        let native_mgr = NativeManager::new(recomp_root);

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
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.gamepad.update();
        self.native_mgr.update_build_logs();

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
                    if ui.button("Hotkeys & Shortcuts...").clicked() {
                        self.show_hotkey_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("About PS2 Native Recomp...").clicked() {
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
                    RichText::new("| PS2 Static Recompiler (SpartanPort x PCSX2 Unified)")
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
                    ui::library_tab::render_library_tab(
                        ui,
                        &mut self.settings,
                        &mut self.runtime_config,
                        &mut self.native_mgr,
                        &self.search_query,
                        self.table_view_mode,
                    );
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
                            ui.image((logo.id(), Vec2::new(160.0, 160.0)));
                        });
                        ui.add_space(8.0);
                    }
                    ui.label(RichText::new("DivinePort - PS2 Static Recompiler Launcher").strong().size(15.0));
                    ui.add_space(6.0);
                    ui.label("Kompilasi statis Ahead-Of-Time (AOT) dari instruksi biner PS2 MIPS R5900 langsung ke kode C++20 dan mesin x86_64 Windows.");
                    ui.label("Menggabungkan arsitektur grafis SpartanPort (AMD FSR 3.1 / 4.1, DLSS, TAA, Frame Generation) dengan sistem kontrol dan pustaka game terstruktur PCSX2.");
                    ui.add_space(10.0);
                    if ui.button("Tutup").clicked() {
                        close_dialog = true;
                    }
                });
            if !open || close_dialog {
                self.show_about_dialog = false;
            }
        }

        ctx.request_repaint();
    }
}

fn render_logs_tab(ui: &mut egui::Ui, native_mgr: &mut NativeManager) {
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
        .with_title("DivinePort - Native PS2 Static Recompiler Launcher");

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

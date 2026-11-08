use crate::config::{FrameGenMode, GameRuntimeConfig, UpscalePreset, UpscalerType};
use eframe::egui::{self, Color32, RichText, Stroke};

pub fn render_graphics_tab(ui: &mut egui::Ui, runtime_config: &mut GameRuntimeConfig) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.heading(RichText::new("PENGATURAN GRAFIS & TEMPORAL UPSCALING (GS)").strong().size(18.0));
            ui.label(RichText::new("Sistem Upscaler Temporal FidelityFX (FSR 3.1, FSR 4, FSR 4.1, DLSS, TAA) & Frame Generation arsitektur SpartanPort.").color(Color32::from_rgb(139, 148, 158)));
            ui.add_space(8.0);

    // -------------------------------------------------------------
    // SECTION 1: RESOLUSI & DISPLAY MODE
    // -------------------------------------------------------------
    egui::Frame::none()
        .fill(Color32::from_rgb(18, 24, 38))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 54, 61)))
        .inner_margin(12.0)
        .rounding(6.0)
        .show(ui, |ui| {
            ui.heading(RichText::new("Resolusi Layar & Output Display").size(15.0));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Target Output Resolution:");
                let targets = [("720p", "720p (1280x720)"), ("1080p", "1080p (1920x1080)"), ("1440p", "1440p (2K)"), ("4k", "4K UHD (3840x2160)")];
                for (val, label) in targets {
                    if ui.selectable_label(runtime_config.target_resolution == val, label).clicked() {
                        runtime_config.target_resolution = val.to_string();
                    }
                }
            });

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Internal PS2 Render Scale:");
                if ui.selectable_label(runtime_config.render_scale == 1, "1x (Native PS2 480i/p)").clicked() {
                    runtime_config.render_scale = 1;
                }
                if ui.selectable_label(runtime_config.render_scale == 2, "2x (HD 960p Sharp Raster)").clicked() {
                    runtime_config.render_scale = 2;
                }
            });

            ui.add_space(6.0);
            ui.checkbox(&mut runtime_config.widescreen_16_9, "Widescreen 16:9 Aspect Ratio (--widescreen)");
            ui.label(RichText::new("   Memperluas viewport rendering native ke aspek rasio 16:9 widescreen.").small().color(Color32::from_rgb(139, 148, 158)));

            ui.add_space(6.0);
            ui.checkbox(&mut runtime_config.fullscreen, "Mulai dalam Mode Fullscreen (--fullscreen)");
            ui.label(RichText::new("   Layar penuh tanpa border (Tekan F11 kapan saja di dalam game untuk toggle).").small().color(Color32::from_rgb(139, 148, 158)));
        });

    ui.add_space(10.0);

    // -------------------------------------------------------------
    // SECTION 2: TEMPORAL UPSCALERS (FSR, DLSS, TAA)
    // -------------------------------------------------------------
    egui::Frame::none()
        .fill(Color32::from_rgb(18, 24, 38))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 54, 61)))
        .inner_margin(12.0)
        .rounding(6.0)
        .show(ui, |ui| {
            ui.heading(RichText::new("Temporal Upscaling Engine (FSR 3.1 / 4 / 4.1 / DLSS / TAA)").size(15.0));
            ui.add_space(6.0);

            // Upscaler selector combo
            ui.horizontal(|ui| {
                ui.label(RichText::new("Upscaler Active:").strong());
                egui::ComboBox::from_id_salt("upscaler_selector")
                    .selected_text(runtime_config.upscaler.display_name())
                    .width(380.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut runtime_config.upscaler, UpscalerType::Off, UpscalerType::Off.display_name());
                        ui.separator();
                        ui.selectable_value(&mut runtime_config.upscaler, UpscalerType::Fsr3, UpscalerType::Fsr3.display_name());
                        ui.selectable_value(&mut runtime_config.upscaler, UpscalerType::Fsr3Ui, UpscalerType::Fsr3Ui.display_name());
                        ui.selectable_value(&mut runtime_config.upscaler, UpscalerType::Fsr4, UpscalerType::Fsr4.display_name());
                        ui.selectable_value(&mut runtime_config.upscaler, UpscalerType::Fsr411, UpscalerType::Fsr411.display_name());
                        ui.separator();
                        ui.selectable_value(&mut runtime_config.upscaler, UpscalerType::Dlss, UpscalerType::Dlss.display_name());
                        ui.separator();
                        ui.selectable_value(&mut runtime_config.upscaler, UpscalerType::Taa, UpscalerType::Taa.display_name());
                    });
            });

            // Upscaler explanation hint
            let upscaler_hint = match runtime_config.upscaler {
                UpscalerType::Off => "Upscaling dinonaktifkan. Game dirender pada resolusi target standar.",
                UpscalerType::Fsr3 => "FSR 3.1 berjalan di semua GPU (NVIDIA, AMD, Intel). Menghasilkan rekonstruksi temporal berkecepatan tinggi.",
                UpscalerType::Fsr3Ui => "FSR 3.1 (Native HUD & Text): HUD, teks subtitle, dan menu tetap dirender tajam 1080p tanpa ikut di-stretch.",
                UpscalerType::Fsr4 => "FSR 4: Model open source FSR 4 terintegrasi untuk AMD Radeon RX 6000 atau lebih baru dan Intel Arc (Windows & Linux).",
                UpscalerType::Fsr411 => "FSR 4.1: Berjalan lewat AMD Vulkan runtime (amdvkffx64.dll) bawaan driver AMD Adrenalin 26.10.1+. (Fallback otomatis ke FSR 3.1 pada GPU lain).",
                UpscalerType::Dlss => "NVIDIA DLSS (Deep Learning Super Sampling): Menggunakan Tensor Core AI hardware NVIDIA RTX untuk rekonstruksi gambar terbaik tanpa ghosting.",
                UpscalerType::Taa => "TAA (Temporal Anti-Aliasing): Anti-aliasing native resolution tanpa upscaling. Menghilangkan tepi objek bergerigi secara temporal.",
            };
            ui.label(RichText::new(format!("[Info] {}", upscaler_hint)).small().color(Color32::from_rgb(139, 148, 158)));

            if runtime_config.upscaler != UpscalerType::Off && runtime_config.upscaler != UpscalerType::Taa {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Quality Preset:").strong());
                    let presets = [
                        (UpscalePreset::NativeAa, "Native AA (100%)"),
                        (UpscalePreset::UltraQuality, "Ultra Quality (77%)"),
                        (UpscalePreset::Quality, "Quality (67%)"),
                        (UpscalePreset::Balanced, "Balanced (59%)"),
                        (UpscalePreset::Performance, "Performance (50%)"),
                        (UpscalePreset::UltraPerformance, "Ultra Perf (33%)"),
                    ];
                    for (preset, label) in presets {
                        if ui.selectable_label(runtime_config.upscale_preset == preset, label).clicked() {
                            runtime_config.upscale_preset = preset;
                        }
                    }
                });

                // Display dynamic calculation of render resolution
                let (rw, rh, tw, th) = runtime_config.calculate_render_resolution();
                ui.add_space(4.0);
                ui.colored_label(
                    Color32::from_rgb(52, 211, 153),
                    RichText::new(format!("[Pipeline] Render Internal: {}x{}  ->  Output Upscaled: {}x{}", rw, rh, tw, th)).strong().small(),
                );
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            // -------------------------------------------------------------
            // FRAME GENERATION (2X)
            // -------------------------------------------------------------
            ui.horizontal(|ui| {
                ui.label(RichText::new("Frame Generation:").strong());
                ui.selectable_value(&mut runtime_config.frame_generation, FrameGenMode::Off, "Off");
                ui.selectable_value(&mut runtime_config.frame_generation, FrameGenMode::Fsr2x, "AMD FSR Frame Gen (2x)");
                ui.selectable_value(&mut runtime_config.frame_generation, FrameGenMode::DlssFg, "NVIDIA DLSS Frame Gen");
            });

            let fg_hint = match runtime_config.frame_generation {
                FrameGenMode::Off => "Frame Generation nonaktif.",
                FrameGenMode::Fsr2x => "AMD FSR Frame Generation (2x) menyisipkan 1 frame interpolasi di antara setiap frame asli (60 FPS -> 120 FPS).",
                FrameGenMode::DlssFg => "NVIDIA DLSS 3 Frame Generation: Optical Flow Accelerator menghasilkan frame interpolasi berkecepatan tinggi khusus kartu grafis seri RTX 40.",
            };
            ui.label(RichText::new(format!("[Info] {}", fg_hint)).small().color(Color32::from_rgb(139, 148, 158)));

            ui.add_space(8.0);
            ui.label(format!("RCAS Sharpening (Robust Contrast Adaptive Sharpening): {:.2}", runtime_config.rcas_sharpening));
            ui.add(egui::Slider::new(&mut runtime_config.rcas_sharpening, 0.0..=1.0).text("Ketajaman Tepi"));
        });

    ui.add_space(10.0);

    // -------------------------------------------------------------
    // SECTION 3: MEMORY & GPU PIPELINE
    // -------------------------------------------------------------
    egui::Frame::none()
        .fill(Color32::from_rgb(18, 24, 38))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 54, 61)))
        .inner_margin(12.0)
        .rounding(6.0)
        .show(ui, |ui| {
            ui.heading(RichText::new("Memori & GPU Pipeline (Spartan Memory Architecture)").size(15.0));
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label("Stream Buffer Memory:");
                let mem_options = [
                    ("cached", "System RAM (Cached - Default)"),
                    ("host", "System RAM (Uncached)"),
                    ("device", "GPU Dedicated Memory"),
                ];
                for (val, desc) in mem_options {
                    if ui.selectable_label(runtime_config.stream_memory == val, desc).clicked() {
                        runtime_config.stream_memory = val.to_string();
                    }
                }
            });
            ui.label(RichText::new("   System RAM (cached) meningkatkan stabilitas frame rate pada laptop dengan GPU diskrit NVIDIA.").small().color(Color32::from_rgb(139, 148, 158)));

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Game Multisampling (MSAA):");
                let msaa_opts = [("auto", "Automatic (Rekomendasi)"), ("off", "Off (Fix garis hitam AMD/Intel)"), ("on", "On (NVIDIA)")];
                for (val, desc) in msaa_opts {
                    if ui.selectable_label(runtime_config.msaa_mode == val, desc).clicked() {
                        runtime_config.msaa_mode = val.to_string();
                    }
                }
            });
            ui.label(RichText::new("   Automatic mendeteksi driver kartu grafis untuk mencegah glitch garis hitam pada RX 6600 & Intel Arc.").small().color(Color32::from_rgb(139, 148, 158)));
        });

    ui.add_space(10.0);

    // -------------------------------------------------------------
    // SECTION 4: FRAME PACING & SINKRONISASI (SPARTANPORT 120 FPS & PCSX2 ASYNC)
    // -------------------------------------------------------------
    egui::Frame::none()
        .fill(Color32::from_rgb(18, 24, 38))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(52, 211, 153)))
        .inner_margin(12.0)
        .rounding(6.0)
        .show(ui, |ui| {
            ui.heading(RichText::new("Optimal Frame Pacing & Adaptasi PCSX2 / SpartanPort Engine").size(15.0).color(Color32::from_rgb(52, 211, 153)));
            ui.add_space(6.0);

            ui.checkbox(
                &mut runtime_config.lock_game_60_display_120,
                RichText::new("SpartanPort 120 FPS Mode: Tampilan 120 FPS dengan Game Logic Terkunci 60 FPS").strong(),
            );
            ui.label(RichText::new("   Mesin merender swapchain pada 120 FPS menggunakan AMD FSR 3.1 Frame Gen, sementara clock simulasi game PS2 tetap terkunci di 60.0 FPS murni (mencegah game bergerak terlalu cepat/liar).").small().color(Color32::from_rgb(139, 148, 158)));

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Graphics Backend API:").strong());
                let backend_opts = [
                    ("vulkan", "Vulkan Core (Rekomendasi Utama: Zero Hitching & Latensi Rendah)"),
                    ("dx12", "Direct3D 12 (D3D12 Default)"),
                ];
                for (val, desc) in backend_opts {
                    if ui.selectable_label(runtime_config.graphics_backend == val, desc).clicked() {
                        runtime_config.graphics_backend = val.to_string();
                    }
                }
            });
            ui.label(RichText::new("   Vulkan memotong overhead driver grafis laptop NVIDIA RTX & AMD Radeon dan menghilangkan jeda presentasi D3D12.").small().color(Color32::from_rgb(139, 148, 158)));

            ui.add_space(8.0);
            ui.checkbox(
                &mut runtime_config.async_audio,
                RichText::new("Sinkronisasi Audio Asinkron (PCSX2 SPU2 Async Mix)").strong(),
            );
            ui.label(RichText::new("   Memisahkan clock audio dari loop emulasi video. Mencegah bug 'stretched audio' di mana device audio virtual (seperti SteelSeries Sonar) memperlambat game menjadi 14 FPS.").small().color(Color32::from_rgb(139, 148, 158)));
        });

        ui.add_space(16.0);
    });
}

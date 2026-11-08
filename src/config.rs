use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlyphStyle {
    Auto,
    PlayStation,
    Xbox,
    Nintendo,
    Keyboard,
}

impl Default for GlyphStyle {
    fn default() -> Self {
        GlyphStyle::Auto
    }
}

impl GlyphStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            GlyphStyle::Auto => "auto",
            GlyphStyle::PlayStation => "playstation",
            GlyphStyle::Xbox => "xbox",
            GlyphStyle::Nintendo => "nintendo",
            GlyphStyle::Keyboard => "keyboard",
        }
    }

    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            GlyphStyle::Auto => "Auto (Match my controller)",
            GlyphStyle::PlayStation => "PlayStation (✕ ○ □ △)",
            GlyphStyle::Xbox => "Xbox (A B X Y)",
            GlyphStyle::Nintendo => "Nintendo (B A Y X)",
            GlyphStyle::Keyboard => "Keyboard & Mouse",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LauncherSettings {
    pub gui_scale: String,           // "auto", "1.0", "1.5"
    pub language: String,            // "id", "en", "zh"
    pub glyph_style: GlyphStyle,     // "auto", "playstation", "xbox", etc.
    pub last_game: Option<PathBuf>,
    pub recomp_dir: PathBuf,
    pub dark_theme: bool,
    pub discord_rpc: bool,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            gui_scale: "auto".to_string(),
            language: "id".to_string(),
            glyph_style: GlyphStyle::Auto,
            last_game: None,
            recomp_dir: PathBuf::from("../ps2-native-recomp"),
            dark_theme: true,
            discord_rpc: false,
        }
    }
}

impl LauncherSettings {
    pub fn load_or_default(path: &Path) -> Self {
        if path.exists() {
            if let Ok(data) = fs::read_to_string(path) {
                if let Ok(settings) = serde_json::from_str::<Self>(&data) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }
}

/// Settings specifically written to `recomp.ini` for the native game engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpscalerType {
    Off,
    Fsr3,    // FSR 3.1 (every GPU)
    Fsr3Ui,  // FSR 3.1 (native HUD and text)
    Fsr4,    // FSR 4 (AMD Radeon RX 6000+, Intel Arc)
    Fsr411,  // FSR 4.1 (AMD driver Vulkan runtime)
    Dlss,    // NVIDIA DLSS (RTX GPUs)
    Taa,     // TAA (native resolution temporal AA)
}

impl Default for UpscalerType {
    fn default() -> Self {
        UpscalerType::Off
    }
}

impl UpscalerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            UpscalerType::Off => "off",
            UpscalerType::Fsr3 => "fsr3",
            UpscalerType::Fsr3Ui => "fsr3ui",
            UpscalerType::Fsr4 => "fsr4",
            UpscalerType::Fsr411 => "fsr411",
            UpscalerType::Dlss => "dlss",
            UpscalerType::Taa => "taa",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            UpscalerType::Off => "Off (Matikan Upscaling)",
            UpscalerType::Fsr3 => "AMD FSR 3.1 (Semua Kartu Grafis / Universal)",
            UpscalerType::Fsr3Ui => "AMD FSR 3.1 (Native HUD & Text preserved)",
            UpscalerType::Fsr4 => "AMD FSR 4 (Radeon RX 6000+ & Intel Arc)",
            UpscalerType::Fsr411 => "AMD FSR 4.1 (AMD Adrenalin Vulkan Runtime)",
            UpscalerType::Dlss => "NVIDIA DLSS (GeForce RTX Tensor Cores)",
            UpscalerType::Taa => "TAA (Native Temporal Anti-Aliasing)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpscalePreset {
    NativeAa,         // 100%
    UltraQuality,     // 77% (1.3x)
    Quality,          // 67% (1.5x)
    Balanced,         // 59% (1.7x)
    Performance,      // 50% (2.0x)
    UltraPerformance, // 33% (3.0x)
}

impl Default for UpscalePreset {
    fn default() -> Self {
        UpscalePreset::Quality
    }
}

impl UpscalePreset {
    pub fn as_code(&self) -> &'static str {
        match self {
            UpscalePreset::NativeAa => "0",
            UpscalePreset::Quality => "1",
            UpscalePreset::Balanced => "2",
            UpscalePreset::Performance => "3",
            UpscalePreset::UltraPerformance => "4",
            UpscalePreset::UltraQuality => "5",
        }
    }

    pub fn scale_factor(&self) -> f32 {
        match self {
            UpscalePreset::NativeAa => 1.0,
            UpscalePreset::UltraQuality => 0.77,
            UpscalePreset::Quality => 0.67,
            UpscalePreset::Balanced => 0.59,
            UpscalePreset::Performance => 0.50,
            UpscalePreset::UltraPerformance => 0.33,
        }
    }

    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            UpscalePreset::NativeAa => "Native AA (100% - Full Res AA)",
            UpscalePreset::UltraQuality => "Ultra Quality (77% Render)",
            UpscalePreset::Quality => "Quality (67% Render)",
            UpscalePreset::Balanced => "Balanced (59% Render)",
            UpscalePreset::Performance => "Performance (50% Render)",
            UpscalePreset::UltraPerformance => "Ultra Performance (33% Render)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameGenMode {
    Off,
    Fsr2x, // AMD FSR Frame Generation 2x
    DlssFg, // NVIDIA DLSS Frame Generation (RTX 40-series)
}

impl Default for FrameGenMode {
    fn default() -> Self {
        FrameGenMode::Off
    }
}

impl FrameGenMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            FrameGenMode::Off => "off",
            FrameGenMode::Fsr2x => "fsr",
            FrameGenMode::DlssFg => "dlss",
        }
    }

    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            FrameGenMode::Off => "Off (Nonaktif)",
            FrameGenMode::Fsr2x => "AMD FSR Frame Generation (2x)",
            FrameGenMode::DlssFg => "NVIDIA DLSS Frame Generation (RTX 40+)",
        }
    }
}

/// Settings specifically written to `recomp.ini` & `spartanport.ini`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameRuntimeConfig {
    pub glyph_style: GlyphStyle,
    pub render_scale: u32,       // 1x, 2x
    pub widescreen_16_9: bool,
    pub fullscreen: bool,
    pub upscaler: UpscalerType,
    pub upscale_preset: UpscalePreset,
    pub frame_generation: FrameGenMode,
    pub target_resolution: String, // "1080p", "1440p", "4k"
    pub rcas_sharpening: f32,    // 0.0 to 1.0
    pub rumble_enabled: bool,
    pub stream_memory: String,   // "cached", "host", "device"
    pub msaa_mode: String,       // "auto", "off", "on"
    pub graphics_backend: String, // "vulkan" (default, lower latency), "dx12"
    pub async_audio: bool,        // true = PCSX2 SPU2 Async Mix (prevents audio stretch lag)
    pub lock_game_60_display_120: bool, // SpartanPort 120 FPS display with 60 FPS game lock
}

impl Default for GameRuntimeConfig {
    fn default() -> Self {
        Self {
            glyph_style: GlyphStyle::Auto,
            render_scale: 1,
            widescreen_16_9: true,
            fullscreen: false,
            upscaler: UpscalerType::Off,
            upscale_preset: UpscalePreset::Quality,
            frame_generation: FrameGenMode::Off,
            target_resolution: "1080p".to_string(),
            rcas_sharpening: 0.5,
            rumble_enabled: true,
            stream_memory: "cached".to_string(),
            msaa_mode: "auto".to_string(),
            graphics_backend: "vulkan".to_string(),
            async_audio: true,
            lock_game_60_display_120: false,
        }
    }
}

impl GameRuntimeConfig {
    pub fn load_or_default(path: &Path) -> Self {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(cfg) = serde_json::from_str::<GameRuntimeConfig>(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save_json(&self, path: &Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, json)
    }

    pub fn calculate_render_resolution(&self) -> (u32, u32, u32, u32) {
        let (tw, th) = match self.target_resolution.as_str() {
            "720p" => (1280, 720),
            "1440p" => (2560, 1440),
            "4k" => (3840, 2160),
            _ => (1920, 1080), // 1080p default
        };

        if self.upscaler == UpscalerType::Off || self.upscaler == UpscalerType::Taa {
            return (tw, th, tw, th);
        }

        let scale = self.upscale_preset.scale_factor();
        let rw = ((tw as f32 * scale).round() as u32) & !1;
        let rh = ((th as f32 * scale).round() as u32) & !1;
        (rw, rh, tw, th)
    }

    pub fn save_to_ini(&self, path: &Path) -> Result<(), std::io::Error> {
        let (rw, rh, tw, th) = self.calculate_render_resolution();
        let content = format!(
            r#"# SpartanPort / DivinePort Runtime Configuration
[Graphics]
render_scale = {}
widescreen_16_9 = {}
fullscreen = {}
upscaler = {}
preset = {}
frame_generation = {}
target_resolution = {}
internal_render_width = {}
internal_render_height = {}
output_width = {}
output_height = {}
rcas_sharpening = {:.2}
stream_memory = {}
msaa = {}
graphics_backend = {}
async_audio = {}
lock_game_60_display_120 = {}

[Controls]
glyph_style = {}
rumble_enabled = {}
"#,
            self.render_scale,
            self.widescreen_16_9,
            self.fullscreen,
            self.upscaler.as_str(),
            self.upscale_preset.as_code(),
            self.frame_generation.as_str(),
            self.target_resolution,
            rw,
            rh,
            tw,
            th,
            self.rcas_sharpening,
            self.stream_memory,
            self.msaa_mode,
            self.graphics_backend,
            self.async_audio,
            self.lock_game_60_display_120,
            self.glyph_style.as_str(),
            self.rumble_enabled
        );

        fs::write(path, content)
    }
}

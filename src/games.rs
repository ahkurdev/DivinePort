#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEntry {
    pub id: String,
    pub title: String,
    pub iso_path: PathBuf,
    pub built: bool,
    pub executable_path: Option<PathBuf>,
}

pub struct GameManager {
    pub games: Vec<GameEntry>,
    pub games_dir: PathBuf,
}

impl GameManager {
    pub fn new(base_dir: &Path) -> Self {
        let games_dir = base_dir.join("games");
        let mut mgr = Self {
            games: Vec::new(),
            games_dir,
        };
        mgr.scan();
        mgr
    }

    pub fn scan(&mut self) {
        self.games.clear();
        if !self.games_dir.exists() {
            let _ = std::fs::create_dir_all(&self.games_dir);
        }

        if let Ok(entries) = std::fs::read_dir(&self.games_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if ext.eq_ignore_ascii_case("iso") || ext.eq_ignore_ascii_case("bin") {
                        let title = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Unknown Game")
                            .to_string();

                        let id = title.to_lowercase().replace(' ', "_");
                        
                        // Check if an executable exists in build/native-<id>/ps2run.exe
                        let exec_path = self.games_dir
                            .parent()
                            .map(|p| p.join("build").join(format!("native-{}", id)).join("ps2run.exe"));

                        let built = exec_path.as_ref().map(|p| p.exists()).unwrap_or(false);

                        self.games.push(GameEntry {
                            id,
                            title,
                            iso_path: path,
                            built,
                            executable_path: if built { exec_path } else { None },
                        });
                    }
                }
            }
        }
    }

    pub fn add_game(&mut self, iso_path: PathBuf) {
        let title = iso_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Custom Game")
            .to_string();

        let id = title.to_lowercase().replace(' ', "_");
        
        let exec_path = self.games_dir
            .parent()
            .map(|p| p.join("build").join(format!("native-{}", id)).join("ps2run.exe"));

        let built = exec_path.as_ref().map(|p| p.exists()).unwrap_or(false);

        self.games.push(GameEntry {
            id,
            title,
            iso_path,
            built,
            executable_path: if built { exec_path } else { None },
        });
    }

    pub fn launch_game(
        &self,
        game: &GameEntry,
        glyph_style: &str,
        render_scale: u32,
        widescreen: bool,
    ) -> Result<std::process::Child, std::io::Error> {
        let mut cmd = if let Some(ref exec) = game.executable_path {
            Command::new(exec)
        } else {
            // Default runner if available
            Command::new("ps2run.exe")
        };

        cmd.env("SPARTAN_GLYPH_STYLE", glyph_style);
        cmd.env("PS2_SCALE", render_scale.to_string());
        if widescreen {
            cmd.arg("--pnach-widescreen");
        }
        cmd.arg("--window");
        cmd.arg("--disc").arg(&game.iso_path);

        cmd.spawn()
    }
}

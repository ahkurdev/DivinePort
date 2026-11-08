use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::SystemTime;

fn chrono_or_simple_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple format YYYY-MM-DD or readable date
    let days = now / 86400;
    let rem_secs = now % 86400;
    let hours = (rem_secs / 3600 + 7) % 24; // UTC+7 approximate
    let minutes = (rem_secs % 3600) / 60;
    format!("Hari ke-{}, {:02}:{:02} WIB", days, hours, minutes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeGameInfo {
    pub name: String,
    pub source: String,
    pub built: String,
    pub took: String,
    pub elf: String,
    pub iso: String,
    pub pnach: String,
    pub sections: Vec<String>,
    pub patches: String,
    pub widescreen: String,
    #[serde(default)]
    pub playtime_seconds: u64,
    #[serde(default)]
    pub last_played: String,
    #[serde(skip)]
    pub is_ready: bool,
    #[serde(skip)]
    pub exe_path: Option<PathBuf>,
}

pub struct NativeManager {
    pub recomp_root: PathBuf,
    pub games: Vec<NativeGameInfo>,
    pub log_lines: VecDeque<String>,
    pub build_receiver: Option<Receiver<String>>,
    pub is_building: bool,
    pub build_progress: f32, // 0.0 - 1.0
    pub current_stage: String,
    pub batch_queue: VecDeque<(String, PathBuf, bool)>,
}

impl NativeManager {
    pub fn new(recomp_root: PathBuf) -> Self {
        let mut mgr = Self {
            recomp_root,
            games: Vec::new(),
            log_lines: VecDeque::with_capacity(1000),
            build_receiver: None,
            is_building: false,
            build_progress: 0.0,
            current_stage: "Idle".to_string(),
            batch_queue: VecDeque::new(),
        };
        mgr.scan_games();
        mgr
    }

    pub fn scan_games(&mut self) {
        self.games.clear();
        let games_dir = self.recomp_root.join("games");
        if !games_dir.exists() {
            let _ = fs::create_dir_all(&games_dir);
        }

        // 1. Scan already recompiled games with app.json
        if let Ok(entries) = fs::read_dir(&games_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let app_json = path.join("app.json");
                    if app_json.exists() {
                        if let Ok(content) = fs::read_to_string(&app_json) {
                            if let Ok(mut info) = serde_json::from_str::<NativeGameInfo>(&content) {
                                let exe = self
                                    .recomp_root
                                    .join("build")
                                    .join(format!("native-{}", info.name))
                                    .join("ps2run.exe");

                                info.is_ready = exe.exists();
                                info.exe_path = if info.is_ready { Some(exe) } else { None };
                                self.games.push(info);
                            }
                        }
                    }
                }
            }
        }

        // 2. Scan raw ISO files in ./games/ and launcher games/ directory
        let search_dirs = [
            PathBuf::from("games"),
            self.recomp_root.join("games"),
            PathBuf::from("../ps2-recomp-launcher/games"),
        ];

        for search_dir in &search_dirs {
            if let Ok(entries) = fs::read_dir(search_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            let ext_lower = ext.to_ascii_lowercase();
                            if ext_lower == "iso" || ext_lower == "bin" || ext_lower == "chd" {
                                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                let safe_name = Self::sanitize_game_name(&file_name);
                                let norm_file_lower = file_name.to_lowercase();

                                // Check if already listed by safe_name, source filename, or iso path
                                let already_exists = self.games.iter().any(|g| {
                                    if g.name == safe_name {
                                        return true;
                                    }
                                    let g_source_name = Path::new(&g.source)
                                        .file_name()
                                        .map(|s| s.to_string_lossy().to_lowercase())
                                        .unwrap_or_default();
                                    if !g_source_name.is_empty() && g_source_name == norm_file_lower {
                                        return true;
                                    }
                                    let g_iso_name = Path::new(&g.iso)
                                        .file_name()
                                        .map(|s| s.to_string_lossy().to_lowercase())
                                        .unwrap_or_default();
                                    if !g_iso_name.is_empty() && g_iso_name == norm_file_lower {
                                        return true;
                                    }
                                    // Match base prefix (e.g. devil-may-cry-3 and devil-may-cry-3-dantes-awakening)
                                    let norm_g = g.name.replace('-', "");
                                    let norm_s = safe_name.replace('-', "");
                                    if norm_g.starts_with(&norm_s) || norm_s.starts_with(&norm_g) {
                                        return true;
                                    }
                                    false
                                });

                                if !already_exists {
                                    let abs_path = if path.is_absolute() {
                                        path.clone()
                                    } else {
                                        std::env::current_dir().unwrap_or_default().join(&path)
                                    };

                                    let exe = self
                                        .recomp_root
                                        .join("build")
                                        .join(format!("native-{}", safe_name))
                                        .join("ps2run.exe");

                                    let is_ready = exe.exists();
                                    let exe_path = if is_ready { Some(exe) } else { None };

                                    self.games.push(NativeGameInfo {
                                        name: safe_name.clone(),
                                        source: abs_path.to_string_lossy().to_string(),
                                        built: String::new(),
                                        took: String::new(),
                                        elf: format!("games/{}/disc", safe_name),
                                        iso: abs_path.to_string_lossy().to_string(),
                                        pnach: String::new(),
                                        sections: vec![],
                                        patches: String::new(),
                                        widescreen: "1".to_string(),
                                        playtime_seconds: 0,
                                        last_played: String::new(),
                                        is_ready,
                                        exe_path,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn update_build_logs(&mut self) {
        if let Some(ref rx) = self.build_receiver {
            while let Ok(line) = rx.try_recv() {
                if line.starts_with("__STAGE__:") {
                    let stage_name = line.strip_prefix("__STAGE__:").unwrap_or("Building");
                    self.current_stage = stage_name.to_string();
                } else if line.starts_with("__PROGRESS__:") {
                    if let Ok(p) = line.strip_prefix("__PROGRESS__:").unwrap_or("0").parse::<f32>() {
                        self.build_progress = p;
                    }
                } else if line == "__BUILD_SUCCESS__" {
                    self.is_building = false;
                    self.build_progress = 1.0;
                    self.current_stage = "Kompilasi Selesai (Siap Dimainkan!)".to_string();
                    self.log_lines.push_back("===> Build Finished Successfully!".to_string());
                    self.scan_games();

                    // Check if there are more games in the batch queue
                    if let Some((next_name, next_iso, next_ws)) = self.batch_queue.pop_front() {
                        self.log_lines.push_back(format!(">>> Memulai batch berikutnya (Sisa di antrean: {}): {}", self.batch_queue.len(), next_name));
                        self.start_native_build(&next_name, &next_iso, next_ws);
                    }
                    break;
                } else if line == "__BUILD_FAILED__" {
                    self.is_building = false;
                    self.current_stage = "Gagal (Lihat log di atas)".to_string();
                    self.log_lines.push_back("===> Build Failed! Check logs above.".to_string());

                    // Check if there are more games in the batch queue even if one fails
                    if let Some((next_name, next_iso, next_ws)) = self.batch_queue.pop_front() {
                        self.log_lines.push_back(format!(">>> Melanjutkan antrean setelah kegagalan (Sisa di antrean: {}): {}", self.batch_queue.len(), next_name));
                        self.start_native_build(&next_name, &next_iso, next_ws);
                    }
                    break;
                } else {
                    if self.log_lines.len() >= 1000 {
                        self.log_lines.pop_front();
                    }
                    self.log_lines.push_back(line);
                }
            }
        }
    }

    pub fn enqueue_build(&mut self, raw_name: &str, iso_path: &Path, widescreen: bool) {
        if !self.is_building {
            self.start_native_build(raw_name, iso_path, widescreen);
        } else {
            self.batch_queue.push_back((raw_name.to_string(), iso_path.to_path_buf(), widescreen));
            self.log_lines.push_back(format!(
                "[+] Ditambahkan ke Antrean Batch: '{}' (Posisi Antrean: {})",
                raw_name,
                self.batch_queue.len()
            ));
        }
    }

    pub fn enqueue_all_unbuilt(&mut self, widescreen: bool) -> usize {
        let mut count = 0;
        let unbuilt: Vec<(String, PathBuf)> = self
            .games
            .iter()
            .filter(|g| !g.is_ready)
            .map(|g| (g.name.clone(), PathBuf::from(&g.source)))
            .collect();

        for (name, path) in unbuilt {
            self.enqueue_build(&name, &path, widescreen);
            count += 1;
        }
        count
    }

    pub fn update_playtime(&mut self, game_name: &str, elapsed_seconds: u64) {
        let now_str = chrono_or_simple_timestamp();
        for game in &mut self.games {
            if game.name == game_name {
                game.playtime_seconds = game.playtime_seconds.saturating_add(elapsed_seconds);
                game.last_played = now_str.clone();

                // Persist to app.json if it exists
                let app_file = self.recomp_root.join("games").join(game_name).join("app.json");
                if app_file.exists() {
                    if let Ok(json) = serde_json::to_string_pretty(&game) {
                        let _ = fs::write(&app_file, json);
                    }
                }
                break;
            }
        }
    }

    pub fn start_native_build(&mut self, raw_name: &str, iso_path: &Path, widescreen: bool) {
        if self.is_building {
            return;
        }

        let safe_name = Self::sanitize_game_name(raw_name);
        self.is_building = true;
        self.build_progress = 0.05;
        self.current_stage = "Inisialisasi Toolchain...".to_string();
        self.log_lines.clear();
        self.log_lines.push_back(format!(
            "Target game: '{}' (Safe Identifier: '{}')",
            raw_name, safe_name
        ));

        let (tx, rx) = channel();
        self.build_receiver = Some(rx);

        let root = self.recomp_root.clone();
        let iso_buf = iso_path.to_path_buf();

        thread::spawn(move || {
            let tools_dir = root.join("tools").join("portable");
            let clang_probe = tools_dir.join("llvm-mingw").join("bin").join("clang++.exe");
            let clang_probe2 = tools_dir.join("bin").join("clang++.exe");

            // -----------------------------------------------------------------
            // STAGE 1: Check and download portable build tools if missing
            // -----------------------------------------------------------------
            if !clang_probe.exists() && !clang_probe2.exists() {
                let _ = tx.send("__STAGE__:Mengunduh Portable Build Tools (LLVM-MinGW, CMake, Ninja, Python)...".to_string());
                let _ = tx.send("__PROGRESS__:0.1".to_string());
                let _ = tx.send("[Stage 1/3] Downloading official toolchain into tools/portable...".to_string());

                let script_path = root.join("scripts").join("portable.ps1");
                let mut ps_cmd = Command::new("powershell.exe");
                ps_cmd.current_dir(&root);
                ps_cmd.args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    &script_path.to_string_lossy(),
                    "-ForApp",
                ]);

                ps_cmd.stdout(Stdio::piped());
                ps_cmd.stderr(Stdio::piped());

                if let Ok(mut child) = ps_cmd.spawn() {
                    let stdout = child.stdout.take().unwrap();
                    let tx_out = tx.clone();
                    thread::spawn(move || {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().flatten() {
                            let _ = tx_out.send(line);
                        }
                    });

                    let stderr = child.stderr.take().unwrap();
                    let tx_err = tx.clone();
                    thread::spawn(move || {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines().flatten() {
                            let _ = tx_err.send(format!("[STDERR] {}", line));
                        }
                    });

                    let status = child.wait();
                    if let Ok(exit) = status {
                        if !exit.success() {
                            let _ = tx.send("Gagal mengunduh portable tools!".to_string());
                            let _ = tx.send("__BUILD_FAILED__".to_string());
                            return;
                        }
                    } else {
                        let _ = tx.send("__BUILD_FAILED__".to_string());
                        return;
                    }
                } else {
                    let _ = tx.send("Gagal menjalankan powershell.exe untuk scripts/portable.ps1".to_string());
                    let _ = tx.send("__BUILD_FAILED__".to_string());
                    return;
                }
            }

            // -----------------------------------------------------------------
            // Find Git Bash on system or in portable tools
            // -----------------------------------------------------------------
            let bash_candidates = [
                tools_dir.join("mingit").join("usr").join("bin").join("bash.exe"),
                PathBuf::from(r"C:\Program Files\Git\bin\bash.exe"),
                PathBuf::from(r"C:\Program Files\Git\usr\bin\bash.exe"),
            ];

            let mut bash_exe = None;
            for candidate in &bash_candidates {
                if candidate.exists() {
                    bash_exe = Some(candidate.clone());
                    break;
                }
            }

            let bash = match bash_exe {
                Some(b) => b,
                None => {
                    let _ = tx.send("Error: Git Bash (bash.exe) tidak ditemukan! Pastikan Git for Windows terinstal.".to_string());
                    let _ = tx.send("__BUILD_FAILED__".to_string());
                    return;
                }
            };

            let _ = tx.send(format!("Menggunakan Bash: {}", bash.display()));

            // Build PATH environment string containing portable tools
            let mut paths = Vec::new();
            paths.push(tools_dir.join("bin"));
            paths.push(tools_dir.join("llvm-mingw").join("bin"));
            paths.push(tools_dir.join("cmake").join("bin"));
            paths.push(tools_dir.join("ninja"));
            paths.push(tools_dir.join("python"));
            paths.push(tools_dir.join("mingit").join("usr").join("bin"));
            if let Some(parent) = bash.parent() {
                paths.push(parent.to_path_buf());
            }
            if let Ok(current_path) = std::env::var("PATH") {
                paths.push(PathBuf::from(current_path));
            }
            let new_path_env = std::env::join_paths(paths).unwrap_or_default();

            // -----------------------------------------------------------------
            // STAGE 2: Setup graphics & window libraries (setup.sh)
            // -----------------------------------------------------------------
            let setup_probe = root.join("runtime").join("wgpu");
            if !setup_probe.exists() {
                let _ = tx.send("__STAGE__:Menyiapkan Translator & Library WebGPU/SDL2...".to_string());
                let _ = tx.send("__PROGRESS__:0.3".to_string());
                let _ = tx.send("[Stage 2/3] Running scripts/setup.sh --native-only...".to_string());

                let mut setup_cmd = Command::new(&bash);
                setup_cmd.current_dir(&root);
                setup_cmd.env("PATH", &new_path_env);
                setup_cmd.arg("scripts/setup.sh");
                setup_cmd.arg("--native-only");

                setup_cmd.stdout(Stdio::piped());
                setup_cmd.stderr(Stdio::piped());

                if let Ok(mut child) = setup_cmd.spawn() {
                    let stdout = child.stdout.take().unwrap();
                    let tx_out = tx.clone();
                    thread::spawn(move || {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().flatten() {
                            let _ = tx_out.send(line);
                        }
                    });

                    let stderr = child.stderr.take().unwrap();
                    let tx_err = tx.clone();
                    thread::spawn(move || {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines().flatten() {
                            let _ = tx_err.send(format!("[STDERR] {}", line));
                        }
                    });

                    let _ = child.wait();
                }
            }

            // -----------------------------------------------------------------
            // STAGE 3: Port Game AOT Recompile (direct mingit bash with portable PATH)
            // -----------------------------------------------------------------
            let _ = tx.send("__STAGE__:Menerjemahkan Kode Mesin PS2 (MIPS ke C++20 & Kompilasi Clang)...".to_string());
            let _ = tx.send("__PROGRESS__:0.5".to_string());
            let _ = tx.send(format!("[Stage 3/3] Compiling native binary for '{}'...", safe_name));

            let tools_dir = root.join("tools").join("portable");
            let bash_exe = tools_dir.join("mingit").join("usr").join("bin").join("bash.exe");
            let python_exe = tools_dir.join("python").join("python.exe");

            // Build isolated portable PATH exactly as in env.ps1
            let portable_path = format!(
                "{};{};{};{};{};{};{};{}",
                tools_dir.join("llvm-mingw").join("bin").to_string_lossy(),
                tools_dir.join("cmake").join("bin").to_string_lossy(),
                tools_dir.join("ninja").to_string_lossy(),
                tools_dir.join("python").to_string_lossy(),
                tools_dir.join("mingit").join("cmd").to_string_lossy(),
                tools_dir.join("mingit").join("usr").join("bin").to_string_lossy(),
                tools_dir.join("busybox").to_string_lossy(),
                std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_string()) + "\\System32",
            );

            let game_dir = root.join("games").join(&safe_name);
            let disc_iso = game_dir.join("disc.iso");
            let _ = fs::create_dir_all(&game_dir);

            // Resolve absolute path to the ISO file
            let abs_iso = if iso_buf.is_absolute() {
                iso_buf.clone()
            } else {
                let cur = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let cand1 = cur.join(&iso_buf);
                if cand1.exists() {
                    cand1
                } else {
                    let cand2 = root.join(&iso_buf);
                    if cand2.exists() {
                        cand2
                    } else {
                        cand1
                    }
                }
            };

            // Link or copy disc.iso into games/<safe_name>/disc.iso so port.sh & pnach.py never fail on relative paths or special chars
            if !disc_iso.exists() && abs_iso.exists() {
                let _ = tx.send(format!("[Setup] Menautkan berkas disc.iso untuk '{}'...", safe_name));
                if fs::hard_link(&abs_iso, &disc_iso).is_err() {
                    let _ = fs::copy(&abs_iso, &disc_iso);
                }
            }

            let disc_posix = format!("games/{}/disc.iso", safe_name);

            // Auto-fetch PNACH patch if widescreen is enabled
            if widescreen {
                let _ = tx.send("[PNACH] Checking PCSX2 patch database for Widescreen 16:9 patch...".to_string());
                let mut pnach_cmd = Command::new(&python_exe);
                pnach_cmd.current_dir(&root);
                pnach_cmd.env("Path", &portable_path);
                pnach_cmd.args([
                    "scripts/pnach.py",
                    "fetch",
                    &disc_posix,
                    "--out",
                    &format!("games/{}", safe_name),
                ]);
                let _ = pnach_cmd.status();
            }

            // Look for fetched pnach file in game dir
            let mut pnach_args = Vec::new();
            if let Ok(entries) = fs::read_dir(&game_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "pnach" {
                            let pnach_posix = path.to_string_lossy().replace('\\', "/");
                            pnach_args.push("--pnach".to_string());
                            pnach_args.push(pnach_posix);
                            pnach_args.push("--pnach-section".to_string());
                            pnach_args.push("Widescreen 16:9".to_string());
                            break;
                        }
                    }
                }
            }

            let mut port_cmd = Command::new(&bash_exe);
            port_cmd.current_dir(&root);
            port_cmd.env("Path", &portable_path);
            port_cmd.arg("scripts/port.sh");
            port_cmd.arg(&safe_name);
            port_cmd.arg(&disc_posix);
            port_cmd.arg("--native-only");
            if widescreen {
                port_cmd.arg("--widescreen");
            }
            for p_arg in pnach_args {
                port_cmd.arg(p_arg);
            }

            port_cmd.stdout(Stdio::piped());
            port_cmd.stderr(Stdio::piped());

            match port_cmd.spawn() {
                Ok(mut child) => {
                    let stdout = child.stdout.take().unwrap();
                    let tx_out = tx.clone();
                    thread::spawn(move || {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().flatten() {
                            let _ = tx_out.send(line);
                        }
                    });

                    let stderr = child.stderr.take().unwrap();
                    let tx_err = tx.clone();
                    thread::spawn(move || {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines().flatten() {
                            let _ = tx_err.send(format!("[STDERR] {}", line));
                        }
                    });

                    let status = child.wait();
                    if let Ok(exit) = status {
                        if exit.success() {
                            // Ensure app.json exists with widescreen metadata
                            let game_folder = root.join("games").join(&safe_name);
                            let app_file = game_folder.join("app.json");
                            if !app_file.exists() {
                                let wide_str = if widescreen { "1" } else { "" };
                                let info = NativeGameInfo {
                                    name: safe_name.clone(),
                                    source: iso_buf.to_string_lossy().to_string(),
                                    built: "2026-11-08".to_string(),
                                    took: "180".to_string(),
                                    elf: format!("games/{}/disc/{}", safe_name, "SLES_544.90"),
                                    iso: format!("games/{}/disc.iso", safe_name),
                                    pnach: "".to_string(),
                                    sections: vec![],
                                    patches: "".to_string(),
                                    widescreen: wide_str.to_string(),
                                    playtime_seconds: 0,
                                    last_played: String::new(),
                                    is_ready: true,
                                    exe_path: None,
                                };
                                if let Ok(json) = serde_json::to_string_pretty(&info) {
                                    let _ = fs::write(&app_file, json);
                                }
                            }

                            let _ = tx.send("__BUILD_SUCCESS__".to_string());
                        } else {
                            let _ = tx.send("__BUILD_FAILED__".to_string());
                        }
                    } else {
                        let _ = tx.send("__BUILD_FAILED__".to_string());
                    }
                }
                Err(err) => {
                    let _ = tx.send(format!("Failed to spawn bash: {}", err));
                    let _ = tx.send("__BUILD_FAILED__".to_string());
                }
            }
        });
    }

    pub fn sanitize_game_name(raw: &str) -> String {
        let stem = Path::new(raw)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(raw);

        // 1. Remove parentheses, brackets, e.g. (USA), [SLUS-12345], (Europe) (En,Ja,Ko)
        let mut cleaned = String::new();
        let mut in_paren: usize = 0;
        let mut in_bracket: usize = 0;
        for ch in stem.chars() {
            match ch {
                '(' => in_paren += 1,
                ')' => in_paren = in_paren.saturating_sub(1),
                '[' => in_bracket += 1,
                ']' => in_bracket = in_bracket.saturating_sub(1),
                _ => {
                    if in_paren == 0 && in_bracket == 0 {
                        cleaned.push(ch);
                    }
                }
            }
        }

        // 2. Normalize special punctuation:
        //    - Colons (:), semicolons (;) -> convert to dash
        //    - Dashes (hyphen -, en-dash –, em-dash —) -> convert to dash
        //    - Ampersand (&) -> convert to 'and'
        //    - Apostrophes (' and ’) -> omit (e.g. Dante's -> dantes)
        let mut safe = String::new();
        let mut prev_dash = false;
        for ch in cleaned.chars() {
            if ch == '\'' || ch == '’' || ch == '`' {
                // Ignore apostrophes
                continue;
            }
            if ch == '&' {
                if !prev_dash && !safe.is_empty() {
                    safe.push('-');
                }
                safe.push_str("and");
                safe.push('-');
                prev_dash = true;
                continue;
            }
            if ch.is_ascii_alphanumeric() {
                safe.push(ch.to_ascii_lowercase());
                prev_dash = false;
            } else if ch == '-' || ch == '–' || ch == '—' || ch == ':' || ch == ';' || ch == '_' || ch == '.' || ch.is_whitespace() {
                if !prev_dash && !safe.is_empty() {
                    safe.push('-');
                    prev_dash = true;
                }
            }
        }

        let trimmed = safe.trim_matches('-').to_string();
        if trimmed.is_empty() {
            "game".to_string()
        } else if trimmed.len() > 50 {
            trimmed[..50].trim_matches('-').to_string()
        } else {
            trimmed
        }
    }

    pub fn launch_native_game(
        &self,
        game: &NativeGameInfo,
        glyph_style: &str,
        runtime_config: &crate::config::GameRuntimeConfig,
    ) -> Result<std::process::Child, std::io::Error> {
        let exe = match game.exe_path {
            Some(ref e) => e,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Executable ps2run.exe not found. Build game first.",
                ))
            }
        };

        let mut cmd = Command::new(exe);
        cmd.current_dir(&self.recomp_root);

        // Ensure gamecontrollerdb.txt exists next to ps2run.exe for generic HID / DirectInput controllers
        if let Some(exe_dir) = exe.parent() {
            let target_db = exe_dir.join("gamecontrollerdb.txt");
            if !target_db.exists() {
                let db_candidates = [
                    PathBuf::from("gamecontrollerdb.txt"),
                    PathBuf::from("assets/gamecontrollerdb.txt"),
                    self.recomp_root.join("gamecontrollerdb.txt"),
                ];
                for cand in &db_candidates {
                    if cand.exists() {
                        let _ = fs::copy(cand, &target_db);
                        break;
                    }
                }
            }
        }

        let db_abs = std::env::current_dir()
            .unwrap_or_default()
            .join("gamecontrollerdb.txt");
        if db_abs.exists() {
            cmd.env("SDL_GAMECONTROLLERCONFIG_FILE", &db_abs);
        }

        let elf_path = self.recomp_root.join(&game.elf);
        let iso_path = self.recomp_root.join(&game.iso);
        let game_dir = self.recomp_root.join("games").join(&game.name);
        let memcard_path = game_dir.join(format!("{}.ps2", game.name));

        // Save current spartanport.ini & recomp.ini to game directory
        let ini_path = game_dir.join("spartanport.ini");
        let _ = runtime_config.save_to_ini(&ini_path);

        cmd.arg(&elf_path);
        cmd.arg("--disc").arg(&iso_path);
        cmd.arg("--game-dir").arg(&game_dir);
        
        if runtime_config.fullscreen {
            cmd.arg("--fullscreen");
        } else {
            cmd.arg("--window");
        }

        cmd.arg("--memcard").arg(&memcard_path);

        if runtime_config.widescreen_16_9 || !game.widescreen.is_empty() {
            cmd.arg("--widescreen");
        }

        if !game.patches.is_empty() {
            let patches_path = self.recomp_root.join(&game.patches);
            cmd.arg("--patches").arg(&patches_path);
        }

        // Set explicit window title so game window is branded DivinePort
        let display_title = game.name.replace('-', " ").to_uppercase();
        cmd.env("PS2_WINDOW_TITLE", format!("DivinePort: {}", display_title));

        // SpartanPort fidelityfx environment variables
        cmd.env("SPARTAN_GLYPH_STYLE", glyph_style);
        cmd.env("PS2_SCALE", runtime_config.render_scale.to_string());
        cmd.env("SPARTAN_UPSCALER", runtime_config.upscaler.as_str());
        cmd.env("SPARTAN_UPSCALE_PRESET", runtime_config.upscale_preset.as_code());
        cmd.env("SPARTAN_STREAM_MEMORY", &runtime_config.stream_memory);
        cmd.env("SPARTAN_RCAS", format!("{:.2}", runtime_config.rcas_sharpening));
        if runtime_config.msaa_mode != "auto" {
            cmd.env("SPARTAN_NO_MSAA", if runtime_config.msaa_mode == "off" { "1" } else { "0" });
        }

        // Graphics API selection (Vulkan core dramatically reduces presentation hitching on GeForce/Radeon)
        cmd.env("PS2_GPU_BACKEND", &runtime_config.graphics_backend);

        // Audio Timing & Stretching Pipeline:
        // When async_audio is enabled, disable audio stretching (PS2_AUDIO_STRETCH=0)
        // so the audio engine runs in real-time without locking the game into stretched x2.0 (30 FPS drop).
        if runtime_config.async_audio {
            cmd.env("PS2_AUDIO_STRETCH", "0");
        } else {
            cmd.env("PS2_AUDIO_STRETCH", "1");
        }

        // Live JIT Vector Unit (VU1) Background Compiler Optimization:
        // Set to low priority so background Clang JIT never steals CPU cycles from active 60 FPS gameplay,
        // and cap compiler worker jobs to 2 threads to prevent CPU load spikes.
        cmd.env("PS2_VU_LIVE_PRIORITY", "low");
        cmd.env("PS2_VU_LIVE_JOBS", "2");
        cmd.env("PS2_VU_LIVE_OPT", "-O2");
        cmd.env("PS2_GPU_LOW_POWER", "0");

        // SpartanPort 120 FPS display with locked 60 FPS PS2 game simulation
        if runtime_config.lock_game_60_display_120 {
            cmd.env("SPARTAN_FRAME_GEN", "fsr2x");
            cmd.env("SPARTAN_DISPLAY_FPS", "120");
            cmd.env("SPARTAN_SIMULATION_LOCK", "60");
            cmd.env("PS2_TARGET_FPS", "120");
            cmd.env("PS2_GAME_LOCK_FPS", "60");
        } else {
            cmd.env("SPARTAN_FRAME_GEN", runtime_config.frame_generation.as_str());
        }

        // Presentation mode: Use mailbox or immediate so present() is non-blocking and
        // pace() accurately controls the exact 60.0 FPS frame intervals without double VSync stalls.
        cmd.env("PS2_PRESENT_MODE", "mailbox");

        // Windows Thread Director Optimization: Elevate to ABOVE_NORMAL_PRIORITY_CLASS (0x00008000)
        // Ensures smooth scheduling onto Intel P-Cores without choking the system audio daemon
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x00008000);
        }

        cmd.spawn()
    }
}

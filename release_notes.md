# DivinePort v1.0.0 - Initial Release

**DivinePort** adalah tool dan launcher Native AOT (Ahead-Of-Time) Static Recompiler untuk game Sony PlayStation 2 ke biner Windows x86_64 murni tanpa emulasi VM tradisional.

---

### Fitur Utama

- **Native Binary Execution**: Menghasilkan biner C++20 murni x86_64 dari file ELF/ISO PS2, memberikan performa tinggi 60 FPS terkunci dengan latensi frametime mendekati nol.
- **Hybrid UI Architecture**: Memadukan antarmuka presisi SpartanPort dengan sistem manajemen pustaka dan navigasi terstruktur gaya PCSX2.
- **Pustaka Upscaling Mutakhir**:
  - AMD FidelityFX Super Resolution (FSR 3.1 & FSR 4.1 Vulkan runtime)
  - NVIDIA DLSS & Intel XeSS
  - Temporal Anti-Aliasing (TAA)
  - AMD FSR Frame Generation (2x Fluid Motion Frames)
  - Contrast Adaptive Sharpening (RCAS)
- **Universal ROM & Title Normalizer**: Mendukung semua penamaan ISO ROM (tanda hubung, titik dua, tanda petik satu, kurung wilayah, kode seri SLUS/SLES/SCES).
- **Dual Visual Presentation**: Mode Table View bergaya PCSX2 untuk informasi teknis lengkap dan mode Card View bergaya SpartanPort dengan cover art interaktif.
- **Widescreen 16:9 & Fullscreen Support**: Resolusi layar lebar terkalibrasi tanpa distorsi rasio aspek.
- **Full Controller & Keyboard Navigation**: Mendukung gamepad Xbox, PlayStation DualShock/DualSense, dan navigasi keyboard menyeluruh.

---

### Cara Penggunaan

1. Ekstrak file arsip `DivinePort-v1.0.0-windows-x64.zip`.
2. Jalankan `DivinePort.exe` (atau melalui `Launch.cmd`).
3. Pilih ISO game PS2 Anda atau letakkan di direktori `games/`.
4. Klik **Recompile AOT** untuk membangun biner native, lalu klik **Play Native**!

---

### Dukungan & Donasi

Jika Anda menikmati DivinePort dan ingin mendukung kelanjutan pengembangan fitur baru:
- **Website Support**: [https://supportzhou.vercel.app](https://supportzhou.vercel.app)

---

### Lisensi

Didistribusikan di bawah lisensi ganda:
- **MIT License**
- **GNU General Public License v3.0 or later**

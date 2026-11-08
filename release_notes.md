# DivinePort v1.1.0 - Performance & Visual Update

Pembaruan minor versi **v1.1.0** membawa optimasi performa besar adaptasi dari PCSX2 dan SpartanPort, sistem box art otomatis, dan eliminasi bottleneck thread.

---

### Pembaruan di Versi 1.1.0

1. **Adaptasi SpartanPort 120 FPS Mode**:
   - Membuka swapchain display rendering hingga **120 FPS** (via AMD FSR 3.1 Frame Generation 2x).
   - Mengunci simulasi logika internal game PS2 tetap di **60.0 FPS presisi** (mencegah game bergerak terlalu cepat/liar).

2. **Adaptasi PCSX2 SPU2 Async Mix**:
   - Memisahkan clock buffer audio dari loop emulasi video (`PS2_AUDIO_STRETCH=0`).
   - Mengeliminasi bug perlambatan 14.5 FPS yang disebabkan oleh latency buffer audio virtual (seperti SteelSeries Sonar / Voicemeeter).

3. **Migrasi Default Vulkan Core**:
   - Beralih ke backend grafis Vulkan (`PS2_GPU_BACKEND=vulkan`) untuk menghilangkan stuttering presentasi 500ms yang terjadi pada Direct3D 12 di GPU laptop NVIDIA RTX dan AMD Radeon.

4. **Optimasi Thread Director Intel P-Cores**:
   - Menyuntikkan flag `ABOVE_NORMAL_PRIORITY_CLASS` pada eksekusi proses native.
   - Memaksa penjadwal Windows 11 mengunci thread simulasi dan vektor (VU1) ke core kencang (P-Cores) berkecepatan 4.4 GHz dan mencegah thread terlempar ke inti hemat daya (E-Cores 1.8 GHz).

5. **Pustaka Box Art Cover 3D/2D Otomatis**:
   - Menampilkan poster cover art vertikal resolusi tinggi di mode Card Grid dan Table View.
   - Terintegrasi dengan repositori resmi PCSX2/AetherSX2 (`xlenore/ps2-covers`).
   - Tombol toolbar baru: `[^] Unduh Box Art` untuk mengunduh seluruh sampul game secara otomatis di latar belakang.

6. **Pemeriksaan Pembaruan (Check for Updates)**:
   - Tombol baru di menu Help untuk memeriksa rilis terbaru secara langsung di GitHub.

7. **Penyimpanan Konfigurasi Otomatis (Instant Auto-Save)**:
   - Setiap perubahan setelan grafis, resolusi, kontrol, dan glif langsung tersimpan ke disk seketika tanpa perlu menekan tombol simpan manual.

---

### Dukungan & Donasi

Jika Anda menikmati DivinePort dan ingin mendukung kelanjutan pengembangan fitur baru:
- **Website Support**: [https://supportzhou.vercel.app](https://supportzhou.vercel.app)

---

### Lisensi

Didistribusikan di bawah lisensi ganda:
- **MIT License**
- **GNU General Public License v3.0 or later**

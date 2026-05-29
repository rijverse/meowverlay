# Meowverlay 🐱

An open-source, secure, and high-performance Bongo Cat overlay for gaming, heavily inspired by the original `bongocat-osu`. Built with **Rust (Tauri)** for system-level input capturing and **HTML5 Canvas (TypeScript/CSS)** for fluid, hardware-accelerated animations.

Unlike traditional overlays, Meowverlay uses a strict file-loading sandbox, native global input hooks, and a beautiful auto-hiding glassmorphic control bar.

---

## 🛠️ Prerequisites

Before running or compiling the project, make sure you have installed the X11 and webkit developmental dependencies for your OS.

### Ubuntu / Debian:
```bash
sudo apt-get update && sudo apt-get install -y libx11-dev libxtst-dev libwebkit2gtk-4.0-dev build-essential libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

### Arch Linux:
```bash
sudo pacman -Syu xorg-server-devel libxtst webkit2gtk gtk3 libayatana-appindicator librsvg
```

---

## 🚀 How to Run

Since the runtime environment uses **Bun**, you can launch the overlay in development mode directly:

1. Install frontend packages:
   ```bash
   bun install
   ```
2. Start the Tauri application:
   ```bash
   bun run tauri dev
   ```

This starts the dev server and launches a transparent, borderless overlay window floating on your screen.

---

## 🎨 Adding Skins

Custom skins are located in the `skins/` folder in the project root:

```text
skins/
└── default/
    ├── config.json         # Keybindings and mouse mapping
    └── img/
        ├── osu/            # Standard mode sprites
        ├── taiko/          # Taiko mode sprites
        ├── catch/          # Catch the Beat sprites
        └── mania/          # Mania (4K/7K) mode sprites
```

### Skin Drop-in Compatibility
The folder architecture is designed to be **100% compatible** with standard `bongocat-osu` skins. You can download any existing Bongo Cat skin and extract its contents directly into `skins/<skin_name>/` to load it in Meowverlay.

---

## ⚙️ How it Works & Features

1. **Draggable Transparent Window:** When you hover over the overlay, a semi-transparent, glassmorphic menu bar slides down at the top. You can click and drag the window from this handle or use it to switch skins/modes.
2. **Global Input Interceptor:** The Rust engine uses `rdev` globally to track keyboard presses and mouse movement without locking the main thread.
3. **Throttled Coordinates:** To prevent overloading the IPC bridge, mouse movement events are rate-limited in Rust to a smooth ~125Hz.
4. **Interactive Hotkey:** Press `Ctrl + R` while the overlay window is focused to hot-reload skins and configs instantly.

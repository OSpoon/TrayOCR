# TrayOCR

> A minimalist, menu-bar-based screenshot OCR utility for macOS and Windows. Powered by Tauri 2, Vue 3, and native system OCR.

<p align="center">
  <img src="src-tauri/icons/128x128.png" width="128" height="128" alt="TrayOCR Logo" />
</p>

TrayOCR is a lightweight and blazing-fast desktop utility that lives in your system tray or macOS menu bar. Press a custom hotkey to select a screen area, extract text instantly, and automatically copy it to your clipboard.

---

## 🌟 Key Features

- **Menu-Bar / Tray POP UI**: Designed to stay out of your way. Clicking the tray icon opens a sleek popover window; the app runs in agent mode without taking space in your Dock.
- **Native System OCR**: Uses Apple's Vision framework on macOS and Windows.Media.Ocr on Windows.
- **Flexible Global Shortcuts**: Set your own hotkeys (e.g. `⌘⌥X` or `⌥A`) in the UI to trigger screen area capture globally.
- **Click-to-Copy History**: Stores up to 200 historical OCR scans locally with timestamp logs. Just click any card to copy the text to your clipboard.
- **Launch at Login & Window Pinning**: Toggle the app to launch automatically when you sign in, and pin the window to stay "Always on Top" when needed.
- **Harmonious Dark & Light Mode**: Automatically matches your system preferences or lets you lock it to Dark/Light mode. Styled with modern typography and sleek glassmorphism themes.
- **Safe OTA Updates**: Fully integrated with Tauri's secure signed auto-updater to notify you and install updates smoothly.

---

## 🛠 Tech Stack

- **Frontend**: [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) + [Vite 6](https://vite.dev/)
- **Styling & UI**: [Tailwind CSS v4](https://tailwindcss.com/) + [shadcn-vue](https://shadcn-vue.com/) (using native `Dialog`, `Button`, `Sonner` toasts, and `Progress` components)
- **Backend (Core)**: [Tauri v2](https://tauri.app/) (Rust-based system bridging)
- **OCR Engine**: macOS native `Vision` framework bindings and Windows native `Windows.Media.Ocr`.
- **Linting & Formatting**: [ESLint](https://eslint.org/) + [simple-git-hooks](https://github.com/toplenboren/simple-git-hooks) + [lint-staged](https://github.com/lint-staged/lint-staged)

---

## 🚀 Getting Started

### Installation
1. Go to the [Releases](https://github.com/OSpoon/tray-ocr-app/releases) page.
2. Download the installer for your system:
   - **`TrayOCR_x.x.x_aarch64.dmg`** for Apple Silicon Macs (M1, M2, M3, M4, etc.)
   - **`TrayOCR_x.x.x_x64.dmg`** for Intel Macs
   - **`TrayOCR_x.x.x_x64-setup.exe`** or `.msi` for Windows
3. On macOS, drag the app into your `Applications` folder. On Windows, run the installer and launch TrayOCR from the Start menu.

> [!WARNING]
> **Screen Recording Permission required on macOS**: Since TrayOCR uses the native screen capture tool (`screencapture -i`) to let you draw a box over target screen contents, macOS will request **Screen Recording** permission the first time you initiate a scan. Please go to **System Settings -> Privacy & Security -> Screen & System Audio Recording** and toggle **TrayOCR** to **ON** to ensure screenshots capture successfully.

## 💻 Local Development

### 1) Prerequisites
Ensure you have Rust, Node.js (v18+), and `pnpm` installed on your macOS machine. Refer to the [Tauri setup guide](https://tauri.app/v2/start/prerequisites/) if needed.

### 2) Install Dependencies
```bash
pnpm install
```

### 3) Run Dev Server
```bash
pnpm tauri dev
```

### 4) Production Build
Compile optimized production bundles:
```bash
pnpm tauri build
```
This runs type-checking (`vue-tsc`), packages assets with Vite, and cross-compiles targets using Cargo.

---

## 🔄 Release & Auto-Update Configuration

TrayOCR comes with a preconfigured GitHub Action release pipeline (`.github/workflows/release.yaml`) that builds signed, production-ready installers for Apple Silicon macOS, Intel macOS, and Windows.

### 1) Generate Updater Keys (Required once)
Tauri requires signing key pairs to authenticate update payloads:
```bash
pnpm tauri signer generate -w ./.tauri/updater.key
```
This creates:
- **Private Key**: `.tauri/updater.key` (⚠️ **Never commit this to git**)
- **Public Key**: `.tauri/updater.key.pub` (Safe to share)

### 2) Save Public Key
Copy the string contents of `.tauri/updater.key.pub` and write it to the `plugins.updater.pubkey` field in [tauri.conf.json](file:///Users/osp/Documents/GitHub/TrayOCR/src-tauri/tauri.conf.json).

### 3) Set Up GitHub Secrets
In your GitHub repository, go to `Settings -> Secrets and variables -> Actions` and add:
- `TAURI_SIGNING_PRIVATE_KEY` (Paste the contents of `.tauri/updater.key`)
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (If you configured a password when generating the key; otherwise leave empty/create empty)

Under `Settings -> Actions -> General -> Workflow permissions`, set permissions to **Read and write permissions** to allow workflow jobs to create releases.

### 4) Release Version Bump
To release a new version, run:
```bash
pnpm release
```
This triggers `bumpp` to bump version logs across `package.json`, `Cargo.toml`, and `tauri.conf.json`, commits files, creates a version Tag (`vX.Y.Z`), and pushes to GitHub. The workflow will catch the tag, build Apple Silicon (`aarch64-apple-darwin`), Intel macOS (`x86_64-apple-darwin`), and Windows (`x86_64-pc-windows-msvc`) packages, and publish updates automatically.

---

## 📄 License
Licensed under the [MIT License](LICENSE).

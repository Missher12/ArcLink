# ArcLink Remote 🖥️

A modern, high-performance, native Windows remote control system built in Rust. It implements a fully decoupled architecture consisting of **ArcLink Host** (be controlled) and **ArcLink Viewer** (controller).

Designed strictly in **Apple Liquid Glass / Windows 11 Mica** visual style: clean light themes, semi-transparent frosted panels, and elegant typography.

---

## 📂 Project Structure

- `crates/arclink-common`: Protocol definitions, input/keyboard event models, and state enums.
- `crates/arclink-host`: The native Windows Host app (includes Desktop Capture & Input Injector).
- `crates/arclink-viewer`: The native Windows Controller client (includes Rendering & Input Capture).
- `crates/arclink-protocol-test`: Rust unit and integration test suites for loops and codecs.
- `docs/`: In-depth developer manuals covering ARCHITECTURE, PROTOCOLS, PERFORMANCE and ROADMAP.

---

## 🛠️ Windows Environment Prerequisites

To compile the native Windows rust apps:

1. Install **Rust** (stable toolchain, version 1.75+):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. Install **Build Tools for Visual Studio 2022** (needed for Microsoft C++ compilation targets):
   - Select "Desktop development with C++" workload during VS Installer setup.

---

## 🚀 Native Compilation & Execution

In a Windows Command Prompt, PowerShell, or bash terminal:

### 1. Build Workspace
```bash
cargo build --release
```

### 2. Run Host App
```bash
cargo run --bin arclink-host
```

### 3. Run Viewer App
```bash
cargo run --bin arclink-viewer
```

### 4. Run Test Suites
```bash
cargo test -p arclink-protocol-test
```

---

## 💻 Interactive Full-Stack Web Playground

This repository features a fully interactive web preview simulating both **ArcLink Host** and **ArcLink Viewer** in real-time. It operates over a real full-stack node/express server so that you can open two browser tabs to connect them and experience the remote control inputs directly!

### Running the Web Server
1. Install dependencies:
   ```bash
   npm install
   ```
2. Boot dev environment:
   ```bash
   npm run dev
   ```
3. Open `http://localhost:3000` in your browser.

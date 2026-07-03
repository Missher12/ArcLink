# ArcLink Remote 🖥️

This repository contains two parts:

1. `arclink-remote/` - **Native Rust Product (正式原生产品)**
   * This is the core product, organized as a Cargo Workspace containing `arclink-common`, `arclink-host`, `arclink-viewer`, `arclink-protocol-test`, and comprehensive `docs/`.
   * It implements the true remote desktop transport, input control injection, low-latency screen capture, and performance diagnostics.
   * **Note:** All actual production development, testing, and deployment must focus strictly on this directory.

2. Repository Root (Web Interface) - **Web Playground / UI Prototype (演示/原型层)**
   * This consists of the root `src/`, `server.ts`, and `package.json`.
   * It serves strictly as an interactive web-based playground and high-fidelity prototype to demonstrate the UI flow and layout.
   * **Note:** The web-based system **cannot** be used as the basis for actual low-latency remote desktop transmission, input injection, native screen capture, or hardware-accelerated encoding.

For detailed compilation and run instructions of the native Rust product, please see:
[**`arclink-remote/README.md`**](./arclink-remote/README.md)

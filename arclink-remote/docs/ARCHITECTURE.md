# ArcLink Remote: MVP Architecture

ArcLink Remote is designed as a modular, low-latency Windows-native remote control software stack written in Rust.

## Process Architecture

The system splits functionality into two independent standalone binaries:

```
+--------------------------+               +--------------------------+
|      ArcLink Viewer      |               |       ArcLink Host       |
|    (Controller Client)   |               |     (Controlled Device)  |
|                          |               |                          |
|  +--------------------+  |  TCP Control  |  +--------------------+  |
|  |     egui GUI       |  |<=============>|  |     egui GUI       |  |
|  +--------------------+  |               |  +--------------------+  |
|                          |               |                          |
|  +--------------------+  |  UDP Video    |  +--------------------+  |
|  |  Video Frame Recv  |  |<--------------|  | Screen Capturer    |  |
|  |  & Hardware Dec    |  |  (Low Latency)|  | & Encoder          |  |
|  +--------------------+  |               |  +--------------------+  |
|                          |               |                          |
|  +--------------------+  |  UDP Input    |  +--------------------+  |
|  | Mouse/Key Listener |  |-------------->|  | OS Input Injector  |  |
|  +--------------------+  | (Prioritized) |  +--------------------+  |
+--------------------------+               +--------------------------+
```

## Core Modules

### 1. ArcLink Host
* **Network Listener**: Runs a Tokio TCP and UDP listening service. TCP coordinates connections, handshakes, heartbeats, and parameter sync. UDP receives mouse and keyboard input packets.
* **Screen Capturer**: Interacts with the Windows Desktop Duplication API (DXGI) or Windows Graphics Capture to fetch hardware-accelerated video frames in less than 8ms.
* **Input Injector**: Maps coordinates sent from the Viewer and triggers low-level Windows `SendInput` APIs to simulate physical keyboard and mouse gestures.
* **Session Manager**: Manages current connection handshakes and implements security rules (e.g., host-side pop-up consent dialog before granting control).

### 2. ArcLink Viewer
* **Connection GUI**: Apple Liquid Glass light aesthetic where users input target Host's IP and port.
* **Interactive Canvas**: Renders incoming remote video frames using wgpu-backed high-performance graphics pipeline, tracking layout boundaries to correctly translate cursor coordinates.
* **Input Capture Hook**: Standard egui pointer/keyboard hooks mapping physical events into logical `InputEvent` packets sent over UDP channel to minimize movement latency.
* **Metrics Monitor**: Real-time sliding window telemetry logging frame rates, network round-trips, jitter, and bandwidth usage.

---

## Network Connection Flow (LAN Direct)

1. **Host Listening**: ArcLink Host starts a TCP server on port `8443` and waits.
2. **Viewer Handshake**: ArcLink Viewer connects to Host IP on port `8443` over TCP and issues a `SessionRequest`.
3. **Host Confirmation**: Host app suspends handshake and pops up a native glass dialog. The Host user must manually click **Accept**.
4. **Session Activation**: Upon consent, Host replies with a `SessionAccept` message detailing secondary UDP video and control ports. Both sides activate UDP pipelines and start low-latency operations.

# ArcLink Remote Development Roadmap

## Phase 1: Windows LAN Remote Control MVP (Current Stage)
- [x] Basic architecture split (`arclink-host` & `arclink-viewer`).
- [x] Protocol schemas (`arclink-common` with serialization test suites).
- [x] Custom Host UI with interactive connection-request modal in light glass theme.
- [x] Custom Viewer UI with interactive canvas, coordinate mapping, and diagnostics.
- [x] Simulated Windows 11 Desktop interface representing cross-platform compilation capabilities.
- [x] Multi-tab local area networking loopback via Full-stack WebSocket integration.

## Phase 2: Local Discovery & Utilities
- [ ] Implement LAN discovery using mDNS (`mdns-sd`) or UDP broadcast queries.
- [ ] Connect with randomized single-use 6-digit PIN codes (avoiding entering raw IPs).
- [ ] Bidirectional clipboard synchronization (text-only initially).
- [ ] Dual monitor/multi-screen display selector toolbar.

## Phase 3: Public Direct Connect & Hole Punching
- [ ] Integrate STUN (`stun` crate) and TURN protocol integrations for public WAN bypass.
- [ ] Implement NAT UDP hole punching (UPnP, PCP, and STUN coordinate pairing).
- [ ] Encrypt all control and video streams via TLS/DTLS.

## Phase 4: Extreme Zero-Copy Performance
- [ ] Implement full DXGI Duplication interface with D3D11 surface locking.
- [ ] Integrate Nvidia NVENC, AMD AMF, and Intel QuickSync bindings via ffmpeg/native wrappers.
- [ ] Integrate DXVA2 / Direct3D11 hardware decoding on the Viewer end.
- [ ] Adaptive Bitrate Control: dynamically lowering framerate/quality under network spikes.

## Phase 5: Cross-platform Ecosystem
- [ ] Native macOS controller client utilizing Metal and AppKit/egui.
- [ ] Web-based Viewer client using WebAssembly, WebGL, and WebSockets.
- [ ] Unified Remote Account SaaS Platform for simple out-of-box device linking.

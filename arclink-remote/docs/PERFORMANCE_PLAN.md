# ArcLink Remote Performance & Optimization Plan

This document outlines the pipeline latency targets and strategies to achieve smooth, sub-80ms local loop remote desktop control.

## 1. End-to-End Latency Budget

To achieve visual fluidness ("glove-like" input and display synchronization), our goal is an end-to-end latency below **80ms**.

| Pipeline Phase | Current MVP (Software Fallback) | Target High Performance | Optimization Strategy |
|---|---|---|---|
| **Screen Capture** | ~16ms - 25ms (GDI/Software) | **< 4ms** (DXGI / Windows Graphics Capture) | Migrate completely to DXGI Desktop Duplication API with direct GPU Surface access. |
| **Image Encoding** | ~15ms - 20ms (JPEG CPU) | **< 6ms** (NVENC / Intel QSV H.264) | Integrate hardware encoding via GPU hardware blocks, transferring raw DX11 surfaces directly. |
| **Network Transit**| ~2ms - 5ms (LAN direct) | **< 2ms** (Prioritized UDP socket) | Optimize Socket buffer pools, utilize UDP over raw TCP, pack events into small binary packets. |
| **Decoding** | ~8ms - 15ms (JPEG CPU decode) | **< 4ms** (D3D11 VAAPI decode) | Hardware decoding utilizing wgpu and DXVA2 within Viewer. |
| **Render / Display**| ~12ms (Standard render repaint) | **< 4ms** (Direct wgpu presentation) | Synchronize render loops with high refresh rate monitors (G-Sync/FreeSync support). |
| **Input Feedback** | ~25ms | **< 15ms** (Prioritized socket threading) | Run Input capture on separate high-priority threads in Viewer. |

---

## 2. Core Latency Mitigation Protocols

### Frame Drops / Frame Leaping
Under network jitter or bandwidth dropouts, UDP packets may be lost or delayed. Instead of stacking old buffered frames (which leads to "catching up" playback speedups), ArcLink implements **Frame Leaping**:
* Viewer tracks the sequence of the incoming frames.
* If a frame arrives with sequence index `N` while we are processing `N-2`, `N-2` and `N-1` are immediately discarded.
* The decoder only works on the newest valid frame `N`.

### Mouse Trajectory Truncation
Mouse coordinates can trigger 120+ events per second. Sending every single coordinate on slower networks wastes bandwidth and induces input queue lag.
* Viewer limits mouse coordinate sampling to the monitor’s target refresh rate.
* If a packet is delayed, the queue only executes the **most recent** pointer position on Host, discarding intermediate steps.

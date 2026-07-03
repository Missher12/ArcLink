# ArcLink Remote Protocol Definition (v1.0-LAN)

ArcLink Remote separates channels based on bandwidth intensity and reliability requirements.

## 1. Channels

| Channel | Protocol | Port | Reliability | Use Cases |
|---|---|---|---|---|
| **Control Channel** | TCP | `8443` (Default) | Reliable / Ordered | Session Handshake, Rejections, Heartbeats, Disconnect notices, Stats syncing |
| **Video Channel** | UDP | `8444` (Default) | Unreliable / Out-of-order | High-speed screenshare frames. Newer frames instantly override older frames. |
| **Input Channel** | UDP | `8445` (Default) | Unreliable but Prioritized | Real-time Mouse movement, keydowns, scroll events. |

---

## 2. Serialization Layout (Bincode / JSON)

To allow interoperability, messages can be serialized using standard `serde` binary formatting (`bincode` for performance, or `serde_json` for debugging).

### Control Messages (`ControlMessage`)

#### Connection Handshake Request
```json
{
  "Request": {
    "session_id": "REQ-0193-AB42",
    "viewer_name": "DESIGN-STATION-01",
    "viewer_ip": "192.168.1.102",
    "request_time": "2026-07-03T09:00:00Z",
    "required_fps": 60,
    "width": 1920,
    "height": 1080
  }
}
```

#### Connection Handshake Acceptance
```json
{
  "Accept": {
    "session_id": "REQ-0193-AB42",
    "host_name": "WORKSTATION-MAIN",
    "accepted_time": "2026-07-03T09:00:02Z",
    "control_port": 8443,
    "video_port": 8444
  }
}
```

#### Connection Handshake Rejection
```json
{
  "Reject": {
    "session_id": "REQ-0193-AB42",
    "reason": "Host user rejected the remote session request."
  }
}
```

---

## 3. Input Events (`InputEvent`)

To achieve low-latency input tracking, mouse movements are normalized.

### Mouse Move Event
Normalized screen coords are sent as floats between `0.0` and `1.0`. The host then scales these to its native monitor resolution (e.g. `1920x1080` or `3840x2160`).
```json
{
  "MouseMove": {
    "norm_x": 0.5123,
    "norm_y": 0.7412
  }
}
```

### Mouse Button Event
```json
{
  "MouseButton": {
    "button": "Left",
    "is_down": true
  }
}
```

### Keyboard Event
```json
{
  "Keyboard": {
    "vk_code": 13,
    "is_down": true,
    "modifiers": 0
  }
}
```

---

## 4. Heartbeat and Liveness

- **Interval**: Sent every **1000ms** on the TCP Control channel.
- **Timeout**: If no heartbeat or TCP traffic is registered for **4000ms**, the connection is marked as timed out.
- **Action**: Both sides instantly release control states and clean up sockets, ensuring the Host PC never gets stuck in a semi-controlled frozen state.

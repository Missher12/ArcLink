export interface DeviceInfo {
  device_id: string;
  device_name: string;
  local_ip: string;
  listen_port: number;
  screen_width: number;
  screen_height: number;
}

export type HostStatus = "idle" | "listening" | "connecting" | "occupied" | "error";
export type ViewerStatus = "disconnected" | "connecting" | "connected" | "reconnecting" | "error";

export interface NetworkStats {
  rtt_ms: number;
  jitter_ms: number;
  packet_loss_rate: number;
  sent_bytes_sec: number;
  rcv_bytes_sec: number;
}

export interface SessionMetrics {
  latency_ms: number;
  fps: number;
  bitrate_kbps: number;
  resolution_width: number;
  resolution_height: number;
  active_duration_secs: number;
  network: NetworkStats;
}

export interface InputEvent {
  type: "mousemove" | "mousedown" | "mouseup" | "keydown" | "keyup" | "scroll";
  x?: number; // Normalized (0 to 1)
  y?: number; // Normalized (0 to 1)
  button?: "left" | "right" | "middle";
  key?: string;
  code?: string;
  deltaX?: number;
  deltaY?: number;
}

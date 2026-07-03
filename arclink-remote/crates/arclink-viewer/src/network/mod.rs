use arclink_common::{
    ControlMessage, DeviceInfo, DisconnectMessage, DisconnectReason, HelloMessage, ViewerStatus,
    RemoteSession, SessionAccept, SessionMetrics, SessionReject, SessionRequest, Heartbeat,
    HeartbeatAck, InputEvent, VideoPacketHeader, PROTOCOL_MAGIC
};
use crate::logging::ViewerLogger;
use crate::metrics::ViewerMetricsManager;
use crate::render::VideoFrameBuffer;
use crate::session::ViewerSessionManager;

use chrono::Utc;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

pub struct NetworkManager {
    status: Arc<Mutex<ViewerStatus>>,
    active_session: Arc<Mutex<Option<RemoteSession>>>,
    metrics_mgr: ViewerMetricsManager,
    session_mgr: ViewerSessionManager,
    frame_buffer: Arc<Mutex<VideoFrameBuffer>>,
    
    // Channel to send inputs to background network thread
    input_tx: Option<UnboundedSender<InputEvent>>,
    cancel_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl NetworkManager {
    pub fn new(
        metrics_mgr: ViewerMetricsManager,
        session_mgr: ViewerSessionManager,
        frame_buffer: Arc<Mutex<VideoFrameBuffer>>,
    ) -> Self {
        Self {
            status: Arc::new(Mutex::new(ViewerStatus::Disconnected)),
            active_session: Arc::new(Mutex::new(None)),
            metrics_mgr,
            session_mgr,
            frame_buffer,
            input_tx: None,
            cancel_tx: None,
        }
    }

    pub fn get_status(&self) -> ViewerStatus {
        *self.status.lock().unwrap()
    }

    pub fn get_active_session(&self) -> Option<RemoteSession> {
        self.active_session.lock().unwrap().clone()
    }

    pub fn send_input_event(&self, event: InputEvent) {
        if let Some(ref tx) = self.input_tx {
            let _ = tx.send(event);
        }
    }

    pub fn connect_to_host(
        &mut self,
        host_ip: String,
        port: u16,
        logger: Arc<Mutex<ViewerLogger>>,
    ) -> Result<(), String> {
        let status = self.status.clone();
        let active_sess = self.active_session.clone();
        let metrics_mgr = self.metrics_mgr.clone();
        let session_mgr = self.session_mgr.clone();
        let frame_buffer = self.frame_buffer.clone();

        let (input_tx, mut input_rx) = unbounded_channel::<InputEvent>();
        self.input_tx = Some(input_tx);

        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
        self.cancel_tx = Some(cancel_tx);

        {
            let mut s = status.lock().unwrap();
            *s = ViewerStatus::Connecting;
        }

        let logger_c = logger.clone();
        tokio::spawn(async move {
            let host_addr = format!("{}:{}", host_ip, port);
            logger_c.lock().unwrap().info(&format!("Attempting to establish TCP handshake with Host: {}", host_addr));

            let mut socket = match TcpStream::connect(&host_addr).await {
                Ok(s) => {
                    logger_c.lock().unwrap().info("TCP socket connected! Executing remote handshake...");
                    s
                }
                Err(e) => {
                    logger_c.lock().unwrap().error(&format!("Failed to connect to host: {}", e));
                    let mut s = status.lock().unwrap();
                    *s = ViewerStatus::Error;
                    return;
                }
            };

            // 1. Send Hello + SessionRequest
            let session_id = format!("SESS-{}", Utc::now().timestamp_millis());
            let request_msg = ControlMessage::SessionRequest(SessionRequest {
                session_id: session_id.clone(),
                viewer_name: "ARC-VIEWER-9911".to_string(),
                viewer_ip: "127.0.0.1".into(), // Will be updated on handshake
                request_time: Utc::now(),
                required_fps: 30,
                width: 1920,
                height: 1080,
            });

            if let Ok(serialized) = bincode::serialize(&request_msg) {
                let mut frame = Vec::new();
                frame.extend_from_slice(&PROTOCOL_MAGIC);
                frame.extend_from_slice(&(serialized.len() as u32).to_be_bytes());
                frame.extend_from_slice(&serialized);
                if let Err(e) = socket.write_all(&frame).await {
                    logger_c.lock().unwrap().error(&format!("Failed to write request: {}", e));
                    let mut s = status.lock().unwrap();
                    *s = ViewerStatus::Error;
                    return;
                }
            }

            logger_c.lock().unwrap().info("Session request dispatched. Waiting for Host approval...");

            // 2. Read Host Handshake response (Accept/Reject)
            let mut header = [0u8; 8];
            if socket.read_exact(&mut header).await.is_err() {
                logger_c.lock().unwrap().error("Connection closed prematurely during handshake.");
                let mut s = status.lock().unwrap();
                *s = ViewerStatus::Error;
                return;
            }
            if &header[0..4] != PROTOCOL_MAGIC {
                logger_c.lock().unwrap().error("Received invalid protocol magic from Host.");
                let mut s = status.lock().unwrap();
                *s = ViewerStatus::Error;
                return;
            }
            let len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
            let mut payload = vec![0u8; len];
            if socket.read_exact(&mut payload).await.is_err() {
                logger_c.lock().unwrap().error("Failed to read handshake response payload.");
                let mut s = status.lock().unwrap();
                *s = ViewerStatus::Error;
                return;
            }

            let response: ControlMessage = match bincode::deserialize(&payload) {
                Ok(r) => r,
                Err(_) => {
                    logger_c.lock().unwrap().error("Failed to parse handshake response.");
                    let mut s = status.lock().unwrap();
                    *s = ViewerStatus::Error;
                    return;
                }
            };

            match response {
                ControlMessage::SessionAccept(accept) => {
                    logger_c.lock().unwrap().info(&format!("Session request ACCEPTED by Host: {}", accept.host_name));
                    
                    let local_ip = socket.local_addr().map(|a| a.ip().to_string()).unwrap_or_else(|_| "127.0.0.1".into());
                    let session = RemoteSession {
                        session_id: accept.session_id,
                        viewer_id: "ARC-VIEWER-9911".to_string(),
                        host_id: accept.host_name,
                        viewer_ip: local_ip,
                        host_ip: host_ip.clone(),
                        start_time: Utc::now(),
                        protocol_version: "1.0".to_string(),
                        allow_control: true,
                    };

                    {
                        let mut s = status.lock().unwrap();
                        *s = ViewerStatus::Connected;
                        *active_sess.lock().unwrap() = Some(session.clone());
                    }

                    session_mgr.start_session(session.clone());
                    metrics_mgr.start_session();

                    // Start background UDP listener to collect streaming video frames on port 8444
                    let udp_listener_addr = SocketAddr::from(([0, 0, 0, 0], 8444));
                    let udp_socket = match UdpSocket::bind(udp_listener_addr) {
                        Ok(sock) => {
                            logger_c.lock().unwrap().info("UDP Video receiver bound to 0.0.0.0:8444");
                            sock
                        }
                        Err(e) => {
                            logger_c.lock().unwrap().error(&format!("UDP Bind Error on port 8444: {}", e));
                            let mut s = status.lock().unwrap();
                            *s = ViewerStatus::Disconnected;
                            return;
                        }
                    };
                    udp_socket.set_read_timeout(Some(Duration::from_millis(50))).unwrap();

                    let frame_buffer_c = frame_buffer.clone();
                    let metrics_mgr_c = metrics_mgr.clone();
                    let session_mgr_c = session_mgr.clone();

                    // UDP collector thread
                    let udp_handler = std::thread::spawn(move || {
                        let mut buffer = vec![0u8; 1500];
                        while session_mgr_c.get_active_session().is_some() {
                            match udp_socket.recv_from(&mut buffer) {
                                Ok((size, _sender)) => {
                                    metrics_mgr_c.add_bytes_recv(size);
                                    let packet = &buffer[0..size];
                                    
                                    // Parse packet header
                                    if packet.len() > 32 {
                                        if let Ok(hdr) = bincode::deserialize::<VideoPacketHeader>(&packet[0..32]) {
                                            if hdr.magic == 0x4152434L || hdr.magic == 0x4152434C { // Match standard magic
                                                let payload_part = &packet[32..];
                                                frame_buffer_c.lock().unwrap().handle_packet(hdr, payload_part);
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    // UDP read timeout or normal spin
                                }
                            }
                        }
                    });

                    // B. Start TCP live communication loop
                    socket.set_nodelay(true).unwrap();
                    let mut last_heartbeat_sent = Instant::now();
                    let mut last_heatbeat_received = Instant::now();
                    let mut sequence = 0;

                    tokio::select! {
                        _ = async {
                            loop {
                                // 1. Send periodic Heartbeats (every 1 second)
                                if last_heartbeat_sent.elapsed() > Duration::from_secs(1) {
                                    sequence += 1;
                                    let hb = ControlMessage::Heartbeat(Heartbeat {
                                        timestamp_us: Utc::now().timestamp_micros() as u64,
                                        sequence,
                                    });
                                    if let Ok(serialized) = bincode::serialize(&hb) {
                                        let mut frame = Vec::new();
                                        frame.extend_from_slice(&PROTOCOL_MAGIC);
                                        frame.extend_from_slice(&(serialized.len() as u32).to_be_bytes());
                                        frame.extend_from_slice(&serialized);
                                        if socket.write_all(&frame).await.is_err() {
                                            break;
                                        }
                                    }
                                    last_heartbeat_sent = Instant::now();
                                }

                                // 2. Timeout if Host is silent for more than 5 seconds
                                if last_heatbeat_received.elapsed() > Duration::from_secs(5) {
                                    logger_c.lock().unwrap().error("Host heartbeat TIMEOUT. Terminating session.");
                                    break;
                                }

                                // 3. Send queued user inputs to Host
                                while let Ok(event) = input_rx.try_recv() {
                                    if let Ok(serialized) = bincode::serialize(&event) {
                                        let mut frame = Vec::new();
                                        frame.extend_from_slice(&PROTOCOL_MAGIC);
                                        frame.extend_from_slice(&(serialized.len() as u32).to_be_bytes());
                                        frame.extend_from_slice(&serialized);
                                        if socket.write_all(&frame).await.is_err() {
                                            break;
                                        }
                                    }
                                }

                                // 4. Non-blocking read for incoming Host messages (Metrics, Disconnects, HeartbeatAcks)
                                let mut r_hdr = [0u8; 8];
                                socket.set_read_timeout(Some(Duration::from_millis(50))).unwrap();
                                if let Ok(_) = socket.read_exact(&mut r_hdr).await {
                                    if &r_hdr[0..4] == PROTOCOL_MAGIC {
                                        let r_len = u32::from_be_bytes([r_hdr[4], r_hdr[5], r_hdr[6], r_hdr[7]]) as usize;
                                        let mut r_payload = vec![0u8; r_len];
                                        if socket.read_exact(&mut r_payload).await.is_ok() {
                                            if let Ok(msg) = bincode::deserialize::<ControlMessage>(&r_payload) {
                                                match msg {
                                                    ControlMessage::HeartbeatAck(ack) => {
                                                        last_heatbeat_received = Instant::now();
                                                        let rtt = (Utc::now().timestamp_micros() as u64).saturating_sub(ack.timestamp_us) as f32 / 1000.0;
                                                        metrics_mgr.update_rtt(rtt);
                                                    }
                                                    ControlMessage::SessionMetrics(m) => {
                                                        // Merge server-calculated stats
                                                        if let Some(c_fps) = m.capture_fps {
                                                            metrics_mgr.update_render_fps(c_fps);
                                                        }
                                                        if let Some(c_lat) = m.capture_latency_ms {
                                                            metrics_mgr.update_render_latency(c_lat);
                                                        }
                                                    }
                                                    ControlMessage::Disconnect(d) => {
                                                        logger_c.lock().unwrap().warn(&format!("Host closed connection cleanly. Reason: {:?}", d.reason));
                                                        break;
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }

                                tokio::time::sleep(Duration::from_millis(15)).await;
                            }
                        } => {}
                        _ = cancel_rx => {
                            logger_c.lock().unwrap().info("Disconnect triggered by Viewer operator.");
                            // Dispatch disconnect notice to Host
                            let d_msg = ControlMessage::Disconnect(DisconnectMessage {
                                reason: DisconnectReason::UserClosed,
                                message: "Viewer closed.".into(),
                            });
                            if let Ok(serialized) = bincode::serialize(&d_msg) {
                                let mut frame = Vec::new();
                                frame.extend_from_slice(&PROTOCOL_MAGIC);
                                frame.extend_from_slice(&(serialized.len() as u32).to_be_bytes());
                                frame.extend_from_slice(&serialized);
                                let _ = socket.write_all(&frame).await;
                            }
                        }
                    }

                    // Clean up and disconnect
                    session_mgr.end_session();
                    let _ = udp_handler.join();
                    
                    {
                        let mut s = status.lock().unwrap();
                        *s = ViewerStatus::Disconnected;
                        *active_sess.lock().unwrap() = None;
                    }
                    metrics_mgr.stop_session();
                    frame_buffer.lock().unwrap().clear();
                    logger_c.lock().unwrap().info("Remote control session concluded safely.");
                }
                ControlMessage::SessionReject(reject) => {
                    logger_c.lock().unwrap().warn(&format!("Session request REJECTED by Host: {}", reject.reason));
                    let mut s = status.lock().unwrap();
                    *s = ViewerStatus::Disconnected;
                }
                _ => {
                    logger_c.lock().unwrap().error("Unexpected protocol message received during handshake.");
                    let mut s = status.lock().unwrap();
                    *s = ViewerStatus::Error;
                }
            }
        });

        Ok(())
    }

    pub fn disconnect_from_host(&mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(());
        }
        let mut s = self.status.lock().unwrap();
        *s = ViewerStatus::Disconnected;
        self.session_mgr.end_session();
        self.metrics_mgr.stop_session();
        self.frame_buffer.lock().unwrap().clear();
    }
}

use arclink_common::{
    ControlMessage, DeviceInfo, DisconnectMessage, DisconnectReason, HelloMessage, HostStatus,
    RemoteSession, SessionAccept, SessionMetrics, SessionReject, SessionRequest, Heartbeat,
    HeartbeatAck, InputEvent, VideoPacketHeader, PROTOCOL_MAGIC
};
use crate::capture::create_default_capturer;
use crate::input::InputInjector;
use crate::logging::HostLogger;
use crate::metrics::HostMetricsManager;
use crate::session::HostSessionManager;

use chrono::Utc;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub enum UserAction {
    Accept,
    Reject,
    Disconnect,
}

pub struct NetworkManager {
    local_ip: String,
    port: u16,
    status: Arc<Mutex<HostStatus>>,
    active_request: Arc<Mutex<Option<SessionRequest>>>,
    active_session: Arc<Mutex<Option<RemoteSession>>>,
    metrics_mgr: HostMetricsManager,
    session_mgr: HostSessionManager,
    injector: Arc<InputInjector>,
    
    // Channels to communicate UI choices to network thread
    user_action_tx: Arc<Mutex<Option<Sender<UserAction>>>>,
    cancel_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl NetworkManager {
    pub fn new(metrics_mgr: HostMetricsManager, session_mgr: HostSessionManager) -> Self {
        let local_ip = detect_local_ip();
        Self {
            local_ip,
            port: 8443,
            status: Arc::new(Mutex::new(HostStatus::Idle)),
            active_request: Arc::new(Mutex::new(None)),
            active_session: Arc::new(Mutex::new(None)),
            metrics_mgr,
            session_mgr,
            injector: Arc::new(InputInjector::new()),
            user_action_tx: Arc::new(Mutex::new(None)),
            cancel_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn local_ip(&self) -> String {
        self.local_ip.clone()
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn get_status(&self) -> HostStatus {
        *self.status.lock().unwrap()
    }

    pub fn get_active_request(&self) -> Option<SessionRequest> {
        self.active_request.lock().unwrap().clone()
    }

    pub fn get_active_session(&self) -> Option<RemoteSession> {
        self.active_session.lock().unwrap().clone()
    }

    pub fn trigger_user_action(&self, action: UserAction) {
        if let Some(tx) = &*self.user_action_tx.lock().unwrap() {
            let _ = tx.send(action);
        }
    }

    pub fn start_listening(&mut self, logger: Arc<Mutex<HostLogger>>) -> Result<(), String> {
        let status = self.status.clone();
        let port = self.port;
        let active_req = self.active_request.clone();
        let active_sess = self.active_session.clone();
        let metrics_mgr = self.metrics_mgr.clone();
        let session_mgr = self.session_mgr.clone();
        let injector = self.injector.clone();

        let (action_tx, action_rx) = channel::<UserAction>();
        *self.user_action_tx.lock().unwrap() = Some(action_tx);

        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();
        *self.cancel_tx.lock().unwrap() = Some(cancel_tx);

        {
            let mut s = status.lock().unwrap();
            *s = HostStatus::Listening;
        }

        // Spawn Tokio server listener
        let logger_c = logger.clone();
        tokio::spawn(async move {
            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            let listener = match TcpListener::bind(addr).await {
                Ok(l) => {
                    logger_c.lock().unwrap().info(&format!("TCP control listener bound to 0.0.0.0:{}", port));
                    l
                }
                Err(e) => {
                    logger_c.lock().unwrap().error(&format!("Failed to bind TCP listener to port {}: {}", port, e));
                    let mut s = status.lock().unwrap();
                    *s = HostStatus::Error;
                    return;
                }
            };

            // Loop waiting for connection
            tokio::select! {
                _ = async {
                    loop {
                        match listener.accept().await {
                            Ok((socket, client_addr)) => {
                                logger_c.lock().unwrap().info(&format!("Inbound TCP connection from client: {}", client_addr));
                                
                                // Process session connection sequence
                                let res = handle_session_handshake(
                                    socket,
                                    client_addr,
                                    status.clone(),
                                    active_req.clone(),
                                    active_sess.clone(),
                                    &action_rx,
                                    metrics_mgr.clone(),
                                    session_mgr.clone(),
                                    injector.clone(),
                                    logger_c.clone()
                                ).await;

                                if let Err(e) = res {
                                    logger_c.lock().unwrap().warn(&format!("Handshake finished with error: {}", e));
                                    let mut s = status.lock().unwrap();
                                    *s = HostStatus::Listening;
                                }
                            }
                            Err(e) => {
                                logger_c.lock().unwrap().error(&format!("TCP accept error: {}", e));
                            }
                        }
                    }
                } => {}
                _ = cancel_rx => {
                    logger_c.lock().unwrap().info("TCP listener service shut down safely.");
                }
            }
        });

        Ok(())
    }

    pub fn stop_listening(&mut self, logger: Arc<Mutex<HostLogger>>) {
        if let Some(tx) = self.cancel_tx.lock().unwrap().take() {
            let _ = tx.send(());
        }
        let mut s = self.status.lock().unwrap();
        *s = HostStatus::Idle;
        self.session_mgr.end_session();
        self.metrics_mgr.stop_session();
        *self.active_request.lock().unwrap() = None;
        *self.active_session.lock().unwrap() = None;
        logger.lock().unwrap().info("Listener stopped. Sockets closed.");
    }
}

async fn handle_session_handshake(
    mut socket: TcpStream,
    client_addr: SocketAddr,
    status: Arc<Mutex<HostStatus>>,
    active_req: Arc<Mutex<Option<SessionRequest>>>,
    active_sess: Arc<Mutex<Option<RemoteSession>>>,
    action_rx: &Receiver<UserAction>,
    mut metrics_mgr: HostMetricsManager,
    session_mgr: HostSessionManager,
    injector: Arc<InputInjector>,
    logger: Arc<Mutex<HostLogger>>
) -> Result<(), String> {
    // 1. Read Hello Message & Request
    let mut header = [0u8; 8];
    socket.read_exact(&mut header).await.map_err(|e| format!("Header read error: {}", e))?;
    if &header[0..4] != PROTOCOL_MAGIC {
        return Err("Invalid protocol magic".into());
    }
    let len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
    if len > 5000 {
        return Err("Payload too large".into());
    }
    let mut payload = vec![0u8; len];
    socket.read_exact(&mut payload).await.map_err(|e| format!("Payload read error: {}", e))?;
    
    let msg: ControlMessage = bincode::deserialize(&payload).map_err(|e| format!("Deserialize request error: {}", e))?;
    
    let req = match msg {
        ControlMessage::SessionRequest(r) => r,
        _ => return Err("Expected SessionRequest as first message".into()),
    };

    logger.lock().unwrap().info(&format!("Handshake session request [{}] received from {}", req.session_id, req.viewer_name));

    // Update state to Connecting
    {
        let mut s = status.lock().unwrap();
        *s = HostStatus::Connecting;
        *active_req.lock().unwrap() = Some(req.clone());
    }

    // 2. Wait for user accept/reject action
    let user_choice = loop {
        match action_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(act) => break act,
            Err(_) => {
                // Check if socket is still alive (by checking if client disconnected)
                let mut buf = [0u8; 1];
                socket.set_read_timeout(Some(Duration::from_millis(1))).unwrap();
                if let Ok(0) = socket.read(&mut buf).await {
                    logger.lock().unwrap().warn("Viewer disconnected during handshake pending.");
                    *active_req.lock().unwrap() = None;
                    return Err("Viewer disconnected".into());
                }
                socket.set_read_timeout(None).unwrap();
            }
        }
    };

    match user_choice {
        UserAction::Accept => {
            logger.lock().unwrap().info("Host user ACCEPTED the session request.");
            
            let accepted_session = RemoteSession {
                session_id: req.session_id.clone(),
                viewer_id: req.viewer_name.clone(),
                host_id: "ARC-HOST-7890".to_string(),
                viewer_ip: client_addr.ip().to_string(),
                host_ip: detect_local_ip(),
                start_time: Utc::now(),
                protocol_version: "1.0".to_string(),
                allow_control: true,
            };

            // Send SessionAccept response
            let accept_msg = ControlMessage::SessionAccept(SessionAccept {
                session_id: req.session_id.clone(),
                host_name: "WORKSTATION-WIN11-PRO".to_string(),
                accepted_time: Utc::now(),
                control_port: 8443,
                video_port: 8444,
            });
            let serialized = bincode::serialize(&accept_msg).unwrap();
            let mut frame = Vec::new();
            frame.extend_from_slice(&PROTOCOL_MAGIC);
            frame.extend_from_slice(&(serialized.len() as u32).to_be_bytes());
            frame.extend_from_slice(&serialized);
            socket.write_all(&frame).await.map_err(|e| format!("Accept write failed: {}", e))?;

            // Transition state to Occupied
            {
                let mut s = status.lock().unwrap();
                *s = HostStatus::Occupied;
                *active_req.lock().unwrap() = None;
                *active_sess.lock().unwrap() = Some(accepted_session.clone());
            }

            session_mgr.start_session(accepted_session.clone());
            metrics_mgr.start_session();

            // Spawn live session loop (Inputs, Video, Heartbeat)
            let session_err = run_live_session(
                socket,
                accepted_session,
                action_rx,
                metrics_mgr.clone(),
                session_mgr,
                injector,
                logger.clone()
            ).await;

            // Session concluded
            {
                let mut s = status.lock().unwrap();
                *s = HostStatus::Listening;
                *active_sess.lock().unwrap() = None;
            }
            metrics_mgr.stop_session();

            if let Err(e) = session_err {
                logger.lock().unwrap().warn(&format!("Active remote session ended with notice: {}", e));
            } else {
                logger.lock().unwrap().info("Remote session closed cleanly.");
            }
        }
        UserAction::Reject | UserAction::Disconnect => {
            logger.lock().unwrap().info("Host user REJECTED the session request.");
            let reject_msg = ControlMessage::SessionReject(SessionReject {
                session_id: req.session_id.clone(),
                reason: "Handshake request rejected by Host user.".to_string(),
            });
            let serialized = bincode::serialize(&reject_msg).unwrap();
            let mut frame = Vec::new();
            frame.extend_from_slice(&PROTOCOL_MAGIC);
            frame.extend_from_slice(&(serialized.len() as u32).to_be_bytes());
            frame.extend_from_slice(&serialized);
            let _ = socket.write_all(&frame).await;
            
            {
                let mut s = status.lock().unwrap();
                *s = HostStatus::Listening;
                *active_req.lock().unwrap() = None;
            }
        }
    }

    Ok(())
}

async fn run_live_session(
    mut socket: TcpStream,
    session: RemoteSession,
    action_rx: &Receiver<UserAction>,
    metrics_mgr: HostMetricsManager,
    session_mgr: HostSessionManager,
    injector: Arc<InputInjector>,
    logger: Arc<Mutex<HostLogger>>
) -> Result<(), String> {
    // A. Spawn Video Streamer (UDP packets over port 8444)
    let video_target_addr: SocketAddr = format!("{}:8444", session.viewer_ip).parse()
        .map_err(|e| format!("Invalid viewer video target IP: {}", e))?;
    
    let udp_socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("UDP Bind Error: {}", e))?;
    udp_socket.set_write_timeout(Some(Duration::from_millis(50))).unwrap();

    let session_c = session.clone();
    let metrics_mgr_c = metrics_mgr.clone();
    let logger_c = logger.clone();

    // Start background video encoding/capturing task
    let video_handler = tokio::task::spawn_blocking(move || {
        let mut capturer = create_default_capturer();
        let _ = capturer.start();
        
        let mut frame_id = 0;
        logger_c.lock().unwrap().info(&format!("UDP Video Streamer task initialized towards: {}", video_target_addr));

        while session_mgr.get_active_session().is_some() {
            let start_frame = Instant::now();
            match capturer.next_frame() {
                Ok(frame) => {
                    frame_id += 1;
                    metrics_mgr_c.update_capture_latency(frame.capture_duration.as_secs_f32() * 1000.0);
                    metrics_mgr_c.update_capture_fps(1.0 / start_frame.elapsed().as_secs_f32());

                    // Fragment JPEG bytes into 1100 byte pieces for safe MTU transmission
                    let payload = frame.bytes;
                    let chunk_size = 1100;
                    let total_chunks = ((payload.len() as f32) / (chunk_size as f32)).ceil() as usize;

                    for index in 0..total_chunks {
                        let start = index * chunk_size;
                        let end = (start + chunk_size).min(payload.len());
                        let chunk_payload = &payload[start..end];

                        let header = VideoPacketHeader {
                            magic: 0x4152434C, // "ARCL"
                            protocol_version: 1,
                            session_id: 12345, // Sim simplified
                            frame_id,
                            capture_timestamp_us: Utc::now().timestamp_micros() as u64,
                            fragment_index: index as u16,
                            fragment_count: total_chunks as u16,
                            payload_len: chunk_payload.len() as u16,
                            flags: if index == total_chunks - 1 { 1 } else { 0 }, // marker for last frag
                        };

                        // Construct packet
                        if let Ok(hdr_bytes) = bincode::serialize(&header) {
                            let mut packet = Vec::with_capacity(hdr_bytes.len() + chunk_payload.len());
                            packet.extend_from_slice(&hdr_bytes);
                            packet.extend_from_slice(chunk_payload);

                            // Send over UDP
                            let _ = udp_socket.send_to(&packet, video_target_addr);
                            metrics_mgr_c.add_bytes_sent(packet.len());
                        }
                    }
                }
                Err(e) => {
                    logger_c.lock().unwrap().warn(&format!("Capture error during streaming: {:?}", e));
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
            // limit to roughly ~30 FPS capture rate to save bandwidth
            std::thread::sleep(Duration::from_millis(33));
        }
        capturer.stop();
        logger_c.lock().unwrap().info("UDP Video Streamer task stopped.");
    });

    // B. Main Control pipeline reader (inputs and heartbeats)
    let mut last_heartbeat_received = Instant::now();
    let mut last_metrics_sent = Instant::now();
    let mut sequence = 0;

    socket.set_nodelay(true).unwrap();

    loop {
        // Check local user action (e.g. Host clicked "Disconnect" button)
        if let Ok(UserAction::Disconnect) = action_rx.try_recv() {
            logger.lock().unwrap().warn("Active remote connection terminated by Host User.");
            
            // Notify client before quitting
            let disc_msg = ControlMessage::Disconnect(DisconnectMessage {
                reason: DisconnectReason::UserClosed,
                message: "Closed by host side operator.".into(),
            });
            if let Ok(serialized) = bincode::serialize(&disc_msg) {
                let mut frame = Vec::new();
                frame.extend_from_slice(&PROTOCOL_MAGIC);
                frame.extend_from_slice(&(serialized.len() as u32).to_be_bytes());
                frame.extend_from_slice(&serialized);
                let _ = socket.write_all(&frame).await;
            }
            break;
        }

        // Heartbeat timeout check (5 seconds timeout)
        if last_heartbeat_received.elapsed() > Duration::from_secs(5) {
            logger.lock().unwrap().error("Viewer heartbeat TIMEOUT (5 seconds silent). Terminating session.");
            break;
        }

        // Send metrics periodically (every 1 second)
        if last_metrics_sent.elapsed() > Duration::from_secs(1) {
            let m = metrics_mgr.get_metrics();
            let stats_msg = ControlMessage::SessionMetrics(m);
            if let Ok(serialized) = bincode::serialize(&stats_msg) {
                let mut frame = Vec::new();
                frame.extend_from_slice(&PROTOCOL_MAGIC);
                frame.extend_from_slice(&(serialized.len() as u32).to_be_bytes());
                frame.extend_from_slice(&serialized);
                if socket.write_all(&frame).await.is_err() {
                    break;
                }
            }
            last_metrics_sent = Instant::now();
        }

        // Non-blocking read or quick read timeout for incoming inputs
        let mut header = [0u8; 8];
        socket.set_read_timeout(Some(Duration::from_millis(50))).unwrap();
        
        match socket.read_exact(&mut header).await {
            Ok(_) => {
                if &header[0..4] != PROTOCOL_MAGIC {
                    logger.lock().unwrap().error("Framing magic error on control stream. Terminating.");
                    break;
                }
                let len = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;
                if len > 50000 {
                    logger.lock().unwrap().error("Control message frame size limit exceeded.");
                    break;
                }
                let mut payload = vec![0u8; len];
                if socket.read_exact(&mut payload).await.is_err() {
                    break;
                }

                // Decode payload
                if let Ok(msg) = bincode::deserialize::<ControlMessage>(&payload) {
                    match msg {
                        ControlMessage::Heartbeat(hb) => {
                            last_heartbeat_received = Instant::now();
                            // Calculate RTT
                            let rtt = (Utc::now().timestamp_micros() as u64).saturating_sub(hb.timestamp_us) as f32 / 1000.0;
                            metrics_mgr.update_rtt(rtt);

                            // Reply HeartbeatAck
                            let ack = ControlMessage::HeartbeatAck(HeartbeatAck {
                                timestamp_us: hb.timestamp_us,
                                sequence: hb.sequence,
                            });
                            if let Ok(serialized) = bincode::serialize(&ack) {
                                let mut frame = Vec::new();
                                frame.extend_from_slice(&PROTOCOL_MAGIC);
                                frame.extend_from_slice(&(serialized.len() as u32).to_be_bytes());
                                frame.extend_from_slice(&serialized);
                                let _ = socket.write_all(&frame).await;
                            }
                        }
                        ControlMessage::Disconnect(_) => {
                            logger.lock().unwrap().info("Viewer client requested to disconnect.");
                            break;
                        }
                        _ => {}
                    }
                } else if let Ok(event) = bincode::deserialize::<InputEvent>(&payload) {
                    // This is a direct input event!
                    if session_mgr.is_session_authorized(&session.session_id, &session.viewer_ip) {
                        let _ = injector.inject_event(&event);
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldCheck => {
                // Read timeout, normal loop spin
            }
            Err(_) => {
                // Socket error
                break;
            }
        }
    }

    session_mgr.end_session();
    let _ = video_handler.await;

    Ok(())
}

fn detect_local_ip() -> String {
    use std::net::UdpSocket;
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(local_addr) = socket.local_addr() {
                return local_addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

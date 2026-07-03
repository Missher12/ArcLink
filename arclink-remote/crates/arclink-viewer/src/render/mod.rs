use arclink_common::VideoPacketHeader;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct FrameFragment {
    pub timestamp: Instant,
    pub fragments: HashMap<u16, Vec<u8>>,
    pub count: u16,
}

pub struct VideoFrameBuffer {
    active_reassembly: HashMap<u64, FrameFragment>,
    latest_decoded_frame: Arc<Mutex<Option<DecodedFrame>>>,
}

#[derive(Clone)]
pub struct DecodedFrame {
    pub width: u32,
    pub height: u32,
    pub rgba_bytes: Vec<u8>,
    pub frame_id: u64,
}

impl VideoFrameBuffer {
    pub fn new() -> Self {
        Self {
            active_reassembly: HashMap::new(),
            latest_decoded_frame: Arc::new(Mutex::new(None)),
        }
    }

    pub fn handle_packet(&mut self, header: VideoPacketHeader, payload: &[u8]) {
        let now = Instant::now();

        // 1. Clean up stale frame reassemblies (older than 100ms)
        self.active_reassembly.retain(|_, frag| now.duration_since(frag.timestamp) < Duration::from_millis(100));

        // 2. Discard if we have a much newer frame_id in our successful buffer already
        if let Some(ref latest) = *self.latest_decoded_frame.lock().unwrap() {
            if header.frame_id < latest.frame_id {
                return; // drop old frame fragments
            }
        }

        // 3. Add to reassembly
        let entry = self.active_reassembly.entry(header.frame_id).or_insert_with(|| FrameFragment {
            timestamp: now,
            fragments: HashMap::new(),
            count: header.fragment_count,
        });

        entry.fragments.insert(header.fragment_index, payload.to_vec());

        // 4. Check if we received all fragments
        if entry.fragments.len() == entry.count as usize {
            // Reassemble complete payload
            let mut complete_bytes = Vec::new();
            for i in 0..entry.count {
                if let Some(chunk) = entry.fragments.get(&i) {
                    complete_bytes.extend_from_slice(chunk);
                } else {
                    return; // missing fragment, shouldn't happen if len matches, but safe guard
                }
            }

            // Remove from active reassembly list
            self.active_reassembly.remove(&header.frame_id);

            // Decode JPEG bytes asynchronously on background pool or here
            let latest_frame = self.latest_decoded_frame.clone();
            let frame_id = header.frame_id;
            
            // Spawn a thread to decode without blocking incoming network socket
            std::thread::spawn(move || {
                if let Ok(img) = image::load_from_memory(&complete_bytes) {
                    let width = img.width();
                    let height = img.height();
                    let rgba_bytes = img.to_rgba8().into_raw();

                    let mut dest = latest_frame.lock().unwrap();
                    // Guarantee we only preserve the absolute latest frame (max 1 frame in slot)
                    if dest.as_ref().map_or(true, |curr| frame_id > curr.frame_id) {
                        *dest = Some(DecodedFrame {
                            width,
                            height,
                            rgba_bytes,
                            frame_id,
                        });
                    }
                }
            });
        }
    }

    pub fn take_latest_frame(&self) -> Option<DecodedFrame> {
        self.latest_decoded_frame.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        let mut f = self.latest_decoded_frame.lock().unwrap();
        *f = None;
    }
}

use arclink_common::CaptureError;
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CaptureSourceInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub pixel_format: String,
    pub captured_at: DateTime<Utc>,
    pub frame_sequence: u64,
    pub bytes: Vec<u8>,
    pub capture_duration: Duration,
}

pub trait ScreenCapturer: Send + Sync {
    fn start(&mut self) -> Result<(), CaptureError>;
    fn next_frame(&mut self) -> Result<CapturedFrame, CaptureError>;
    fn stop(&mut self);
    fn source_info(&self) -> CaptureSourceInfo;
}

// =========================================================================
// Real Windows GDI Capturer
// =========================================================================
#[cfg(target_os = "windows")]
pub struct WindowsGdiCapturer {
    width: u32,
    height: u32,
    frame_sequence: u64,
    is_running: bool,
}

#[cfg(target_os = "windows")]
impl WindowsGdiCapturer {
    pub fn new() -> Self {
        use windows::Win32::Graphics::Gdi::*;
        let (w, h) = unsafe {
            let h_dc = GetDC(None);
            let w = GetDeviceCaps(h_dc, HORZRES) as u32;
            let h = GetDeviceCaps(h_dc, VERTRES) as u32;
            ReleaseDC(None, h_dc);
            (w, h)
        };
        Self {
            width: w,
            height: h,
            frame_sequence: 0,
            is_running: false,
        }
    }
}

#[cfg(target_os = "windows")]
impl ScreenCapturer for WindowsGdiCapturer {
    fn start(&mut self) -> Result<(), CaptureError> {
        self.is_running = true;
        Ok(())
    }

    fn next_frame(&mut self) -> Result<CapturedFrame, CaptureError> {
        if !self.is_running {
            return Err(CaptureError::CaptureFailed("Capturer not started".into()));
        }

        let start_time = Instant::now();
        use windows::Win32::Graphics::Gdi::*;
        use windows::Win32::Foundation::HWND;

        unsafe {
            let h_screen_dc = GetDC(HWND(0));
            if h_screen_dc.is_invalid() {
                return Err(CaptureError::CaptureFailed("Failed to get screen DC".into()));
            }

            let h_mem_dc = CreateCompatibleDC(h_screen_dc);
            if h_mem_dc.is_invalid() {
                ReleaseDC(HWND(0), h_screen_dc);
                return Err(CaptureError::CaptureFailed("Failed to create memory DC".into()));
            }

            let h_bitmap = CreateCompatibleBitmap(h_screen_dc, self.width as i32, self.height as i32);
            if h_bitmap.is_invalid() {
                let _ = DeleteDC(h_mem_dc);
                ReleaseDC(HWND(0), h_screen_dc);
                return Err(CaptureError::CaptureFailed("Failed to create bitmap".into()));
            }

            let h_old_bitmap = SelectObject(h_mem_dc, h_bitmap);

            // Copy screen contents to memory DC
            let ok = BitBlt(
                h_mem_dc,
                0,
                0,
                self.width as i32,
                self.height as i32,
                h_screen_dc,
                0,
                0,
                SRCCOPY,
            );

            if !ok.as_bool() {
                SelectObject(h_mem_dc, h_old_bitmap);
                let _ = DeleteObject(h_bitmap);
                let _ = DeleteDC(h_mem_dc);
                ReleaseDC(HWND(0), h_screen_dc);
                return Err(CaptureError::CaptureFailed("BitBlt screen copy failed".into()));
            }

            // Read pixels from bitmap
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: self.width as i32,
                    biHeight: -(self.height as i32), // Top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD::default(); 1],
            };

            let pixel_count = (self.width * self.height) as usize;
            let mut raw_pixels = vec![0u8; pixel_count * 4];

            let scanlines = GetDIBits(
                h_mem_dc,
                h_bitmap,
                0,
                self.height,
                Some(raw_pixels.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            // Cleanup GDI objects immediately
            SelectObject(h_mem_dc, h_old_bitmap);
            let _ = DeleteObject(h_bitmap);
            let _ = DeleteDC(h_mem_dc);
            ReleaseDC(HWND(0), h_screen_dc);

            if scanlines == 0 {
                return Err(CaptureError::CaptureFailed("GetDIBits pixels extraction failed".into()));
            }

            // Compress BGRA to JPEG using the `image` crate
            let mut jpeg_bytes = Vec::new();
            {
                // Convert BGRA to RGB for JPEG encoding
                let mut rgb_pixels = Vec::with_capacity(pixel_count * 3);
                for chunk in raw_pixels.chunks_exact(4) {
                    rgb_pixels.push(chunk[2]); // R
                    rgb_pixels.push(chunk[1]); // G
                    rgb_pixels.push(chunk[0]); // B
                }

                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_bytes, 75);
                if encoder.encode(&rgb_pixels, self.width, self.height, image::ColorType::Rgb8).is_err() {
                    return Err(CaptureError::CaptureFailed("JPEG encoding of screen frame failed".into()));
                }
            }

            self.frame_sequence += 1;
            Ok(CapturedFrame {
                width: self.width,
                height: self.height,
                pixel_format: "JPEG".to_string(),
                captured_at: Utc::now(),
                frame_sequence: self.frame_sequence,
                bytes: jpeg_bytes,
                capture_duration: start_time.elapsed(),
            })
        }
    }

    fn stop(&mut self) {
        self.is_running = false;
    }

    fn source_info(&self) -> CaptureSourceInfo {
        CaptureSourceInfo {
            name: "Native Windows GDI Capturer".to_string(),
            width: self.width,
            height: self.height,
        }
    }
}

// =========================================================================
// Fallback Cross-Platform Capturer
// =========================================================================
pub struct FallbackCapturer {
    width: u32,
    height: u32,
    frame_sequence: u64,
    is_running: bool,
}

impl FallbackCapturer {
    pub fn new() -> Self {
        Self {
            width: 1280,
            height: 720,
            frame_sequence: 0,
            is_running: false,
        }
    }
}

impl ScreenCapturer for FallbackCapturer {
    fn start(&mut self) -> Result<(), CaptureError> {
        self.is_running = true;
        Ok(())
    }

    fn next_frame(&mut self) -> Result<CapturedFrame, CaptureError> {
        if !self.is_running {
            return Err(CaptureError::CaptureFailed("Capturer not started".into()));
        }

        let start_time = Instant::now();
        self.frame_sequence += 1;

        // On fallback platforms, we construct a real JPEG buffer containing a simple graphic pattern
        // rather than doing nothing, ensuring it compiles and can be decoded normally.
        let mut jpeg_bytes = Vec::new();
        {
            let pixel_count = (self.width * self.height) as usize;
            let mut rgb_pixels = vec![120u8; pixel_count * 3];
            
            // Draw a moving box pattern based on frame sequence to make it visibly "dynamic"
            let offset_x = (self.frame_sequence * 8) % (self.width as u64 - 100);
            let offset_y = (self.frame_sequence * 4) % (self.height as u64 - 100);
            for y in 0..100 {
                for x in 0..100 {
                    let px = (offset_y + y) as usize * self.width as usize + (offset_x + x) as usize;
                    if px * 3 + 2 < rgb_pixels.len() {
                        rgb_pixels[px * 3] = 58;    // R
                        rgb_pixels[px * 3 + 1] = 126; // G
                        rgb_pixels[px * 3 + 2] = 235; // B
                    }
                }
            }

            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_bytes, 60);
            let _ = encoder.encode(&rgb_pixels, self.width, self.height, image::ColorType::Rgb8);
        }

        std::thread::sleep(Duration::from_millis(33)); // cap to ~30 FPS for local simulation

        Ok(CapturedFrame {
            width: self.width,
            height: self.height,
            pixel_format: "Development Fallback JPEG".to_string(),
            captured_at: Utc::now(),
            frame_sequence: self.frame_sequence,
            bytes: jpeg_bytes,
            capture_duration: start_time.elapsed(),
        })
    }

    fn stop(&mut self) {
        self.is_running = false;
    }

    fn source_info(&self) -> CaptureSourceInfo {
        CaptureSourceInfo {
            name: "Fallback Software Capturer".to_string(),
            width: self.width,
            height: self.height,
        }
    }
}

/// Returns a suitable screen capturer for the current platform
pub fn create_default_capturer() -> Box<dyn ScreenCapturer> {
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsGdiCapturer::new())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Box::new(FallbackCapturer::new())
    }
}

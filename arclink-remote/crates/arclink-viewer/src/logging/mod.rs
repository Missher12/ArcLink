use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub struct ViewerLogger {
    logs: Vec<String>,
    log_file_path: PathBuf,
}

impl ViewerLogger {
    pub fn new() -> Self {
        let mut path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        path.push("arclink_viewer.log");
        Self {
            logs: Vec::new(),
            log_file_path: path,
        }
    }

    pub fn log(&mut self, level: &str, msg: &str) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let log_line = format!("[{}] [{}] {}", timestamp, level, msg);
        
        // Ring buffer for UI
        self.logs.push(log_line.clone());
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }

        // Print to stdout
        println!("{}", log_line);

        // Append to file
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)
        {
            let _ = writeln!(file, "{}", log_line);
        }
    }

    pub fn info(&mut self, msg: &str) {
        self.log("INFO", msg);
    }

    pub fn warn(&mut self, msg: &str) {
        self.log("WARN", msg);
    }

    pub fn error(&mut self, msg: &str) {
        self.log("ERROR", msg);
    }

    pub fn get_ui_logs(&self) -> &[String] {
        &self.logs
    }

    pub fn clear(&mut self) {
        self.logs.clear();
    }

    pub fn file_path_str(&self) -> String {
        self.log_file_path.to_string_lossy().to_string()
    }
}

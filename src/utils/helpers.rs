use crate::components::{Toast, ToastType};
use crate::AppContext;
use dioxus::prelude::*;

/// Format duration from milliseconds to MM:SS
pub fn format_duration(ms: u32) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{}:{:02}", minutes, seconds)
}

/// Sanitize filename by replacing invalid characters
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// Pick a JSON file using native file picker
pub fn pick_json_file() -> Option<String> {
    use std::process::Command;

    // Use native file picker based on OS
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(r#"POSIX path of (choose file with prompt "Select a playlist JSON file" of type {"public.json"})"#)
            .output()
            .ok()?;

        if output.status.success() {
            let path = String::from_utf8(output.stdout).ok()?;
            let trimmed = path.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("zenity")
            .args(&["--file-selection", "--file-filter=*.json"])
            .output()
            .ok()?;

        if output.status.success() {
            let path = String::from_utf8(output.stdout).ok()?;
            return Some(path.trim().to_string());
        }
    }

    #[cfg(target_os = "windows")]
    {
        // For Windows, we'll use a simple dialog
        println!("Please enter the full path to the JSON file:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok()?;
        return Some(input.trim().to_string());
    }

    None
}

/// Save file using native save dialog
pub fn save_json_file(default_filename: &str) -> Option<String> {
    use std::process::Command;

    // Use native save dialog based on OS
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"POSIX path of (choose file name with prompt "Save playlist as" default name "{}")"#,
            default_filename
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .ok()?;

        if output.status.success() {
            let path = String::from_utf8(output.stdout).ok()?;
            let trimmed = path.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("zenity")
            .args(&[
                "--file-selection",
                "--save",
                "--confirm-overwrite",
                &format!("--filename={}", default_filename),
                "--file-filter=*.json"
            ])
            .output()
            .ok()?;

        if output.status.success() {
            let path = String::from_utf8(output.stdout).ok()?;
            return Some(path.trim().to_string());
        }
    }

    #[cfg(target_os = "windows")]
    {
        // For Windows, we'll use a simple dialog
        println!("Enter the full path where you want to save {} (or press Enter for default):", default_filename);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok()?;
        let path = input.trim();

        if path.is_empty() {
            // Use default Downloads location
            let home_dir = std::env::var("USERPROFILE").ok()?;
            return Some(format!("{}\\Downloads\\{}", home_dir, default_filename));
        }

        return Some(path.to_string());
    }

    None
}

/// Import playlist from JSON file
pub fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

// Toast helper functions
pub fn show_toast(context: &AppContext, message: String, toast_type: ToastType) {
    let mut toasts = context.toasts;
    let mut counter = context.toast_counter;

    let id = counter();
    counter.set(id + 1);

    let toast = Toast {
        message,
        toast_type,
        id,
    };

    let mut current_toasts = toasts();
    current_toasts.push(toast.clone());
    toasts.set(current_toasts);

    // Auto-remove toast after 5 seconds
    spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        let mut current_toasts = toasts();
        current_toasts.retain(|t| t.id != id);
        toasts.set(current_toasts);
    });
}

pub fn show_success(context: &AppContext, message: String) {
    show_toast(context, message, ToastType::Success);
}

pub fn show_error(context: &AppContext, message: String) {
    show_toast(context, message, ToastType::Error);
}

pub fn show_info(context: &AppContext, message: String) {
    show_toast(context, message, ToastType::Info);
}
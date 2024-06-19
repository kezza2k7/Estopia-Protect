extern crate walkdir;
extern crate regex;

use walkdir::WalkDir;
use std::fs;
use regex::Regex;
use std::process::Command;
use std::{thread, time};

fn main() {
    let target_file_name = "login.pass"; // The file to look for on the USB device

    loop {
        if !is_target_file_present(target_file_name) {
            println!("USB not present or file not found. Locking system...");
            lock_system();
        }

        // Sleep for a while before checking again
        thread::sleep(time::Duration::from_secs(2));
    }
}

fn is_target_file_present(file_name: &str) -> bool {
    // List all drives and check if the file exists on any of them
    let drives = get_usb_drives();
    for drive in drives {
        println!("Checking drive: {}", drive);
        if is_file_on_drive(&drive, file_name) {
            return true;
        }
    }
    false
}

fn get_usb_drives() -> Vec<String> {
    let mut drives = Vec::new();

    // For Windows
    if cfg!(target_os = "windows") {
        let output = Command::new("wmic")
            .args(&["logicaldisk", "get", "name,description"])
            .output()
            .expect("Failed to execute WMIC command");

        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("Removable Disk") {
                let re = Regex::new(r"\s{2,}").unwrap();
                let parts: Vec<&str> = re.splitn(line, 2).collect();
                if parts.len() > 1 {
                    let drive = parts[1].trim_end_matches(':').trim().to_string();
                    drives.push(drive);
                }
            }
        }
    } else {
        // For Linux/MacOS
        let base_dirs = ["/media", "/run/media", "/mnt"];
        for base_dir in &base_dirs {
            if let Ok(entries) = fs::read_dir(base_dir) {
                for entry in entries.flatten() {
                    drives.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }

    println!("USB drives: {:?}", drives);
    drives
}

fn is_file_on_drive(drive: &str, file_name: &str) -> bool {
    for entry in WalkDir::new(drive).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            if entry.file_name() == file_name {
                println!("Found the file!");
                return true;
            }
        }
    }
    println!("Finished checking all files and directories. Didn't find the file.");
    return false;
}

fn lock_system() {
    // Lock the system depending on the operating system
    println!("Locking system...");
    if cfg!(target_os = "windows") {
        Command::new("rundll32.exe")
            .arg("user32.dll,LockWorkStation")
            .output()
            .expect("Failed to lock workstation");
    } else if cfg!(target_os = "linux") {
        Command::new("systemctl")
            .arg("restart")
            .arg("display-manager")
            .output()
            .expect("Failed to lock screen");
    } else if cfg!(target_os = "macos") {
        Command::new("/System/Library/CoreServices/Menu Extras/User.menu/Contents/Resources/CGSession")
            .arg("-suspend")
            .output()
            .expect("Failed to lock screen");
    } else {
        eprintln!("Unsupported operating system");
    }
}

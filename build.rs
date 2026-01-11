use std::{fs};

fn main() {
    // Simple ad‑hoc parser: KEY=VALUE per line.
    let env_path = ".env";
    if let Ok(content) = fs::read_to_string(env_path) {
        for line in content.lines() {
            if let Some((k, v)) = line.split_once('=') {
                println!("cargo:rustc-env={}={}", k.trim(), v.trim());
            }
        }
    } else {
        // Optional: fallback or build error
        println!("cargo:warning=.env not found; using dummy wifi creds");
        println!("cargo:rustc-env=WIFI_SSID=dummy");
        println!("cargo:rustc-env=WIFI_PASSWORD=dummy");
    }
    embuild::espidf::sysenv::output();
}
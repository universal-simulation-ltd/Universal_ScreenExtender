//! Read the Mac's current Wi-Fi network so the GUI can embed the SSID in the
//! combined connect QR (https URL fragment) and show the network name in the UI.
//!
//! Uses the `airport` CLI — private but present on macOS 10.7 through 14.
//! Password extraction requires a keychain dialog and is deferred to a later
//! version; the connect QR embeds the host address + PIN regardless, so the
//! phone can connect once it's on the same network.

use std::process::Command;

/// The current Wi-Fi network.
pub struct WifiInfo {
    pub ssid: String,
    /// Cleartext key if we could read it (currently always `None` on macOS
    /// without a keychain prompt — left in the struct for future use).
    pub password: Option<String>,
    /// QR auth tag: `"WPA"`, `"WEP"`, or `"nopass"`.
    pub auth: String,
}

impl WifiInfo {
    pub fn masked_password(&self) -> Option<String> {
        self.password.as_ref().map(|p| "•".repeat(p.chars().count().min(12)))
    }
}

pub fn current_wifi() -> Option<WifiInfo> {
    let ssid = current_ssid()?;
    Some(WifiInfo { ssid, password: None, auth: "WPA".to_owned() })
}

/// SSID of the connected Wi-Fi interface, via the `airport` utility.
fn current_ssid() -> Option<String> {
    let airport = "/System/Library/PrivateFrameworks/Apple80211.framework\
                   /Versions/Current/Resources/airport";
    let out = Command::new(airport).arg("-I").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        let line = line.trim();
        // airport -I outputs lines like "  SSID: MyNetwork"
        if let Some(rest) = line.strip_prefix("SSID: ") {
            let ssid = rest.trim().to_owned();
            if !ssid.is_empty() {
                return Some(ssid);
            }
        }
    }
    None
}


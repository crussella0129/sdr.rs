//! Hamlib Rigctl TCP command protocol engine (default port 4532).

use sdr_core::traits::{Result, SdrError};

/// Rig operational modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigMode {
    AM,
    FM,
    WFM,
    CW,
    USB,
    LSB,
    RAW,
}

impl RigMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            RigMode::AM => "AM",
            RigMode::FM => "FM",
            RigMode::WFM => "WFM",
            RigMode::CW => "CW",
            RigMode::USB => "USB",
            RigMode::LSB => "LSB",
            RigMode::RAW => "RAW",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "AM" => Some(RigMode::AM),
            "FM" => Some(RigMode::FM),
            "WFM" => Some(RigMode::WFM),
            "CW" => Some(RigMode::CW),
            "USB" => Some(RigMode::USB),
            "LSB" => Some(RigMode::LSB),
            "RAW" => Some(RigMode::RAW),
            _ => None,
        }
    }
}

/// Transceiver State for Rigctl Control.
#[derive(Debug, Clone)]
pub struct RigState {
    pub frequency_hz: u64,
    pub mode: RigMode,
    pub passband_width_hz: u32,
    pub vfo: String,
}

impl Default for RigState {
    fn default() -> Self {
        Self {
            frequency_hz: 144_000_000,
            mode: RigMode::FM,
            passband_width_hz: 15_000,
            vfo: "VFOA".to_string(),
        }
    }
}

/// Handler for Hamlib Rigctl commands.
pub struct RigctlHandler {
    state: RigState,
}

impl RigctlHandler {
    pub fn new(initial_state: RigState) -> Self {
        Self {
            state: initial_state,
        }
    }

    /// Process a line from a Hamlib client (e.g. "f\n", "F 144200000\n", "m\n", "M USB 2800\n", "\dump_state\n")
    /// and return the compliant response string.
    pub fn handle_command(&mut self, cmd_line: &str) -> Result<String> {
        let trimmed = cmd_line.trim();
        if trimmed.is_empty() {
            return Ok(String::new());
        }

        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = tokens[0];

        match cmd {
            // Get frequency
            "f" | "\\get_freq" => Ok(format!("{}\n", self.state.frequency_hz)),

            // Set frequency: F <freq_hz>
            "F" | "\\set_freq" => {
                if tokens.len() < 2 {
                    return Err(SdrError::Protocol(
                        "Missing frequency parameter".to_string(),
                    ));
                }
                let freq: u64 = tokens[1]
                    .parse()
                    .map_err(|_| SdrError::Protocol("Invalid frequency value".to_string()))?;
                self.state.frequency_hz = freq;
                Ok("RPRT 0\n".to_string())
            }

            // Get mode
            "m" | "\\get_mode" => Ok(format!(
                "{}\n{}\n",
                self.state.mode.as_str(),
                self.state.passband_width_hz
            )),

            // Set mode: M <mode> <width>
            "M" | "\\set_mode" => {
                if tokens.len() < 2 {
                    return Err(SdrError::Protocol("Missing mode parameter".to_string()));
                }
                if let Some(m) = RigMode::from_str(tokens[1]) {
                    self.state.mode = m;
                    if tokens.len() >= 3 {
                        if let Ok(w) = tokens[2].parse::<u32>() {
                            self.state.passband_width_hz = w;
                        }
                    }
                    Ok("RPRT 0\n".to_string())
                } else {
                    Err(SdrError::Protocol(format!("Unknown mode: {}", tokens[1])))
                }
            }

            // Get VFO
            "v" | "\\get_vfo" => Ok(format!("{}\n", self.state.vfo)),

            // Dump state (queried by GPredict and WSJT-X)
            "\\dump_state" => {
                let mut out = String::new();
                out.push_str("0\n"); // Protocol version
                out.push_str("2\n"); // Rig model (SDR)
                out.push_str("100000 6000000000 0x1ef -1 -1 0x1 0x0\n"); // Frequency range 100 kHz - 6 GHz
                out.push_str("0 0 0 0 0 0 0\n");
                out.push_str("0x1ef 1\n");
                out.push_str("0x1ef 0\n");
                out.push_str("0 0\n");
                out.push_str("0 0\n");
                out.push_str("RPRT 0\n");
                Ok(out)
            }

            // Quit
            "q" => Ok("".to_string()),

            _ => Ok("RPRT -1\n".to_string()),
        }
    }

    pub fn state(&self) -> &RigState {
        &self.state
    }
}

use std::{fs::File, io::Error};
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};

use crate::{config_dir, LOCALE};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum StatMode {
    Main,
    Aux,
    Both,
    Combined
}

impl StatMode {
    pub const fn all() -> &'static [StatMode] {
        &[StatMode::Combined, StatMode::Both, StatMode::Main, StatMode::Aux]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            StatMode::Main => "Show main only",
            StatMode::Aux => "Show auxiliary only",
            StatMode::Both => "Show both",
            StatMode::Combined => "Show combined",
        }
    }

    pub fn from_str(name: &str) -> Option<StatMode> {
        match name {
            "Show main only" => Some(StatMode::Main),
            "Show auxiliary only" => Some(StatMode::Aux),
            "Show both" => Some(StatMode::Both),
            "Show combined" => Some(StatMode::Combined),
            _ => None
        }
    }

    pub fn format_fails(buf: &mut String, show_fails: bool, fails: u64) {
        if show_fails && fails != 0 {
            buf.push_str(" (");
            buf.push_str(&fails.to_formatted_string(&LOCALE));
            buf.push_str(" failed)");
        }
    }

    pub fn format_with_fails(self, buf: &mut String, main_value: u64, aux_value: u64, show_fails: bool, main_fails: u64, aux_fails: u64) {
        match self {
            StatMode::Main => {
                buf.push_str(&main_value.to_formatted_string(&LOCALE));
                Self::format_fails(buf, show_fails, main_fails);
            }
            StatMode::Aux => {
                buf.push_str(&aux_value.to_formatted_string(&LOCALE));
                buf.push_str(" Aux");
                Self::format_fails(buf, show_fails, aux_fails);
            }
            StatMode::Combined => {
                buf.push_str(&(main_value + aux_value).to_formatted_string(&LOCALE));
                Self::format_fails(buf, show_fails, main_fails + aux_fails);
            }
            StatMode::Both => {
                if aux_value == 0 {
                    buf.push_str(&main_value.to_formatted_string(&LOCALE));
                    Self::format_fails(buf, show_fails, main_fails);
                } else if main_value == 0 {
                    buf.push_str(&aux_value.to_formatted_string(&LOCALE));
                    buf.push_str(" Aux");
                    Self::format_fails(buf, show_fails, aux_fails);
                } else {
                    buf.push_str(&main_value.to_formatted_string(&LOCALE));
                    Self::format_fails(buf, show_fails, main_fails);
                    buf.push_str(" + ");
                    buf.push_str(&aux_value.to_formatted_string(&LOCALE));
                    buf.push_str(" Aux");
                    Self::format_fails(buf, show_fails, aux_fails);
                }
            }
        }
    }

    pub fn format(self, buf: &mut String, main_value: u64, aux_value: u64) {
        self.format_with_fails(buf, main_value, aux_value, false, 0, 0);
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CallStackDepthMode {
    Enabled,
    WithMax
}

impl CallStackDepthMode {
    pub const fn all() -> &'static [CallStackDepthMode] {
        &[CallStackDepthMode::Enabled, CallStackDepthMode::WithMax]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            CallStackDepthMode::Enabled => "Current depth only",
            CallStackDepthMode::WithMax => "Current and max depth",
        }
    }

    pub fn from_str(name: &str) -> Option<CallStackDepthMode> {
        match name {
            "Current depth only" => Some(CallStackDepthMode::Enabled),
            "Current and max depth" => Some(CallStackDepthMode::WithMax),
            _ => None
        }
    }

    pub fn format(self, buf: &mut String, depth: usize, max_depth: usize) {
        match self {
            CallStackDepthMode::Enabled => {
                buf.push_str(&depth.to_formatted_string(&LOCALE));
            }
            CallStackDepthMode::WithMax => {
                buf.push_str(&depth.to_formatted_string(&LOCALE));
                buf.push_str(" (");
                buf.push_str(&max_depth.to_formatted_string(&LOCALE));
                buf.push_str(" max)");
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct StatsSettings {
    pub array_length: bool,
    pub currently_running: bool,
    pub writes: Option<StatMode>,
    pub swaps: Option<StatMode>,
    pub failed_swaps: bool,
    pub reads: Option<StatMode>,
    pub comparisons: bool,
    pub failed_comparisons: bool,
    pub time: bool,
    pub call_stack_depth: Option<CallStackDepthMode>,
    pub recursion_depth: Option<CallStackDepthMode>,
}

impl StatsSettings {
    pub fn new() -> Self {
        StatsSettings {
            array_length: true,
            currently_running: true,
            writes: Some(StatMode::Combined),
            swaps: Some(StatMode::Combined),
            failed_swaps: true,
            reads: Some(StatMode::Combined),
            comparisons: true,
            failed_comparisons: true,
            time: true,
            call_stack_depth: None,
            recursion_depth: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Unreliability {
    pub enabled: bool,
    pub comparisons: f64,
    pub swaps: f64,
}

impl Unreliability {
    pub fn new() -> Self {
        Self {
            enabled: false,
            comparisons: 0.0,
            swaps: 0.0,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.comparisons < 0.0 || self.comparisons > 1.0 {
            Err(Error::other("Invalid comparisons unreliability rate value"))
        } else if self.swaps < 0.0 || self.swaps > 1.0 {
            Err(Error::other("Invalid swaps unreliability rate value"))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Render {
    pub enabled: bool,
    pub fps: u16,
    pub bitrate: u32,
    pub profile: String,
    pub save_timestamps: bool,
}

impl Render {
    pub fn new() -> Self {
        Self {
            enabled: false,
            fps: 60,
            bitrate: 8000,
            profile: String::from("h264-sw"),
            save_timestamps: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct UniVSettings {
    pub show_text:          bool,
    pub show_aux:           bool,
    pub precise_highlights: bool,
    pub play_sound:         bool,
    pub reverb:             bool,
    pub internal_info:      bool,
    pub sound:              String,
    pub resolution:         [u16; 2],
    pub render:             Render,
    pub stats:              StatsSettings,
    pub unreliability:      Unreliability
}

impl UniVSettings {
    pub fn new() -> Self {
        Self {
            show_text:          true,
            show_aux:           true,
            precise_highlights: true,
            play_sound:         true,
            reverb:             false,
            internal_info:      false,
            sound:              String::from("Default"),
            resolution:         [1280, 720],
            render:             Render::new(),
            stats:              StatsSettings::new(),
            unreliability:      Unreliability::new()
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.unreliability.validate()?;
        Ok(())
    }

    pub fn load() -> Result<Self, Error> {
        let f = File::open(config_dir!().join("UniV.json"))?;
        let settings: UniVSettings = serde_json::from_reader(f).map_err(|e| Error::other(e))?;
        settings.validate()?;
        Ok(settings)
    }

    pub fn save(&self) -> Result<(), Error> {
        let f = File::create(config_dir!().join("UniV.json"))?;
        serde_json::to_writer_pretty(f, self).map_err(|e| Error::other(e))
    }
}

#[macro_export]
macro_rules! profiles_dir {
    () => {
        $crate::program_dir!().join("profiles")
    };
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub codec:   String,
    pub pix_fmt: String,
    pub preset:  String,
    pub tune:    String,
    pub profile: String
}

impl Profile {
    pub fn new() -> Self {
        Self {
            codec:   String::from("libx264"),
            pix_fmt: String::from("yuv420p"),
            preset:  String::from("slower"),
            tune:    String::from("film"),
            profile: String::from("high")
        }
    }

    pub fn load(name: &String) -> Result<Self, Error> {
        let f = File::open(profiles_dir!().join(format!("{name}.json")))?;
        serde_json::from_reader(f).map_err(|e| Error::other(e))
    }
}
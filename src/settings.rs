use std::{fs::File, io::Error};
use num_format::ToFormattedString;
use serde::{Deserialize, Serialize};

use crate::{config_dir, LOCALE};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

    pub fn format(self, main_value: u64, aux_value: u64) -> String {
        match self {
            StatMode::Main => main_value.to_formatted_string(&LOCALE),
            StatMode::Aux  =>  aux_value.to_formatted_string(&LOCALE),
            StatMode::Combined => {
                (main_value + aux_value).to_formatted_string(&LOCALE)
            }
            StatMode::Both => {
                if aux_value == 0 {
                    main_value.to_formatted_string(&LOCALE)
                } else if main_value == 0 {
                    format!("{} Aux", aux_value.to_formatted_string(&LOCALE))
                } else {
                    format!(
                        "{} + {} Aux",
                        main_value.to_formatted_string(&LOCALE),
                        aux_value.to_formatted_string(&LOCALE)
                    )
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RecursionDepthMode {
    Enabled,
    WithMax
}

impl RecursionDepthMode {
    pub const fn all() -> &'static [RecursionDepthMode] {
        &[RecursionDepthMode::Enabled, RecursionDepthMode::WithMax]
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            RecursionDepthMode::Enabled => "Current depth only",
            RecursionDepthMode::WithMax => "Current and max depth",
        }
    }

    pub fn from_str(name: &str) -> Option<RecursionDepthMode> {
        match name {
            "Current depth only" => Some(RecursionDepthMode::Enabled),
            "Current and max depth" => Some(RecursionDepthMode::WithMax),
            _ => None
        }
    }

    pub fn format(self, depth: usize, max_depth: usize) -> String {
        match self {
            RecursionDepthMode::Enabled => depth.to_formatted_string(&LOCALE),
            RecursionDepthMode::WithMax => {
                format!(
                    "{} ({} max)",
                    depth.to_formatted_string(&LOCALE),
                    max_depth.to_formatted_string(&LOCALE)
                )
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct StatsSettings {
    pub array_length: bool,
    pub currently_running: bool,
    pub writes: Option<StatMode>,
    pub swaps: Option<StatMode>,
    pub reads: Option<StatMode>,
    pub comparisons: bool,
    pub time: bool,
    pub recursion_depth: Option<RecursionDepthMode>,
}

impl StatsSettings {
    pub fn new() -> Self {
        StatsSettings {
            array_length: true,
            currently_running: true,
            writes: Some(StatMode::Combined),
            swaps: Some(StatMode::Combined),
            reads: Some(StatMode::Combined),
            comparisons: true,
            time: true,
            recursion_depth: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct UniVSettings {
    pub show_text:          bool,
    pub show_aux:           bool,
    pub precise_highlights: bool,
    pub play_sound:         bool,
    pub reverb:             bool,
    pub render:             bool,
    pub internal_info:      bool,
    pub save_timestamps:    bool,
    pub bitrate:            u32,
    pub render_fps:         u16,
    pub profile:            String,
    pub sound:              String,
    pub resolution:         [u16; 2],
    pub stats:              StatsSettings
}

impl UniVSettings {
    pub fn new() -> Self {
        Self {
            show_text:          true,
            show_aux:           true,
            precise_highlights: true,
            play_sound:         true,
            reverb:             false,
            render:             false,
            internal_info:      false,
            save_timestamps:    false,
            bitrate:            8000,
            render_fps:         60,
            profile:            String::from("h264-sw"),
            sound:              String::from("Default"),
            resolution:         [1280, 720],
            stats:              StatsSettings::new()
        }
    }

    pub fn load() -> Result<Self, Error> {
        let f = File::open(config_dir!().join("UniV.json"))?;
        serde_json::from_reader(f).map_err(|e| Error::other(e))
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
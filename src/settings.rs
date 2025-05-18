use std::{fs::File, io::Error};
use serde::{Deserialize, Serialize};

use crate::config_dir;

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
    pub bitrate:            u32,
    pub render_fps:         u16,
    pub profile:            String,
    pub sound:              String,
    pub resolution:         [u16; 2]
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
            bitrate:            8000,
            render_fps:         60,
            profile:            String::from("h264-sw"),
            sound:              String::from("Default"),
            resolution:         [1280, 720],
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
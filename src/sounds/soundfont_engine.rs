use std::{fs::File, io::{Cursor, Error}, path::Path, rc::Rc};

use raylib::{RaylibHandle, RaylibThread};
use rustysynth::SoundFont;
use serde::{Deserialize, Serialize};

use crate::{config_dir, gui::{FileOption, Gui}, sound, sounds::default_engine, univm::object::ExecutionInterrupt, utils::sound::BaseSoundFontEngine, DEFAULT_SOUNDFONT};

macro_rules! soundfont_config_file {
    () => {
        config_dir!().join("Soundfont.json")
    };
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SoundfontEngineSettings {
    pub file:   String,
    pub bank:   i32,
    pub preset: i32
}

impl SoundfontEngineSettings {
    pub fn load() -> Result<Self, Error> {
        let f = File::open(soundfont_config_file!())?;
        serde_json::from_reader(f).map_err(|e| Error::other(e))
    }

    pub fn save(&self) -> Result<(), Error> {
        let f = File::create(soundfont_config_file!())?;
        serde_json::to_writer_pretty(f, self).map_err(|e| Error::other(e))
    }
}

pub struct SoundfontEngine {
    base: BaseSoundFontEngine
}

impl SoundfontEngine {
    fn load_soundfont<T: AsRef<Path>>(path: &T, gui: &mut Gui, rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<Option<SoundFont>, ExecutionInterrupt> {
        let mut file = {
            match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    gui.build_fn = Gui::popup;
                    gui.popup.set(
                        "Soundfont sound engine",
                        format!("Unable to open soundfont file:\n{}", e).as_str()
                    ).unwrap();
                    gui.run(rl, thread)?;
                    return Ok(None);
                }
            }
        };

        Ok(match SoundFont::new(&mut file) {
            Ok(sf) => Some(sf),
            Err(e) => {
                gui.build_fn = Gui::popup;
                gui.popup.set(
                    "Soundfont sound engine",
                    format!("Unable to load soundfont file:\n{}", e).as_str()
                ).unwrap();
                gui.run(rl, thread)?;
                None
            }
        })
    }
}

sound! {
    name = "Soundfont";

    SoundfontEngine::new(self) {
        SoundfontEngine { 
            base: BaseSoundFontEngine::new() 
        }
    }

    prepare(shared, gui, rl, thread) {
        if soundfont_config_file!().exists() {
            let config = {
                match SoundfontEngineSettings::load() {
                    Ok(conf) => conf,
                    Err(e) => {
                        gui.build_fn = Gui::popup;
                        gui.popup.set(
                            "Soundfont sound engine",
                            format!("Unable to load soundfont configuration:\n{}", e).as_str()
                        ).unwrap();
                        gui.run(rl, thread)?;
                        return Ok(());
                    }
                }
            };

            let soundfont = {
                if let Some(sf) = Self::load_soundfont(&config.file, gui, rl, thread)? {
                    sf
                } else {
                    return Ok(());
                }
            };

            self.base.prepare(shared, soundfont, config.bank, config.preset);
        } else {
            let soundfont = SoundFont::new(&mut Cursor::new(DEFAULT_SOUNDFONT))
                .expect("Could not load default SoundFont");
            self.base.prepare(shared, soundfont, default_engine::BANK, default_engine::PRESET);
        }
    }

    config(shared, gui, rl, thread) {
        gui.build_fn = Gui::popup;
        gui.popup.set("Soundfont sound engine", "Select soundfont file to load").unwrap();
        gui.run(rl, thread)?;

        let mut sf_path = None;
        let soundfont;

        loop {
            gui.build_fn = Gui::file_dialog;
            gui.file_dialog.set("Soundfont sound engine", false).unwrap();
            gui.run(rl, thread)?;
            
            match gui.file_dialog.selected.clone() {
                FileOption::Some(path) => {
                    if let Some(sf) = Self::load_soundfont(&path, gui, rl, thread)? {
                        sf_path = Some(path);
                        soundfont = sf;
                        break;
                    } else {
                        return Ok(());
                    }
                }
                FileOption::Canceled => {
                    gui.build_fn = Gui::popup;
                    gui.popup.set("Soundfont sound engine", "Canceled. Using default soundfont").unwrap();
                    gui.run(rl, thread)?;

                    soundfont = SoundFont::new(&mut Cursor::new(DEFAULT_SOUNDFONT))
                        .expect("Could not load default SoundFont");
                    break;
                }
                FileOption::None => {
                    gui.build_fn = Gui::popup;
                    gui.popup.set("Soundfont sound engine", "No soundfont file selected").unwrap();
                    gui.run(rl, thread)?;
                    continue;
                }
            }
        }

        let instruments = soundfont.get_presets();
        let instrument_names = instruments.iter().map(|x| Rc::from(x.get_name())).collect();

        gui.build_fn = Gui::selection;
        gui.selection.set("Soundfont sound engine", "Select instrument", instrument_names, 0).unwrap();
        gui.run(rl, thread)?;

        let bank = instruments[gui.selection.index].get_bank_number();
        let preset = instruments[gui.selection.index].get_patch_number();

        self.base.prepare(shared, soundfont, bank, preset);

        if let Some(path) = sf_path {
            let settings = SoundfontEngineSettings { 
                file: path.to_str().expect("Could not convert PathBuf to str").to_string(), 
                bank,
                preset
            };

            if let Err(e) = settings.save() {
                gui.build_fn = Gui::popup;
                gui.popup.set(
                    "Soundfont sound engine",
                    format!("Unable to save soundfont configuration:\n{}", e).as_str()
                ).unwrap();
                gui.run(rl, thread)?;
                return Ok(());
            }
        }
    }

    play(value, max, length) {
        self.base.play(value, max, length)
    }
}
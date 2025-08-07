use std::{fs, rc::Rc};

use raylib::ffi::TraceLogLevel;

use crate::{config_dir, imgui_centered_text, imgui_spaced_separator, log, profiles_dir, settings::UniVSettings};

use super::Gui;

pub struct Settings {
    pub object: UniVSettings,
    pub sound_setup: bool,
    pub reset_pressed: bool,
    pub config_deleted: bool,
    pub reload_algos: bool,
    pub back: bool,

    pub profiles: Vec<Rc<str>>,
    pub configs:  Vec<Rc<str>>,

    pub curr_profile: usize,
    pub curr_config: usize,
    pub curr_sound: usize,
}

impl Settings {
    pub fn new() -> Self {
        Settings { 
            object: UniVSettings::new(), 
            sound_setup: false,
            reset_pressed: false,
            config_deleted: false,
            reload_algos: false,
            back: false,

            profiles: Vec::new(), 
            configs: Vec::new(),

            curr_profile: 0,
            curr_config: 0,
            curr_sound: 0
        }
    }

    pub fn partial_reset(&mut self) {
        self.sound_setup = false;
        self.reset_pressed = false;
        self.reload_algos = false;
    }

    fn load_profiles(&mut self) {
        let err = 'err: {
            match fs::read_dir(profiles_dir!()) {
                Ok(files) => {
                    self.profiles = Vec::new();
                    for file in files {
                        match file {
                            Ok(f) => {
                                let filename = f.file_name();
                                let name = filename.to_str().unwrap();
                                if name.ends_with(".json") {
                                    self.profiles.push(name.strip_suffix(".json").unwrap().into());
                                }
                            }
                            Err(e) => {
                                break 'err e.to_string();
                            }
                        }
                    }
                    
                    if self.profiles.is_empty() {
                        self.profiles.push(Rc::from("<no profiles>"));
                        self.curr_profile = 0;
                    } else {
                        self.profiles.sort();

                        if let Ok(index) = self.profiles.binary_search(&Rc::from(self.object.profile.as_str())) {
                            self.curr_profile = index;
                        } else {
                            self.curr_profile = 0;
                        }
                    }

                    return;
                }
                Err(e) => e.to_string(),
            }
        };
        
        log!(TraceLogLevel::LOG_ERROR, "Error listing profiles");
        log!(TraceLogLevel::LOG_ERROR, "    > {}", err.to_string());
        self.profiles = vec![Rc::from("<no profiles>")];
        self.curr_profile = 0;
    }

    pub fn load_configs(&mut self) {
        let err = 'err: {
            match fs::read_dir(config_dir!()) {
                Ok(files) => {
                    self.configs = Vec::new();
                    for file in files {
                        match file {
                            Ok(f) => {
                                let filename = f.file_name();
                                let name = filename.to_str().unwrap();
                                if name.ends_with(".json") {
                                    self.configs.push(name.strip_suffix(".json").unwrap().into());
                                }
                            }
                            Err(e) => {
                                break 'err e.to_string();
                            }
                        }
                    }
                    
                    self.curr_config = 0;
                    if self.configs.is_empty() {
                        self.configs.push(Rc::from("<no configs>"));
                    } else {
                        self.configs.sort();
                    }

                    return;
                }
                Err(e) => e.to_string(),
            }
        };
        
        log!(TraceLogLevel::LOG_ERROR, "Error listing configurations");
        log!(TraceLogLevel::LOG_ERROR, "    > {}", err.to_string());
        self.configs = vec![Rc::from("<no configs>")];
        self.curr_config = 0;
    }


    pub fn load(&mut self, settings: &UniVSettings, sounds: &[Rc<str>]) {
        self.object = settings.clone();

        self.sound_setup = false;
        self.reset_pressed = false;
        self.back = false;
        self.curr_sound = sounds.binary_search(&Rc::from(self.object.sound.as_str())).unwrap();

        self.load_profiles();
        self.load_configs();
    }
}

impl Gui {
    pub fn settings(&mut self) -> bool {
        let ui = self.imgui.new_frame();

        let mut quit = false;

        ui.window("Settings")
            .position(
                [
                    self.resolution_x / 2.0 - self.settings_window_size_x / 2.0, 
                    self.resolution_y / 2.0 - self.settings_window_size_y / 2.0
                ], 
                imgui::Condition::Appearing
            )
            .size(
                [
                    self.settings_window_size_x, 
                    self.settings_window_size_y
                ], 
                imgui::Condition::Appearing
            )
            .bg_alpha(Gui::ALPHA)
            .build(|| {
                imgui_centered_text!(ui, "Visual");
                imgui_spaced_separator!(ui);

                ui.checkbox("Show text", &mut self.settings.object.show_text);
                if ui.is_item_hovered() {
                    ui.tooltip(|| {
                        ui.text("Show statistics while running algorithms");
                    });
                }

                ui.checkbox("Show auxiliary arrays", &mut self.settings.object.show_aux);
                ui.checkbox("Show internal information", &mut self.settings.object.internal_info);
                if ui.is_item_hovered() {
                    ui.tooltip_text("Show extra information about the program's state");
                }

                ui.checkbox("Precise highlights", &mut self.settings.object.precise_highlights);
                if ui.is_item_hovered() {
                    ui.tooltip_text(concat!(
                        "Only shows highlights relative to each operation.\n\n",
                        "Offers better performance and more accurate visualizations,\n",
                        "but some users may find disabling this setting to be more aesthetically pleasing.\n",
                        "May affect visualization speed"
                    ));
                }

                ui.spacing();
                imgui_centered_text!(ui, "Graphics");
                imgui_spaced_separator!(ui);

                ui.input_scalar_n("Resolution (x, y)", &mut self.settings.object.resolution).build();

                ui.spacing();
                imgui_centered_text!(ui, "Sound");
                imgui_spaced_separator!(ui);

                ui.checkbox("Play sound", &mut self.settings.object.play_sound);
                ui.checkbox("Reverb and chorus", &mut self.settings.object.reverb);
                if ui.is_item_hovered() {
                    ui.tooltip(|| {
                        ui.text("Hints the sound engine to activate reverb and chorus. The engine may ignore this");
                    });
                }

                ui.spacing();

                if ui.button("Setup") {
                    self.settings.sound_setup = true;
                    quit = true;
                }

                if ui.is_item_hovered() {
                    ui.tooltip(|| {
                        ui.text("Runs the preparation script for the selected sound engine");
                    });
                }
                
                ui.same_line();

                ui.combo_simple_string("Sound", &mut self.settings.curr_sound, &self.sounds);

                ui.spacing();
                imgui_centered_text!(ui, "Render mode");
                imgui_spaced_separator!(ui);

                ui.checkbox("Enable (off is real time mode)", &mut self.settings.object.render);
                if ui.is_item_hovered() {
                    ui.tooltip(|| {
                        ui.text("Enables render mode");
                    });
                }
            
                ui.disabled(!self.settings.object.render, || {
                    ui.checkbox("Save timestamps", &mut self.settings.object.save_timestamps);
                    if ui.is_item_hovered() {
                        ui.tooltip(|| {
                            ui.text(concat!(
                                "Stores a list of timestamps in YouTube-compatible format as defined in the running automation.\n",
                                "The output is saved in 'timestamps.txt'.\n",
                                "Note that this only works in render mode"
                            ));
                        });
                    }
                });

                ui.input_scalar("FPS", &mut self.settings.object.render_fps).build();
                if ui.is_item_hovered() {
                    ui.tooltip(|| {
                        ui.text("Sets the video output FPS for render mode");
                    });
                }

                ui.input_scalar("Bitrate (kbps)", &mut self.settings.object.bitrate).build();
                ui.combo_simple_string("Profile", &mut self.settings.curr_profile, &self.settings.profiles);

                ui.spacing();
                imgui_centered_text!(ui, "Other");
                imgui_spaced_separator!(ui);

                if cfg!(not(feature = "lite")) {
                    if ui.button("Reload algorithms") {
                        self.settings.reload_algos = true;
                        quit = true;
                    }
                }

                ui.spacing();

                let config_name = Rc::clone(&self.settings.configs[self.settings.curr_config]);

                if self.settings.reset_pressed && config_name.as_ref() != "<no configs>" {
                    if ui.button("Cancel deletion") {
                        self.settings.reset_pressed = false;
                    }
                } else {
                    if ui.button("Reset configuration") {
                        self.settings.reset_pressed = true;
                    }

                    if ui.is_item_hovered() {
                        ui.tooltip(|| {
                            ui.text("Delete settings for the selected module");
                        });
                    }
                }

                ui.same_line();
                ui.combo_simple_string("##config", &mut self.settings.curr_config, &self.settings.configs);

                if self.settings.reset_pressed && 
                    config_name.as_ref() != "<no configs>" && 
                    ui.button(format!("Confirm deletion for '{config_name}'").as_str()) 
                {
                    self.settings.reset_pressed = false;
                    
                    if let Err(e) = fs::remove_file(config_dir!().join(format!("{config_name}.json"))) {
                        log!(TraceLogLevel::LOG_ERROR, "Error deleting configuration");
                        log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
                    } else {
                        self.settings.config_deleted = true;
                        self.settings.load_configs();
                    }
                }

                ui.set_cursor_pos([ui.cursor_pos()[0], ui.window_content_region_max()[1] - Gui::BACK_BUTTON_Y_SIZE]);
                if ui.button("Back") {
                    self.settings.back = true;
                    quit = true;
                }

                ui.same_line();
                ui.set_cursor_pos([
                    ui.window_content_region_max()[0] - Gui::SAVE_BUTTON_X_SIZE, 
                    ui.window_content_region_max()[1] - Gui::SAVE_BUTTON_Y_SIZE
                ]);
                
                if ui.button_with_size("Save", [Gui::SAVE_BUTTON_X_SIZE, Gui::SAVE_BUTTON_Y_SIZE]) {
                    quit = true;
                }
            });

        if quit && !self.settings.back {
            let profile = self.settings.profiles[self.settings.curr_profile].to_string();

            if profile.as_str() != "<no profiles>" {
                self.settings.object.profile = profile;
            }
            
            self.settings.object.sound = self.sounds[self.settings.curr_sound].to_string();
        }

        quit
    }
}
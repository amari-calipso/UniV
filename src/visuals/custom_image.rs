use std::{cell::OnceCell, collections::HashMap, fs::File, io::Error, rc::Rc};

use raylib::{color::Color, ffi::{PixelFormat, TraceLogLevel, Vector2}, math::Rectangle, texture::{Image, Texture2D}, RaylibHandle, RaylibThread};
use serde::{Deserialize, Serialize};

use crate::{get_expect, gui::{FileOption, Gui}, univm::object::ExecutionInterrupt, utils::{gfx::line_visual::LineVisual, translate}, visual, IdentityHashMap, DEFAULT_IMAGE, DEFAULT_IMAGE_FORMAT, LOG_LEVEL};

macro_rules! custom_image_config_file {
    () => {
        $crate::config_dir!().join("CustomImage.json")
    };
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CustomImageSettings {
    pub file: Option<String>,
}

impl CustomImageSettings {
    pub fn load() -> Result<Self, Error> {
        let f = File::open(custom_image_config_file!())?;
        serde_json::from_reader(f).map_err(|e| Error::other(e))
    }

    pub fn save(&self) -> Result<(), Error> {
        let f = File::create(custom_image_config_file!())?;
        serde_json::to_writer_pretty(f, self).map_err(|e| Error::other(e))
    }
}

pub struct CustomImage {
    line_visual: LineVisual,
    image: OnceCell<Image>,
    chunks: IdentityHashMap<usize, Texture2D>,
    aux_chunks: Vec<Texture2D>,
    aux_map_factor: f64
}

impl CustomImage {
    const HIGHLIGHT_ALPHA: u8 = 127;

    fn load_image(&mut self, file: &str, gui: &mut Gui, rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<(), ExecutionInterrupt> {
        match Image::load_image(file) {
            Ok(mut image) => {
                let width  = rl.get_screen_width();
                let height = rl.get_screen_height();
                image.resize(width, height);

                self.image.take();
                self.image.set(image).unwrap();
            }
            Err(e) => {
                gui.build_fn = Gui::popup;
                gui.popup.set(
                    "Custom Image visual style",
                    format!("Unable to load image:\n{}", e).as_str()
                ).unwrap();
                gui.run(rl, thread)?;
            }
        }

        Ok(())
    }

    fn load_default_image(&mut self, rl: &mut RaylibHandle) {
        let mut image = Image::load_image_from_mem(DEFAULT_IMAGE_FORMAT, DEFAULT_IMAGE)
            .expect("Could not load default image");

        let width  = rl.get_screen_width();
        let height = rl.get_screen_height();
        image.resize(width, height);
        image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);

        self.image.take();
        self.image.set(image).unwrap();
    }
}

visual! {
    name            = "Custom Image";
    highlight_color = Color::WHITE;

    CustomImage::new(self) {
        CustomImage { 
            line_visual: LineVisual::new(),
            image: OnceCell::new(),
            chunks: HashMap::default(),
            aux_chunks: Vec::new(),
            aux_map_factor: 0.0
        }
    }

    init(_shared, gui, rl, thread) {
        if cfg!(feature = "wasm") {
            self.load_default_image(rl);
            return Ok(());
        }

        loop {
            if custom_image_config_file!().exists() {
                let config = {
                    match CustomImageSettings::load() {
                        Ok(conf) => conf,
                        Err(e) => {
                            gui.build_fn = Gui::popup;
                            gui.popup.set(
                                "Custom Image visual style",
                                format!("Unable to load custom image configuration:\n{}", e).as_str()
                            ).unwrap();
                            gui.run(rl, thread)?;
                            return Ok(());
                        }
                    }
                };

                if config.file.is_none() {
                    self.load_default_image(rl);
                    return Ok(());
                }

                self.load_image(&config.file.unwrap(), gui, rl, thread)?;
                return Ok(());
            }

            gui.build_fn = Gui::selection;
            gui.selection.set(
                "Custom Image visual style",
                concat!(
                    "Do you want to set an image for the Custom Image visual style?\n",
                    "If you say no, this will not be asked again,\n",
                    "but you can change this setting later by resetting the 'CustomImage'\n",
                    "configuration through UniV's settings."
                ),
                [
                    "Yes", "No"
                ].into_iter().map(|x| Rc::from(x)).collect(),
                0
            ).unwrap();
            gui.run(rl, thread)?;

            let config;
            if gui.selection.index == 0 {
                loop {
                    gui.build_fn = Gui::file_dialog;
                    gui.file_dialog.set("Custom Image visual style", false).unwrap();
                    gui.run(rl, thread)?;

                    match gui.file_dialog.selected.clone() {
                        FileOption::Some(path) => {
                            let path = path.to_str().unwrap();
                            self.load_image(path, gui, rl, thread)?;
                            config = CustomImageSettings {
                                file: Some(String::from(path))
                            };
                        }
                        FileOption::Canceled => {
                            gui.build_fn = Gui::popup;
                            gui.popup.set(
                                "Custom Image visual style",
                                "Canceled. Using default image"
                            ).unwrap();
                            gui.run(rl, thread)?;

                            self.load_default_image(rl);
                            config = CustomImageSettings {
                                file: None
                            };
                        }
                        FileOption::None => {
                            gui.build_fn = Gui::popup;
                            gui.popup.set(
                                "Custom Image visual style",
                                "No image file selected"
                            ).unwrap();
                            gui.run(rl, thread)?;
                            continue;
                        }
                    }

                    break;
                }
            } else {
                config = CustomImageSettings {
                    file: None
                };
            }

            if let Err(e) = config.save() {
                gui.build_fn = Gui::popup;
                gui.popup.set(
                    "Custom Image visual style",
                    format!("Unable to save configuration:\n{}", e).as_str()
                ).unwrap();
                gui.run(rl, thread)?;
                return Ok(());
            }
        }
    }

    // suppresses noisy output from unloading thousands of chunks
    unload(rl, _thread) {
        rl.set_trace_log(TraceLogLevel::LOG_NONE);
        self.chunks.clear();
        self.aux_chunks.clear();
        rl.set_trace_log(LOG_LEVEL);
    }

    prepare(shared, rl, thread) {
        self.line_visual.prepare(shared, rl);

        rl.set_trace_log(TraceLogLevel::LOG_NONE);

        let length = shared.array.len();

        self.chunks.clear();

        for i in 0 .. length {
            let x = translate(
                i as f64, 
                0.0, length as f64, 
                0.0, self.line_visual.resolution_x as f64 - self.line_visual.rounded_line_width as f64
            ) as f32;

            let mut chunk = get_expect!(self.image).clone();
            chunk.crop(Rectangle {
                x, y: 0.0, 
                width: self.line_visual.rounded_line_width as f32,
                height: self.line_visual.resolution_y as f32
            });

            let texture = rl.load_texture_from_image(thread, &chunk)
                .expect("Could not load image chunk as texture");

            self.chunks.insert(shared.verify_array[i].idx, texture);
        }

        rl.set_trace_log(LOG_LEVEL);
    }

    on_aux_on(shared, rl, thread) {
        self.line_visual.on_aux_on(shared, rl);

        rl.set_trace_log(TraceLogLevel::LOG_NONE);

        let y = (self.line_visual.resolution_y / 2 - self.line_visual.top / 2) as f32;
        let line_width_f32 = self.line_visual.aux_rounded_line_width as f32;

        self.aux_chunks.clear();

        let mut x = 0f32;
        while x < self.line_visual.resolution_x as f32 {
            let mut chunk = get_expect!(self.image).clone();
            chunk.crop(Rectangle {
                x, y, 
                width: line_width_f32,
                height: self.line_visual.aux_resolution_y as f32
            });

            let texture = rl.load_texture_from_image(thread, &chunk)
                .expect("Could not load image chunk as texture");

            self.aux_chunks.push(texture);

            x += line_width_f32;
        }

        if self.aux_chunks.is_empty() {
            let mut chunk = get_expect!(self.image).clone();
            chunk.crop(Rectangle {
                x: 0.0, y, 
                width: chunk.width as f32,
                height: self.line_visual.aux_resolution_y as f32
            });

            let texture = rl.load_texture_from_image(thread, &chunk)
                .expect("Could not load image chunk as texture");

            self.aux_chunks.push(texture);
            self.aux_map_factor = 0.0;
        } else {
            self.aux_map_factor = self.aux_chunks.len() as f64 / (shared.aux_max + 1) as f64;
        }

        rl.set_trace_log(LOG_LEVEL);
    }
    
    on_aux_off(shared, rl, _thread) {
        self.line_visual.on_aux_off(shared, rl);
    }

    draw(shared, draw, indices) {
        let resolution_x_f64 = self.line_visual.resolution_x as f64;
        let length_f64 = shared.array.len() as f64;

        let mut x = 0.0;
        while x < resolution_x_f64 {
            let idx = translate(
                x, 
                0.0, resolution_x_f64,
                0.0, length_f64
            ) as usize;

            draw.draw_texture(
                &self.chunks[&shared.array[idx].idx], 
                x as i32, 
                self.line_visual.top - self.line_visual.top / 2, 
                Color::WHITE
            );

            x += self.line_visual.rounded_line_width as f64;
        }

        let mut last_i = 0usize;
        let mut x = 0usize;

        for i in 0 .. shared.array.len() {
            let width = (self.line_visual.line_width * (i + 1) as f64) as usize - x;
            if width == 0 {
                continue;
            }

            if let Some(mut color) = LineVisual::get_highlight_color(last_i, i, indices) {
                color.a = Self::HIGHLIGHT_ALPHA;
                draw.draw_rectangle(
                    x as i32, 
                    self.line_visual.top, 
                    width as i32, 
                    self.line_visual.resolution_y - self.line_visual.top, 
                    color
                );
            }

            x += width;
            last_i = i;
        }
    }

    draw_aux(shared, draw, indices) {
        let resolution_x_f64 = self.line_visual.resolution_x as f64;
        let length_f64 = shared.aux.len() as f64;

        let mut x = 0.0;
        while x < resolution_x_f64 {
            let idx = translate(
                x, 
                0.0, resolution_x_f64,
                0.0, length_f64
            ) as usize;

            draw.draw_texture(
                &self.aux_chunks[(shared.aux[idx].value as f64 * self.aux_map_factor) as usize], 
                x as i32, 
                0, 
                Color::WHITE
            );

            x += self.line_visual.aux_rounded_line_width as f64;
        }

        let mut last_i = 0usize;
        let mut x = 0usize;

        for i in 0 .. shared.aux.len() {
            let width = (self.line_visual.aux_line_width * (i + 1) as f64) as usize - x;
            if width == 0 {
                continue;
            }

            if let Some(mut color) = LineVisual::get_highlight_color(last_i, i, indices) {
                color.a = Self::HIGHLIGHT_ALPHA;
                draw.draw_rectangle(
                    x as i32, 
                    0,
                    width as i32, 
                    self.line_visual.aux_resolution_y, 
                    color
                );
            }

            x += width;
            last_i = i;
        }

        draw.draw_line_ex(
            Vector2 { x: 0.0, y: self.line_visual.aux_resolution_y as f32 }, 
            Vector2 { x: self.line_visual.resolution_x as f32, y: self.line_visual.aux_resolution_y as f32 },
            LineVisual::AUX_LINE_WIDTH,
            LineVisual::AUX_LINE_COLOR
        );
    }
}
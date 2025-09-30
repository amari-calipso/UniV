use std::{ffi::{c_char, c_void}, fs::File, io::{Error, Seek, SeekFrom, Write}};

use raylib::{color::Color, ffi::{PixelFormat, TraceLogLevel}, math::{Rectangle, Vector2}, prelude::{RaylibDraw, RaylibTextureModeExt}, texture::{Image, RaylibTexture2D, RenderTexture2D}, RaylibHandle, RaylibThread};
use serde::{Deserialize, Serialize};

use crate::{gui::Gui, log, program_dir, univm::object::ExecutionInterrupt, utils::gfx::{get_image_bytes, ImageFormat}, value::Value, Shared, LOG_LEVEL, REFERENCE_FRAMERATE};

use super::line_visual::LineVisual;

pub const DEFAULT_CONFIG_WINDOW_DIV_X: f32 = 3.0;
pub const DEFAULT_CONFIG_WINDOW_DIV_Y: f32 = 4.0;

#[macro_export]
macro_rules! data_trace_config_path {
    ($name: expr) => {
        $crate::config_dir!().join($name).with_extension("json")
    };
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DefaultDataTraceSettings {
    pub export_as: Option<String>
}

impl DefaultDataTraceSettings {
    pub fn load(name: &str) -> Result<Self, Error> {
        let f = File::open(data_trace_config_path!(name))?;
        serde_json::from_reader(f).map_err(|e| Error::other(e))
    }

    pub fn save(&self, name: &str) -> Result<(), Error> {
        let f = File::create(data_trace_config_path!(name))?;
        serde_json::to_writer_pretty(f, self).map_err(|e| Error::other(e))
    }

    pub fn export_as(&self) -> Option<ImageFormat> {
        self.export_as.as_ref().map(|x| {
            if let Some(format) = ImageFormat::from_str(x.as_str()) {
                format
            } else {
                log!(TraceLogLevel::LOG_ERROR, "Invalid data trace export format '{}'", x);
                log!(TraceLogLevel::LOG_WARNING, "Defaulting to TGA");
                ImageFormat::Tga
            }
        })
    }
}

pub enum DataTraceOutput {
    None,
    File(File),
    Buffer(Vec<u8>)
}

impl DataTraceOutput {
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, DataTraceOutput::None)
    }
}

pub struct BaseDataTrace {
    pub base: LineVisual,

    pub old_array:    Vec<Value>,
    pub main_texture: Option<RenderTexture2D>,
    swap_texture:     Option<RenderTexture2D>,

    pub old_aux:      Vec<Value>,
    pub aux_texture:  Option<RenderTexture2D>,
    aux_swap_texture: Option<RenderTexture2D>,

    pub export_as: Option<ImageFormat>,
    output_height: i32,
    output: DataTraceOutput,

    frame_counter: u64,
}

#[macro_export]
macro_rules! data_trace_common_config_ui {
    ($ui: expr, $should_export: expr, $current_format: expr, $all_formats: expr) => {
        {
            $ui.checkbox("Export data trace", $should_export);
            if $ui.is_item_hovered() {
                $ui.tooltip(|| {
                    $ui.text(concat!(
                        "Exports the entire data trace as an image file.\n\n",
                        "Note that this feature can lead to high RAM usage, unless\n",
                        "the TGA format is used"
                    ));
                });
            }

            $ui.disabled(!*$should_export, || {
                $ui.combo_simple_string(
                    "Export format", 
                    $current_format, 
                    $all_formats
                );
            });
        }
    };
}

#[macro_export]
macro_rules! data_trace_default_save_row {
    ($base: expr, $running: ident, $ui: expr, $should_export: expr, $current_format: expr, $all_formats: expr, $name: expr) => {
        {
            use $crate::gui::Gui;
            use $crate::utils::gfx::ImageFormat;
            use raylib::ffi::TraceLogLevel;

            $ui.set_cursor_pos([$ui.cursor_pos()[0], $ui.window_content_region_max()[1] - Gui::BACK_BUTTON_Y_SIZE]);
            if $ui.button("Back") {
                $running = false;
            }

            $ui.same_line();
            $ui.set_cursor_pos([
                $ui.window_content_region_max()[0] - Gui::SAVE_BUTTON_X_SIZE, 
                $ui.window_content_region_max()[1] - Gui::SAVE_BUTTON_Y_SIZE
            ]);
            
            if $ui.button_with_size("Save", [Gui::SAVE_BUTTON_X_SIZE, Gui::SAVE_BUTTON_Y_SIZE]) {
                let export_as;
                if $should_export {
                    export_as = Some($all_formats[$current_format].to_string());
                    $base.export_as = Some(ImageFormat::from_str($all_formats[$current_format]).unwrap());
                } else {
                    export_as = None;
                    $base.export_as = None;
                }

                log!(TraceLogLevel::LOG_INFO, "Saving configuration");
                
                let settings = DefaultDataTraceSettings { export_as };
                if let Err(e) = settings.save($name) {
                    log!(TraceLogLevel::LOG_ERROR, "Could not save configuration");
                    log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
                }

                $running = false;
            }
        }
    };
}

impl BaseDataTrace {
    pub const HIGHLIGHT_HEIGHT: i32 = 4;

    pub fn new() -> Self {
        BaseDataTrace { 
            base: LineVisual::new(), 
            old_array: Vec::new(), 
            main_texture: None, 
            swap_texture: None, 
            old_aux: Vec::new(), 
            aux_texture: None,
            aux_swap_texture: None, 
            export_as: None,
            output_height: 0,
            output: DataTraceOutput::None,
            frame_counter: 0
        }
    }

    pub fn init(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let width  = rl.get_screen_width() as u32;
        let height = rl.get_screen_height() as u32;
        
        self.main_texture = Some(
            rl.load_render_texture(thread, width, height)
                .expect("Could not load render texture")
        );

        self.swap_texture = Some(
            rl.load_render_texture(thread, width, height)
                .expect("Could not load render texture")
        );

        self.aux_texture = Some(
            rl.load_render_texture(thread, width, height / 4)
                .expect("Could not load render texture")
        );

        self.aux_swap_texture = Some(
            rl.load_render_texture(thread, width, height / 4)
                .expect("Could not load render texture")
        );
    }

    pub fn default_config_init(&mut self, config_name: &str, gui: &mut Gui, rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<(), ExecutionInterrupt> {
        if data_trace_config_path!(config_name).exists() {
            match DefaultDataTraceSettings::load(config_name) {
                Ok(settings) => {
                    self.export_as = settings.export_as();
                }
                Err(e) => {
                    gui.build_fn = Gui::popup;
                    gui.popup.set(
                        "Data Trace visual style",
                        format!("Unable to load data trace configuration:\n{}", e).as_str()
                    ).unwrap();
                    gui.run(rl, thread)?;
                }
            }
        }

        Ok(())
    }

    pub fn default_config(&mut self, config_name: &str, gui: &mut Gui, rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<(), ExecutionInterrupt> {
        let resolution_x = rl.get_screen_width() as f32;
        let resolution_y = rl.get_screen_height() as f32;

        let size = [
            resolution_x / DEFAULT_CONFIG_WINDOW_DIV_X,
            resolution_y / DEFAULT_CONFIG_WINDOW_DIV_Y,
        ];

        let mut should_export = self.export_as.is_some();
        let all_formats: Vec<&str> = ImageFormat::all().into_iter().map(|x| x.as_str()).collect();
        let mut current_format = {
            if let Some(export_as) = self.export_as {
                all_formats.binary_search(&export_as.as_str()).unwrap()
            } else {
                all_formats.binary_search(&"tga").unwrap()
            }
        };

        gui.begin(rl);

        let mut running = true;
        while running {
            gui.update(rl)?;

            let ui = gui.imgui.new_frame();
            ui.window("Data Trace configuration")
                .size(size, imgui::Condition::Appearing)
                .position(
                    [
                        resolution_x / 2.0 - size[0] / 2.0, 
                        resolution_y / 2.0 - size[1] / 2.0
                    ], 
                    imgui::Condition::Appearing
                )
                .bg_alpha(Gui::ALPHA)
                .build(|| {
                    data_trace_common_config_ui!(ui, &mut should_export, &mut current_format, &all_formats);
                    data_trace_default_save_row!(self, running, ui, should_export, current_format, all_formats, config_name);
                });

            gui.render(rl, thread);
        }

        gui.end(rl, thread);
        Ok(())
    }

    fn write(file: &mut File, data: &[u8]) {
        if let Err(e) = file.write(data) {
            log!(TraceLogLevel::LOG_ERROR, "Could not write to data trace export file");
            log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
        }
    }

    fn load_texture_to_ram(texture: &mut Option<RenderTexture2D>, rl: &mut RaylibHandle) -> Image {
        rl.set_trace_log(TraceLogLevel::LOG_NONE); // disables logs from loading image from texture and unloading

        let raw_texture = texture.take().unwrap().to_raw();
        let image = unsafe { Image::from_raw(raylib::ffi::LoadImageFromTexture(raw_texture.texture)) };
        *texture = Some(unsafe { RenderTexture2D::from_raw(raw_texture) });

        rl.set_trace_log(LOG_LEVEL);

        image
    }

    pub fn finalize(&mut self, shared: &mut Shared, rl: &mut RaylibHandle) {
        let output = self.output.take();
        if matches!(output, DataTraceOutput::None) {
            return;
        }

        let remaining = self.output_height % self.main_texture.as_ref().unwrap().height();
        let mut chunk = None;
        if remaining != 0 {
            let mut tmp = Self::load_texture_to_ram(&mut self.main_texture, rl);
            tmp.crop(Rectangle { 
                x: 0.0, 
                y: (tmp.height - remaining) as f32,
                width: tmp.width as f32,
                height: remaining as f32
            });
            chunk = Some(tmp);
        }

        match output {
            DataTraceOutput::File(mut file) => {
                if let Some(chunk) = chunk {
                    Self::write(&mut file, get_image_bytes(&chunk));
                }

                const HEIGHT_BYTES_OFFSET: u64 = 14;
                if let Err(e) = file.seek(SeekFrom::Start(HEIGHT_BYTES_OFFSET)) {
                    log!(TraceLogLevel::LOG_ERROR, "Could not finish writing data trace export file");
                    log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
                } else {
                    Self::write(&mut file, &(self.output_height as u16).to_le_bytes());
                }
            }
            DataTraceOutput::Buffer(mut items) => {
                if let Some(chunk) = chunk {
                    items.extend_from_slice(get_image_bytes(&chunk));
                }

                let image = raylib::ffi::Image {
                    data: items.as_ptr() as *mut c_void,
                    width: self.main_texture.as_ref().unwrap().width(),
                    height: self.output_height,
                    mipmaps: 1,
                    format: PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32
                };

                let filename = format!("data-trace-{}.{}\0", shared.get_unique_id(), self.export_as.unwrap().as_str());
                unsafe { raylib::ffi::ExportImage(image, filename.as_ptr() as *const c_char) };
            }
            DataTraceOutput::None => unreachable!()
        }

        self.output_height = 0;
    }

    pub fn prepare(&mut self, shared: &mut Shared, rl: &mut RaylibHandle) {
        self.base.prepare(shared, rl);

        if self.old_array.capacity() < shared.array.len() {
            self.old_array.reserve(shared.array.len() - self.old_array.capacity());
        }

        if let Some(format) = self.export_as {
            self.finalize(shared, rl);

            if matches!(format, ImageFormat::Tga) {
                // use custom tga encoder for lower ram usage
                match File::create(program_dir!().join(format!("data-trace-{}.tga", shared.get_unique_id()))) {
                    Ok(mut f) => {
                        // tga header
                        Self::write(&mut f, &[
                            0, // image id field length
                            0, // color map type (0: no color map)
                            2, // image type (2: uncompressed true-color image)
                            0, 0, 0, 0, 0 // color map specification (no color map)
                        ]);
                        
                        // image specification
                        Self::write(&mut f, &0u16.to_le_bytes()); // x-origin
                        Self::write(&mut f, &0u16.to_le_bytes()); // y-origin
                        Self::write(&mut f, &(self.main_texture.as_ref().unwrap().width() as u16).to_le_bytes()); // image width
                        Self::write(&mut f, &0u16.to_le_bytes()); // image height (computed later)
                        Self::write(&mut f, &32u8.to_le_bytes()); // pixel depth (bits per pixel)

                        // image descriptor
                        Self::write(&mut f, &((
                            8 << 0 | // alpha bits (targa32)
                            1 << 5 | // origin: upper left corner
                            0 << 6   // interleaving: non-interleaved
                        ) as u8).to_le_bytes());

                        self.output = DataTraceOutput::File(f);
                    }
                    Err(e) => {
                        log!(TraceLogLevel::LOG_ERROR, "Could not open data trace export file");
                        log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
                    }
                }
            } else {
                self.output = DataTraceOutput::Buffer(Vec::new());
            }
        }
    }

    pub fn on_aux_on(&mut self, shared: &Shared, rl: &RaylibHandle) {
        self.base.on_aux_on(shared, rl);

        if self.old_aux.capacity() < shared.aux.len() {
            self.old_aux.reserve(shared.aux.len() - self.old_aux.capacity());
        }
    }

    #[inline]
    pub fn on_aux_off(&mut self, shared: &Shared, rl: &RaylibHandle) {
        self.base.on_aux_off(shared, rl);
    }

    fn draw_and_update_array(
        texture: &mut RenderTexture2D, swap: &mut RenderTexture2D, 
        old: &mut Vec<Value>, new: &[Value], 
        rl: &mut RaylibHandle, thread: &RaylibThread
    ) {
        old.clear();
        old.extend(new.iter().cloned());

        {                    
            let mut draw = rl.begin_texture_mode(thread, swap);
            draw.draw_texture_rec(
                &texture,
                Rectangle::new(
                    0.0, 0.0, 
                    texture.width() as f32, 
                    texture.height() as f32
                ),
                Vector2::new(0.0, -1.0),
                Color::WHITE
            );
        }

        {
            let mut draw = rl.begin_texture_mode(thread, texture);
            draw.draw_texture_rec(
                &swap,
                Rectangle::new(
                    0.0, 0.0, 
                    swap.width() as f32, 
                    swap.height() as f32
                ),
                Vector2::new(0.0, 0.0),
                Color::WHITE
            );
        }
    }

    fn update(
        texture: &mut RenderTexture2D, swap: &mut RenderTexture2D, 
        old: &mut Vec<Value>, new: &Vec<Value>,
        rl: &mut RaylibHandle, thread: &RaylibThread
    ) -> bool {
        if old.len() != new.len() {
            Self::draw_and_update_array(texture, swap, old, new, rl, thread);
            return true;
        }

        for i in 0 .. new.len() {
            if old[i] != new[i] {
                Self::draw_and_update_array(texture, swap, old, new, rl, thread);
                return true;
            }
        }

        false
    }

    pub fn should_update(&mut self, fps: u32) -> bool {
        if self.frame_counter >= (fps / REFERENCE_FRAMERATE) as u64 {
            self.frame_counter = 0;
            return true;
        }

        self.frame_counter += 1;
        false
    }

    pub fn update_main(&mut self, shared: &Shared, rl: &mut RaylibHandle, thread: &RaylibThread) -> bool {
        if Self::update(
            self.main_texture.as_mut().unwrap(), self.swap_texture.as_mut().unwrap(),
            &mut self.old_array, &shared.array, rl, thread
        ) {
            if !matches!(self.output, DataTraceOutput::None) {
                self.output_height += 1;

                if self.output_height % self.main_texture.as_ref().unwrap().height() == 0 {
                    let chunk = Self::load_texture_to_ram(&mut self.main_texture, rl);
                    let bytes = get_image_bytes(&chunk);

                    match &mut self.output {
                        DataTraceOutput::File(file) => Self::write(file, bytes),
                        DataTraceOutput::Buffer(items) => items.extend_from_slice(bytes),
                        DataTraceOutput::None => unreachable!(),
                    }
                }
            }

            true
        } else {
            false
        }
    } 

    pub fn update_aux(&mut self, shared: &Shared, rl: &mut RaylibHandle, thread: &RaylibThread) -> bool {
        Self::update(
            self.aux_texture.as_mut().unwrap(), self.aux_swap_texture.as_mut().unwrap(),
            &mut self.old_aux, &shared.aux, rl, thread
        )
    } 
}
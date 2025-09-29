use std::{fs::File, io::{Seek, SeekFrom, Write}, iter::repeat};

use raylib::{color::Color, ffi::TraceLogLevel, math::{Rectangle, Vector2}, prelude::{RaylibDraw, RaylibTextureModeExt}, texture::{RaylibTexture2D, RenderTexture2D}, RaylibHandle, RaylibThread};

use crate::{log, program_dir, value::Value, Shared, LOG_LEVEL, REFERENCE_FRAMERATE};

use super::line_visual::LineVisual;

pub struct BaseDataTrace {
    pub base: LineVisual,

    pub old_array:    Vec<Value>,
    pub main_texture: Option<RenderTexture2D>,
    swap_texture:     Option<RenderTexture2D>,

    pub old_aux:      Vec<Value>,
    pub aux_texture:  Option<RenderTexture2D>,
    aux_swap_texture: Option<RenderTexture2D>,

    pub should_export: bool,
    export_buf: Option<Vec<u8>>,
    output_height: i32,
    output: Option<File>,

    frame_counter: u64,
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
            should_export: true,
            export_buf: None,
            output_height: 0,
            output: None,
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

        if self.should_export {
            let needed_cap = (width * height * 3) as usize;
            if let Some(buf) = &mut self.export_buf {
                if needed_cap > buf.capacity() {
                    buf.reserve(needed_cap - buf.capacity());
                }
            } else {
                self.export_buf = Some(Vec::with_capacity(needed_cap))
            }
        }
    }

    fn write(file: &mut File, data: &[u8]) {
        if let Err(e) = file.write(data) {
            log!(TraceLogLevel::LOG_ERROR, "Could not write to data trace export file");
            log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
        }
    }

    fn load_texture_to_ram(texture: &mut Option<RenderTexture2D>, rl: &mut RaylibHandle) -> raylib::ffi::Image {
        rl.set_trace_log(TraceLogLevel::LOG_NONE); // disables logs from loading image from texture and unloading

        let raw_texture = texture.take().unwrap().to_raw();
        let image = unsafe { raylib::ffi::LoadImageFromTexture(raw_texture.texture) };
        *texture = Some(unsafe { RenderTexture2D::from_raw(raw_texture) });

        rl.set_trace_log(LOG_LEVEL);

        image
    }
    
    fn write_image_chunk(buf: &mut Vec<u8>, file: &mut File, image: raylib::ffi::Image) {
        let image_data = unsafe {
            std::slice::from_raw_parts(image.data as *const u8, (image.width * image.height * 4) as usize)
        };

        let width = image.width as usize;
        let height = image.height as usize;

        buf.clear();

        for y in (0 .. height).rev() {
            for x in 0 .. width as usize  {
                let idx = (y * width + x) * 4; // 4 bytes for RGBA
                buf.extend_from_slice(&image_data[idx..idx + 3]); // 3 bytes for RGB
            }

            let padding = width * 3 % 4; // rows must be 4 bytes aligned
            if padding != 0 {
                buf.extend(repeat(0).take(padding));
            }
        }

        Self::write(file, buf);
    }

    pub fn prepare(&mut self, shared: &Shared, rl: &mut RaylibHandle) {
        self.base.prepare(shared, rl);

        if self.old_array.capacity() < shared.array.len() {
            self.old_array.reserve(shared.array.len() - self.old_array.capacity());
        }

        if self.should_export {
            const HEIGHT_BYTES_OFFSET: u64 = 0x16;
            const SIZE_BYTES_OFFSET: u64 = 0x02;
            const DIB_HEADER_SIZE: u32 = 40;
            const HEADER_SIZE: i32 = DIB_HEADER_SIZE as i32 + 14;

            if let Some(mut f) = self.output.take() {
                let remaining = self.output_height % self.main_texture.as_ref().unwrap().height();
                if remaining != 0 {
                    let mut chunk = Self::load_texture_to_ram(&mut self.main_texture, rl);

                    unsafe { 
                        raylib::ffi::ImageCrop(
                            &mut chunk, 
                            raylib::ffi::Rectangle { 
                                x: 0.0, 
                                y: (chunk.height - remaining) as f32,
                                width: chunk.width as f32,
                                height: remaining as f32
                            }
                        );
                    }

                    Self::write_image_chunk(self.export_buf.as_mut().unwrap(), &mut f, chunk);
                }

                if let Err(e) = f.seek(SeekFrom::Start(SIZE_BYTES_OFFSET)) {
                    log!(TraceLogLevel::LOG_ERROR, "Could not finish writing data trace export file");
                    log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
                } else {
                    Self::write(&mut f, &(self.main_texture.as_ref().unwrap().width() * self.output_height * 3 + HEADER_SIZE).to_le_bytes());
                }

                if let Err(e) = f.seek(SeekFrom::Start(HEIGHT_BYTES_OFFSET)) {
                    log!(TraceLogLevel::LOG_ERROR, "Could not finish writing data trace export file");
                    log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
                } else {
                    Self::write(&mut f, &self.output_height.to_le_bytes());
                }

                self.output_height = 0;
            }

            match File::create(program_dir!().join(format!("data-trace-{}.bmp", self.frame_counter))) {
                Ok(mut f) => {
                    // bitmap header
                    Self::write(&mut f, b"BM");
                    Self::write(&mut f, &0u32.to_le_bytes()); // size of file (computed later)
                    Self::write(&mut f, &0u32.to_le_bytes()); // unused
                    Self::write(&mut f, &(HEADER_SIZE as u32).to_le_bytes()); // data offset
                    // dib header
                    Self::write(&mut f, &DIB_HEADER_SIZE.to_le_bytes()); // size of header
                    Self::write(&mut f, &self.main_texture.as_ref().unwrap().width().to_le_bytes()); // width
                    Self::write(&mut f, &0i32.to_le_bytes()); // height (computed later)
                    Self::write(&mut f, &1u16.to_le_bytes()); // color planes
                    Self::write(&mut f, &24u16.to_le_bytes()); // bits per pixel
                    Self::write(&mut f, &0u32.to_le_bytes()); // compression method (none, BI_RGB)
                    Self::write(&mut f, &0u32.to_le_bytes()); // image size (can be 0 for BI_RGB)
                    Self::write(&mut f, &2835i32.to_le_bytes()); // pixel per meter horizontal
                    Self::write(&mut f, &2835i32.to_le_bytes()); // pixel per meter vertical
                    Self::write(&mut f, &0u32.to_le_bytes()); // number of colors (can be 0 for 2^n)
                    Self::write(&mut f, &0u32.to_le_bytes()); // number of important colors (whatever that means, usually ignored)
                    self.output = Some(f);
                }
                Err(e) => {
                    log!(TraceLogLevel::LOG_ERROR, "Could not open data trace export file");
                    log!(TraceLogLevel::LOG_ERROR, "    > {}", e.to_string());
                }
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
            if let Some(output) = &mut self.output {
                self.output_height += 1;

                if self.output_height % self.main_texture.as_ref().unwrap().height() == 0 {
                    let chunk = Self::load_texture_to_ram(&mut self.main_texture, rl);
                    Self::write_image_chunk(self.export_buf.as_mut().unwrap(), output, chunk);
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
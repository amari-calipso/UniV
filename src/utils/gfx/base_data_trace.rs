use std::cell::OnceCell;

use raylib::{color::Color, math::{Rectangle, Vector2}, prelude::{RaylibDraw, RaylibTextureModeExt}, texture::{RaylibTexture2D, RenderTexture2D}, RaylibHandle, RaylibThread};

use crate::{get_expect_mut, value::Value, Shared, REFERENCE_FRAMERATE};

use super::line_visual::LineVisual;

pub struct BaseDataTrace {
    pub base: LineVisual,

    pub old_array:    Vec<Value>,
    pub main_texture: OnceCell<RenderTexture2D>,
    swap_texture:     OnceCell<RenderTexture2D>,

    pub old_aux:      Vec<Value>,
    pub aux_texture:  OnceCell<RenderTexture2D>,
    aux_swap_texture: OnceCell<RenderTexture2D>,

    frame_counter: u64,
}

impl BaseDataTrace {
    pub const HIGHLIGHT_HEIGHT: i32 = 4;

    pub fn new() -> Self {
        BaseDataTrace { 
            base: LineVisual::new(), 
            old_array: Vec::new(), 
            main_texture: OnceCell::new(), 
            swap_texture: OnceCell::new(), 
            old_aux: Vec::new(), 
            aux_texture: OnceCell::new(),
            aux_swap_texture: OnceCell::new(), 
            frame_counter: 0
        }
    }

    pub fn init(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        let width  = rl.get_screen_width() as u32;
        let height = rl.get_screen_height() as u32;
        
        self.main_texture.take();
        self.swap_texture.take();
        self.aux_texture.take();
        self.aux_swap_texture.take();

        self.main_texture.set(
            rl.load_render_texture(thread, width, height)
                .expect("Could not load render texture")
        ).unwrap();

        self.swap_texture.set(
            rl.load_render_texture(thread, width, height)
                .expect("Could not load render texture")
        ).unwrap();

        self.aux_texture.set(
            rl.load_render_texture(thread, width, height / 4)
                .expect("Could not load render texture")
        ).unwrap();

        self.aux_swap_texture.set(
            rl.load_render_texture(thread, width, height / 4)
                .expect("Could not load render texture")
        ).unwrap();
    }

    pub fn prepare(&mut self, shared: &Shared, rl: &RaylibHandle) {
        self.base.prepare(shared, rl);

        if self.old_array.capacity() < shared.array.len() {
            self.old_array.reserve(shared.array.len() - self.old_array.capacity());
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
        Self::update(
            get_expect_mut!(self.main_texture), get_expect_mut!(self.swap_texture),
            &mut self.old_array, &shared.array, rl, thread
        )
    } 

    pub fn update_aux(&mut self, shared: &Shared, rl: &mut RaylibHandle, thread: &RaylibThread) -> bool {
        Self::update(
            get_expect_mut!(self.aux_texture), get_expect_mut!(self.aux_swap_texture),
            &mut self.old_aux, &shared.aux, rl, thread
        )
    } 
}
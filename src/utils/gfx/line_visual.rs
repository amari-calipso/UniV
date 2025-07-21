use std::{cell::OnceCell, cmp::max};

use raylib::{color::Color, ease::{self, EaseFn, Tween}, RaylibHandle};

use crate::{IdentityHashMap, Shared};

pub struct LineVisual {
    pub resolution_x: i32,
    pub resolution_y: i32,

    pub aux_resolution_y: i32,
    pub top: i32,

    pub line_length_mlt: f64,
    pub line_width: f64,
    pub rounded_line_width: usize,

    pub aux_line_length_mlt: f64,
    pub aux_line_width: f64,
    pub aux_rounded_line_width: usize,
}

impl LineVisual {
    pub const AUX_LINE_WIDTH: f32 = 2.0;
    pub const AUX_LINE_COLOR: Color = Color::BLUE;

    pub fn new() -> Self {
        LineVisual {
            resolution_x: 0,
            resolution_y: 0,

            aux_resolution_y: 0,
            top: 0,

            line_length_mlt: 1.0,
            line_width: 1.0,
            rounded_line_width: 1,

            aux_line_length_mlt: 1.0,
            aux_line_width: 1.0,
            aux_rounded_line_width: 1,
        }
    }
    
    pub fn prepare(&mut self, shared: &Shared, rl: &RaylibHandle) {
        self.resolution_x = rl.get_screen_width();
        self.resolution_y = rl.get_screen_height();
        self.line_length_mlt = self.resolution_y as f64 / (shared.array_max + 1) as f64;
        self.line_width      = self.resolution_x as f64 / shared.array.len() as f64;
        self.rounded_line_width = max(1, self.line_width.round() as usize);
        self.top = 0;
    }

    pub fn on_aux_on(&mut self, shared: &Shared, rl: &RaylibHandle) {
        self.resolution_y = rl.get_screen_height();
        self.top = self.resolution_y / 4;
        let main_y_size = self.resolution_y - self.top;

        self.line_length_mlt = main_y_size as f64 / (shared.array_max + 1) as f64;

        self.aux_resolution_y    = self.top;
        self.aux_line_length_mlt = self.aux_resolution_y as f64 / (shared.aux_max + 1) as f64;
        self.aux_line_width      = self.resolution_x as f64 / shared.aux.len() as f64;
        self.aux_rounded_line_width = max(1, self.aux_line_width.round() as usize);
    }

    pub fn on_aux_off(&mut self, shared: &Shared, rl: &RaylibHandle) {
        self.resolution_y    = rl.get_screen_height();
        self.line_length_mlt = self.resolution_y as f64 / (shared.array_max + 1) as f64;
        self.top = 0;
    }

    pub fn get_highlight_color(last_idx: usize, curr_idx: usize, indices: &IdentityHashMap<usize, Color>) -> Option<Color> {
        debug_assert!(last_idx <= curr_idx);

        if curr_idx - last_idx + 1 < indices.len() {
            for idx in last_idx ..= curr_idx {
                if let Some(&color) = indices.get(&idx) {
                    return Some(color);
                }
            }
        } else {
            for (&idx, &color) in indices {
                if (idx == 0 && curr_idx == 0) || (last_idx < idx && idx <= curr_idx) {
                    return Some(color);
                }
            }
        }

        None
    }
}

pub struct AnimatedLineVisual {
    pub base: LineVisual,
    
    anim_start_time: f32,

    line_length_mlt_tween:     OnceCell<Tween>,
    aux_line_length_mlt_tween: OnceCell<Tween>,
    aux_resolution_y_tween:    OnceCell<Tween>,
    top_tween:                 OnceCell<Tween>,
}

macro_rules! line_visual_update_one_tween {
    ($slf: ident, $new_duration: ident, $tween_name: ident, $target_var: ident, $type: ty) => {
        {
            if let Some(tween) = $slf.$tween_name.get_mut() {
                $slf.base.$target_var = tween.apply($new_duration - $slf.anim_start_time) as $type;
                
                if tween.has_completed() {
                    $slf.$tween_name.take();
                }
            }
        }
    };
}

macro_rules! line_visual_is_tween_animating {
    ($slf: ident, $tween_name: ident) => {
        {
            if let Some(tween) = $slf.$tween_name.get() {
                if !tween.has_completed() {
                    return true;
                }
            }
        }
    };
}

impl AnimatedLineVisual {
    const EASER: EaseFn = ease::linear_none; // TODO
    const DURATION: f32 = 1.0; // TODO

    pub fn new() -> Self {
        Self {
            base: LineVisual::new(),
            anim_start_time: 0.0,
            line_length_mlt_tween: OnceCell::new(),
            aux_line_length_mlt_tween: OnceCell::new(),
            aux_resolution_y_tween: OnceCell::new(),
            top_tween: OnceCell::new(),
        }
    }

    pub fn prepare(&mut self, shared: &Shared, rl: &RaylibHandle) {
        self.base.prepare(shared, rl);
        self.line_length_mlt_tween.take();
        self.aux_line_length_mlt_tween.take();
        self.aux_resolution_y_tween.take();
        self.top_tween.take();
    }

    fn common_tween_update(&mut self, shared: &Shared, old_line_length_mlt: f32, old_top: f32) {
        self.anim_start_time = shared.visual_duration.as_secs_f32();

        let _ = self.line_length_mlt_tween.set(Tween::new(
            Self::EASER, 
            old_line_length_mlt, self.base.line_length_mlt as f32,
            Self::DURATION
        ));

        let _ = self.top_tween.set(Tween::new(
            Self::EASER, 
            old_top, self.base.top as f32,
            Self::DURATION
        ));
    }

    pub fn on_aux_on(&mut self, shared: &Shared, rl: &RaylibHandle) {
        let old_line_length_mlt = self.base.line_length_mlt as f32;
        let old_top = self.base.top as f32;

        self.base.on_aux_on(shared, rl);

        self.common_tween_update(shared, old_line_length_mlt, old_top);

        let _ = self.aux_line_length_mlt_tween.set(Tween::new(
            Self::EASER, 
            0.0, self.base.aux_line_length_mlt as f32,
            Self::DURATION
        ));

        let _ = self.aux_resolution_y_tween.set(Tween::new(
            Self::EASER, 
            0.0, self.base.aux_resolution_y as f32,
            Self::DURATION
        ));
    }

    pub fn on_aux_off(&mut self, shared: &Shared, rl: &RaylibHandle) {
        let old_line_length_mlt = self.base.line_length_mlt as f32;
        let old_aux_line_length_mlt = self.base.aux_line_length_mlt as f32;
        let old_aux_resolution_y = self.base.aux_resolution_y as f32;
        let old_top = self.base.top as f32;
        
        self.base.on_aux_off(shared, rl);

        self.common_tween_update(shared, old_line_length_mlt, old_top);

        let _ = self.aux_line_length_mlt_tween.set(Tween::new(
            Self::EASER, 
            old_aux_line_length_mlt, 0.0,
            Self::DURATION
        ));

        let _ = self.aux_resolution_y_tween.set(Tween::new(
            Self::EASER, 
            old_aux_resolution_y, 0.0,
            Self::DURATION
        ));
    }

    pub fn update(&mut self, shared: &Shared) {
        let new_duration = shared.visual_duration.as_secs_f32();
        line_visual_update_one_tween!(self, new_duration, line_length_mlt_tween, line_length_mlt, f64);
        line_visual_update_one_tween!(self, new_duration, aux_line_length_mlt_tween, aux_line_length_mlt, f64);
        line_visual_update_one_tween!(self, new_duration, aux_resolution_y_tween, aux_resolution_y, i32);
        line_visual_update_one_tween!(self, new_duration, top_tween, top, i32);
        self.anim_start_time = new_duration;
    }

    pub fn is_animating(&self) -> bool {
        line_visual_is_tween_animating!(self, line_length_mlt_tween);
        line_visual_is_tween_animating!(self, aux_line_length_mlt_tween);
        line_visual_is_tween_animating!(self, aux_resolution_y_tween);
        line_visual_is_tween_animating!(self, top_tween);
        false
    }
}

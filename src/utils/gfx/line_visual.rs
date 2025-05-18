use std::cmp::max;

use raylib::{color::Color, RaylibHandle};

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

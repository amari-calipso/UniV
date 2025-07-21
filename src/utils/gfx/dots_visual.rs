use std::cmp::max;

use raylib::{color::Color, prelude::RaylibDraw, RaylibHandle};

use crate::Shared;

use super::line_visual::LineVisual;

pub struct DotsVisual {
    pub base: LineVisual
}

impl DotsVisual {
    pub const HIGHLIGHT_SIZE_THRESHOLD: usize = 10;
    pub const MIN_HIGHLIGHT_SIZE: usize = 4;

    pub fn new() -> Self {
        DotsVisual { 
            base: LineVisual::new() 
        }
    }

    pub fn prepare(&mut self, shared: &Shared, rl: &RaylibHandle) {
        self.base.prepare(shared, rl);
        self.base.line_length_mlt = (self.base.resolution_y as f64 - self.base.line_width) / (shared.array_max + 1) as f64;
    }

    pub fn on_aux_on(&mut self, shared: &Shared, rl: &RaylibHandle) {
        self.base.on_aux_on(shared, rl);
        let q = self.base.resolution_y / 4;
        let q_f64 = q as f64;
        self.base.line_length_mlt = (q_f64 * 3.0 - 2.0 - self.base.line_width) / (shared.array_max + 1) as f64;
        self.base.aux_resolution_y = q;
        self.base.aux_line_length_mlt = (q_f64 - self.base.aux_line_width) / (shared.aux_max + 1) as f64;
    }

    pub fn on_aux_off(&mut self, shared: &Shared, rl: &RaylibHandle) {
        self.base.on_aux_off(shared, rl);
        self.base.line_length_mlt = (self.base.resolution_y as f64 - self.base.line_width) / (shared.array_max + 1) as f64;
    }

    pub fn draw_dot(&self, mut x: usize, y: i32, width: usize, resolution_y: i32, color: Color, is_highlight: bool, draw: &mut impl RaylibDraw) {
        if is_highlight {
            let w = max(Self::MIN_HIGHLIGHT_SIZE, {
                if width < Self::HIGHLIGHT_SIZE_THRESHOLD {
                    width * 2
                } else {
                    width
                }
            });

            let hdiff = (w - width) / 2;

            if x >= hdiff {
                x -= hdiff;
            }

            draw.draw_rectangle(
                x as i32, 
                resolution_y - y - hdiff as i32 - width as i32, 
                w as i32, 
                w as i32, 
                color
            );
        } else {
            draw.draw_rectangle(
                x as i32, 
                resolution_y - y - width as i32, 
                width as i32, 
                width as i32, 
                color
            );
        }
    }
}
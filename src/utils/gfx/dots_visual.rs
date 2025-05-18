use raylib::RaylibHandle;

use crate::Shared;

use super::line_visual::LineVisual;

pub struct DotsVisual {
    pub base: LineVisual
}

impl DotsVisual {
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
}
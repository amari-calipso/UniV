use raylib::{color::Color, ffi::Vector2};

use crate::{utils::gfx::{dots_visual::DotsVisual, line_visual::LineVisual}, visual};

pub struct ScatterPlot {
    dots_visual: DotsVisual
}

visual! {
    name            = "Scatter Plot";
    highlight_color = Color::RED;

    ScatterPlot::new(self) {
        ScatterPlot { 
            dots_visual: DotsVisual::new() 
        }
    }

    prepare(shared, rl, _thread) {
        self.dots_visual.prepare(shared, rl);
    }

    on_aux_on(shared, rl, _thread) {
        self.dots_visual.on_aux_on(shared, rl);
    }
    
    on_aux_off(shared, rl, _thread) {
        self.dots_visual.on_aux_off(shared, rl);
    }

    draw(shared, draw, indices) {
        let mut last_i = 0usize;
        let mut x = 0usize;

        for i in 0 .. shared.array.len() {
            let width = (self.dots_visual.base.line_width * (i + 1) as f64) as usize - x;
            if width == 0 {
                continue;
            }

            let y = (shared.array[i].pos_value() as f64 * self.dots_visual.base.line_length_mlt) as i32;

            if let Some(color) = LineVisual::get_highlight_color(last_i, i, indices) {
                self.dots_visual.draw_dot(x, y, width, self.dots_visual.base.resolution_y, color, color == self.highlight_color(), draw);
            } else {
                self.dots_visual.draw_dot(x, y, width, self.dots_visual.base.resolution_y, Color::WHITE, false, draw);
            }

            x += width;
            last_i = i;
        }
    }

    draw_aux(shared, draw, indices) {
        draw.draw_rectangle(
            0, 0, 
            self.dots_visual.base.resolution_x, self.dots_visual.base.aux_resolution_y, 
            Color::BLACK
        );

        let mut last_i = 0usize;
        let mut x = 0usize;

        for i in 0 .. shared.aux.len() {
            let width = (self.dots_visual.base.aux_line_width * (i + 1) as f64) as usize - x;
            if width == 0 {
                continue;
            }

            let y = (shared.aux[i].pos_value() as f64 * self.dots_visual.base.aux_line_length_mlt) as i32;

            if let Some(color) = LineVisual::get_highlight_color(last_i, i, indices) {
                self.dots_visual.draw_dot(x, y, width, self.dots_visual.base.aux_resolution_y, color, color == self.highlight_color(), draw);
            } else {
                self.dots_visual.draw_dot(x, y, width, self.dots_visual.base.aux_resolution_y, Color::WHITE, false, draw);
            }

            x += width;
            last_i = i;
        }

        draw.draw_line_ex(
            Vector2 { x: 0.0, y: self.dots_visual.base.aux_resolution_y as f32 }, 
            Vector2 { x: self.dots_visual.base.resolution_x as f32, y: self.dots_visual.base.aux_resolution_y as f32 },
            LineVisual::AUX_LINE_WIDTH,
            LineVisual::AUX_LINE_COLOR
        );
    }
}
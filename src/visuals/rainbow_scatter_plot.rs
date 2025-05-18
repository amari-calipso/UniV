use raylib::{color::Color, ffi::Vector2};

use crate::{utils::gfx::{dots_visual::DotsVisual, line_visual::LineVisual}, visual};

pub struct RainbowScatterPlot {
    dots_visual: DotsVisual,
    color_const: f64,
    aux_color_const: f64,
}

visual! {
    name            = "Rainbow Scatter Plot";
    highlight_color = Color::WHITE;

    RainbowScatterPlot::new(self) {
        RainbowScatterPlot { 
            dots_visual: DotsVisual::new(),
            color_const: 1.0,
            aux_color_const: 1.0,
        }
    }

    prepare(shared, rl, _thread) {
        self.dots_visual.prepare(shared, rl);
        self.color_const = 1.0 / shared.array_max as f64;
    }

    on_aux_on(shared, rl, _thread) {
        self.dots_visual.on_aux_on(shared, rl);
        self.aux_color_const = 1.0 / shared.aux_max as f64;
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

            let value = shared.array[i].pos_value() as f64;

            let color = {
                if let Some(color) = LineVisual::get_highlight_color(last_i, i, indices) {
                    color
                } else {
                    Color::color_from_hsv((value * self.color_const) as f32 * 360.0, 1.0, 1.0)
                }
            };

            let y = (value * self.dots_visual.base.line_length_mlt) as i32;
            draw.draw_rectangle(
                x as i32, 
                self.dots_visual.base.resolution_y - y, 
                width as i32, 
                width as i32, 
                color
            );

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

            let value = shared.aux[i].pos_value() as f64;

            let color = {
                if let Some(color) = LineVisual::get_highlight_color(last_i, i, indices) {
                    color
                } else {
                    Color::color_from_hsv((value * self.aux_color_const) as f32 * 360.0, 1.0, 1.0)
                }
            };

            let y = (value * self.dots_visual.base.aux_line_length_mlt) as i32;
            draw.draw_rectangle(
                x as i32, 
                self.dots_visual.base.aux_resolution_y - y, 
                width as i32, 
                width as i32, 
                color
            );

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
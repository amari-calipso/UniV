use raylib::{color::Color, ffi::Vector2, prelude::{RaylibDraw, RaylibTextureModeExt}};

use crate::{get_expect, get_expect_mut, utils::gfx::{base_data_trace::BaseDataTrace, line_visual::LineVisual}, visual};

pub struct BwGradientDataTrace {
    base: BaseDataTrace,
    color_const: f64,
    aux_color_const: f64,
}

visual! {
    name            = "B/W Gradient Data Trace";
    highlight_color = Color::RED;

    BwGradientDataTrace::new(self) {
        BwGradientDataTrace { 
            base: BaseDataTrace::new(),
            color_const: 1.0,
            aux_color_const: 1.0,
        }
    }

    init(_shared, _gui, rl, thread) {
        self.base.init(rl, thread);
    }

    prepare(shared, rl, _thread) {
        self.base.prepare(shared, rl);
        self.color_const = 215.0 / shared.array_max as f64;
    }

    on_aux_on(shared, rl, _thread) {
        self.base.on_aux_on(shared, rl);
        self.aux_color_const = 215.0 / shared.aux_max as f64;
    }
    
    on_aux_off(shared, rl, _thread) {
        self.base.on_aux_off(shared, rl);
    }

    pre_draw(shared, rl, thread) {
        if !self.base.should_update(shared.fps) {
            return;
        }

        if self.base.update_main(&shared, rl, thread) {
            let mut draw = rl.begin_texture_mode(thread, get_expect_mut!(self.base.main_texture));

            let mut x = 0usize;
            for i in 0 .. shared.array.len() {
                let width = (self.base.base.line_width * (i + 1) as f64) as usize - x;
                if width == 0 {
                    continue;
                }

                let color_value = (40.0 + shared.array[i].pos_value() as f64 * self.color_const) as u8;
                let color = Color::new(color_value, color_value, color_value, 255);

                draw.draw_rectangle(x as i32, 0, width as i32, 1, color);
                x += width;
            }
        }

        if self.base.update_aux(&shared, rl, thread) {
            let mut draw = rl.begin_texture_mode(thread, get_expect_mut!(self.base.aux_texture));

            let mut x = 0usize;
            for i in 0 .. shared.aux.len() {
                let width = (self.base.base.aux_line_width * (i + 1) as f64) as usize - x;
                if width == 0 {
                    continue;
                }

                let color_value = (40.0 + shared.aux[i].pos_value() as f64 * self.aux_color_const) as u8;
                let color = Color::new(color_value, color_value, color_value, 255);

                draw.draw_rectangle(x as i32, 0, width as i32, 1, color);
                x += width;
            }
        }
    }

    draw(shared, draw, indices) {
        draw.draw_texture(get_expect!(self.base.main_texture), 0, 0, Color::WHITE);

        let mut last_i = 0usize;
        let mut x = 0usize;

        for i in 0 .. shared.array.len() {
            let width = (self.base.base.line_width * (i + 1) as f64) as usize - x;
            if width == 0 {
                continue;
            }

            if let Some(color) = LineVisual::get_highlight_color(last_i, i, indices) {
                draw.draw_rectangle(
                    x as i32, 
                    self.base.base.resolution_y - BaseDataTrace::HIGHLIGHT_HEIGHT, 
                    width as i32, 
                    BaseDataTrace::HIGHLIGHT_HEIGHT, 
                    color
                );
            }

            x += width;
            last_i = i;
        }
    }

    draw_aux(shared, draw, indices) {
        draw.draw_texture(get_expect!(self.base.aux_texture), 0, 0, Color::WHITE);

        let mut last_i = 0usize;
        let mut x = 0usize;

        for i in 0 .. shared.aux.len() {
            let width = (self.base.base.aux_line_width * (i + 1) as f64) as usize - x;
            if width == 0 {
                continue;
            }

            if let Some(color) = LineVisual::get_highlight_color(last_i, i, indices) {
                draw.draw_rectangle(
                    x as i32, 
                    self.base.base.aux_resolution_y - BaseDataTrace::HIGHLIGHT_HEIGHT, 
                    width as i32, 
                    BaseDataTrace::HIGHLIGHT_HEIGHT, 
                    color
                );
            }

            x += width;
            last_i = i;
        }

        draw.draw_line_ex(
            Vector2 { x: 0.0, y: self.base.base.aux_resolution_y as f32 }, 
            Vector2 { x: self.base.base.resolution_x as f32, y: self.base.base.aux_resolution_y as f32 },
            LineVisual::AUX_LINE_WIDTH,
            LineVisual::AUX_LINE_COLOR
        );
    }
}
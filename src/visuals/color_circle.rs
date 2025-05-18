use raylib::color::Color;

use crate::{utils::gfx::circle_visual::CircleVisual, visual};

pub struct ColorCircle {
    circle_visual: CircleVisual,
    color_const: f64,
    aux_color_const: f64,
}

visual! {
    name            = "Color Circle";
    highlight_color = Color::WHITE;

    ColorCircle::new(self) {
        ColorCircle { 
            circle_visual: CircleVisual::new(),
            color_const: 1.0,
            aux_color_const: 1.0,
        }
    }

    prepare(shared, rl, _thread) {
        self.circle_visual.prepare(shared, rl);
        self.color_const = 1.0 / shared.array_max as f64;
    }

    on_aux_on(shared, _rl, _thread) {
        self.circle_visual.on_aux_on(shared);
        self.aux_color_const = 1.0 / shared.aux_max as f64;
    }

    draw(shared, draw, indices) {
        for i in 0 .. shared.array.len() {
            let pos = &self.circle_visual.points[i];

            let color = {
                if let Some(color) = indices.get(&i) {
                    *color
                } else {
                    Color::color_from_hsv(
                        (shared.array[i].pos_value() as f64 * self.color_const) as f32 * 360.0, 
                        1.0, 1.0
                    )
                }
            };

            draw.draw_triangle(
                pos.end, 
                pos.start, 
                self.circle_visual.base.circle_center, 
                color
            );
        }
    }

    draw_aux(shared, draw, indices) {
        for i in 0 .. shared.aux.len() {
            let pos = &self.circle_visual.aux_points[i];

            let color = {
                if let Some(color) = indices.get(&i) {
                    *color
                } else {
                    Color::color_from_hsv(
                        (shared.aux[i].pos_value() as f64 * self.aux_color_const) as f32 * 360.0, 
                        1.0, 1.0
                    )
                }
            };

            draw.draw_triangle(
                pos.end, 
                pos.start, 
                self.circle_visual.base.aux_circle_center, 
                color
            );
        }
    }
}
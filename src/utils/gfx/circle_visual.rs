use std::ops::Range;

use raylib::{math::Vector2, RaylibHandle};

use crate::{utils::translate, Shared};

use super::base_circle_visual::BaseCircleVisual;

pub struct CircleVisual {
    pub base: BaseCircleVisual,
    pub points: Vec<Range<Vector2>>,
    pub aux_points: Vec<Range<Vector2>>,
}

impl CircleVisual {
    pub fn new() -> Self {
        CircleVisual { 
            base: BaseCircleVisual::new(),
            points: Vec::new(),
            aux_points: Vec::new()
        }
    }

    pub fn prepare(&mut self, shared: &Shared, rl: &RaylibHandle) {
        self.base.prepare(shared, rl);

        let length = shared.array.len();

        self.points.clear();
        self.points.resize(length, Default::default());

        for i in 0 .. length {
            let angle = translate(
                i as f64, 
                0.0, length as f64, 
                BaseCircleVisual::CIRCLE_START, 
                BaseCircleVisual::CIRCLE_END
            ) as f32;

            let pos = {
                Vector2::new(angle.cos(), angle.sin())
                    .scale_by(self.base.circle_radius as f32) 
                + self.base.circle_center
            };

            let pos_end = {
                let angle = angle + self.base.angle_step;
                Vector2::new(angle.cos(), angle.sin())
                    .scale_by(self.base.circle_radius as f32) 
                + self.base.circle_center
            };

            self.points[i] = pos .. pos_end;
        }
    }

    pub fn on_aux_on(&mut self, shared: &Shared) {
        self.base.on_aux_on(shared);

        let length = shared.aux.len();

        self.aux_points.clear();
        self.aux_points.resize(length, Default::default());

        for i in 0 .. length {
            let angle = translate(
                i as f64, 
                0.0, length as f64, 
                BaseCircleVisual::CIRCLE_START, 
                BaseCircleVisual::CIRCLE_END
            ) as f32;

            let pos = {
                Vector2::new(angle.cos(), angle.sin())
                    .scale_by(self.base.aux_circle_radius as f32) 
                + self.base.aux_circle_center
            };

            let pos_end = {
                let angle = angle + self.base.aux_angle_step;
                Vector2::new(angle.cos(), angle.sin())
                    .scale_by(self.base.aux_circle_radius as f32) 
                + self.base.aux_circle_center
            };

            self.aux_points[i] = pos .. pos_end;
        }
    }
}
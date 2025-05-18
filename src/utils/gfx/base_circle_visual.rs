use std::{cmp::max, f64::consts::PI};

use ordered_float::OrderedFloat;
use raylib::{math::Vector2, RaylibHandle};

use crate::Shared;

pub struct BaseCircleVisual {
    pub resolution_x: i32,
    pub resolution_y: i32,

    pub circle_center: Vector2,
    pub circle_radius: f64,
    pub angle_step:    f32,

    pub aux_circle_center: Vector2,
    pub aux_circle_radius: f64,
    pub aux_angle_step:    f32,
}

impl BaseCircleVisual {
    pub const ANGLE_TOL: f32 = 0.2f32.to_radians();
    pub const CIRCLE_START: f64 = -PI / 2.0;
    pub const CIRCLE_END: f64 = 1.5 * PI;

    pub fn new() -> Self {
        BaseCircleVisual {
            resolution_x: 0,
            resolution_y: 0,

            circle_center: Vector2 { x: 0.0, y: 0.0 },
            circle_radius: 1.0,
            angle_step: Self::ANGLE_TOL,

            aux_circle_center: Vector2 { x: 0.0, y: 0.0 },
            aux_circle_radius: 1.0,
            aux_angle_step: Self::ANGLE_TOL,
        }
    }

    pub fn prepare(&mut self, shared: &Shared, rl: &RaylibHandle) {
        self.resolution_x = rl.get_screen_width();
        self.resolution_y = rl.get_screen_height();

        if self.resolution_y < self.resolution_x {
            self.circle_radius = (self.resolution_y / 2 - 20) as f64;
            self.circle_center = Vector2 {
                x: (self.resolution_x as f64 - self.circle_radius - 20.0) as f32,
                y: (self.resolution_y / 2) as f32,
            }
        } else if self.resolution_y == self.resolution_x {
            self.circle_radius = ((self.resolution_x / 7) * 2 - 20) as f64;
            self.circle_center = Vector2 {
                x: (self.resolution_x as f64 - self.circle_radius - 20.0) as f32,
                y: (self.resolution_y / 2) as f32,
            }
        } else {
            self.circle_radius = (self.resolution_x / 2 - 20) as f64;
            self.circle_center = Vector2 {
                x: (self.resolution_x / 2) as f32,
                y: (self.resolution_y as f64 - self.circle_radius - 20.0) as f32,
            }
        }

        if shared.array.len() == 360 {
            self.angle_step = 1f32.to_radians();
        } else {
            self.angle_step = max(
                OrderedFloat(Self::ANGLE_TOL),
                OrderedFloat((360.0 / shared.array.len() as f32).to_radians())
            ).0;
        }
    }

    pub fn on_aux_on(&mut self, shared: &Shared) {
        if self.resolution_y <= self.resolution_x {
            self.aux_circle_radius = (self.resolution_y / 6 - 20) as f64;
            self.aux_circle_center = Vector2 {
                x: (self.resolution_x / 4) as f32,
                y: (self.resolution_y / 4 * 3) as f32
            };
        } else {
            self.aux_circle_radius = (self.resolution_x / 6 - 20) as f64;
            self.aux_circle_center = Vector2 {
                x: (self.resolution_y / 5 * 4) as f32,
                y: (self.resolution_y / 4) as f32
            };
        }

        if shared.aux.len() == 360 {
            self.aux_angle_step = 1f32.to_radians();
        } else {
            self.aux_angle_step = max(
                OrderedFloat(Self::ANGLE_TOL),
                OrderedFloat((360.0 / shared.aux.len() as f32).to_radians())
            ).0;
        }
    }
}
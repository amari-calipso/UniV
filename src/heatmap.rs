use std::{cmp::{max, min}, collections::HashMap};

use ordered_float::OrderedFloat;
use raylib::color::Color;

use crate::{utils::translate, IdentityHashMap};

pub const MAX_HEAT: f64   = 10_000.0;
pub const BASE_HEAT: f64  = 3_000.0;
pub const SWEEP_HEAT: f64 = MAX_HEAT;

pub const HEAT_RATE: f64   = 1.2;
pub const COOLING_MLT: f64 = 0.9925;

pub const REFERENCE_FRAMERATE: u16 = 60;

pub struct HeatMap {
    pub map: IdentityHashMap<usize, f64>
}

impl HeatMap {
    pub fn new() -> Self {
        HeatMap {
            map: HashMap::default()
        }
    }

    pub fn access(&mut self, idx: usize) {
        if !self.map.contains_key(&idx) {
            self.map.insert(idx, BASE_HEAT);
        }
    
        self.map.insert(
            idx, 
            min(
                OrderedFloat(MAX_HEAT), 
                max(
                    OrderedFloat(BASE_HEAT), 
                    OrderedFloat(*self.map.get(&idx).unwrap() * HEAT_RATE)
                )
            ).0
        );
    }

    pub fn tick(&mut self) {
        for (_, value) in self.map.iter_mut() {
            *value = max(OrderedFloat(BASE_HEAT), OrderedFloat(*value * COOLING_MLT)).0;
        }
    }

    #[allow(dead_code)]
    pub fn get_normalized_value(&mut self, min_output: f64, idx: usize) -> f64 {
        if !self.map.contains_key(&idx) {
            self.map.insert(idx, BASE_HEAT);
        }

        translate(
            *self.map.get(&idx).unwrap(), 
            BASE_HEAT, 
            MAX_HEAT, 
            min_output,
            1.0
        )
    }

    pub fn get_color(&mut self, color_map: &[Color], min_idx: usize, idx: usize) -> Color {
        if !self.map.contains_key(&idx) {
            self.map.insert(idx, BASE_HEAT);
        }
    
        color_map[
            translate(
                *self.map.get(&idx).unwrap(), 
                BASE_HEAT, 
                MAX_HEAT, 
                min_idx as f64,
                (color_map.len() - 1) as f64
            ) as usize
        ]
    }
}

pub fn should_tick<T: TryInto<u16>>(heatmap_cnt: u16, fps: T) -> bool 
    where <T as TryInto<u16>>::Error: std::fmt::Debug 
{
    heatmap_cnt >= fps.try_into().unwrap() / REFERENCE_FRAMERATE
}
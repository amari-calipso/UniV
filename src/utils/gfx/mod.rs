use raylib::{math::Vector2, prelude::RaylibDraw, text::WeakFont};
use crate::{DEFAULT_SPACING_X, DEFAULT_SPACING_Y};

pub mod line_visual;
pub mod dots_visual;
pub mod base_circle_visual;
pub mod circle_visual;
pub mod base_data_trace;

#[macro_export]
macro_rules! rgb_color {
    ($r:expr, $g:expr, $b:expr) => {
        raylib::color::Color::new($r, $g, $b, 255)
    };
}

#[macro_export]
macro_rules! draw_flipped_texture {
    ($draw: ident, $src: expr, $width: ident, $height: ident) => {
        $draw.draw_texture_pro( // draws the texture flipped upside down (coordinate system is y-flipped in texture mode)
            $src, 
            Rectangle { x: 0.0, y: 0.0, width: $width as f32, height: -($height as f32) }, 
            Rectangle { x: 0.0, y: 0.0, width: $width as f32, height: $height as f32 }, 
            Vector2 { x: 0.0, y: 0.0 }, 
            0.0, Color::WHITE
        );
    };
}

// raylib-rs technically provides this, but it doesn't work with WeakFont for some reason
fn measure_weak_font_text(font: &WeakFont, text: &str, font_size: f32, spacing: f32) -> Vector2 {
    let c_text = std::ffi::CString::new(text).expect("Invalid C string");
    unsafe { raylib::ffi::MeasureTextEx(*font.as_ref(), c_text.as_ptr(), font_size, spacing).into() }
}

pub fn draw_outline_text(
    text: &str, mut pos: Vector2, font_size: usize, font: &WeakFont,
    color: raylib::color::Color, outline_size: usize, draw: &mut impl RaylibDraw
) {
    debug_assert!(outline_size <= pos.x as usize && outline_size <= pos.y as usize);    
    
    let font_size = font_size as f32;
    let outline_size = outline_size as f32;

    let offset_same     = Vector2 { x: outline_size, y: outline_size };
    let offset_opposite = Vector2 { x: outline_size, y: -outline_size };

    use raylib::color::Color;

    for line in text.split('\n') {
        let size = {
            if line != "" {
                draw.draw_text_ex(&font, line, pos - offset_same, font_size, DEFAULT_SPACING_X, Color::BLACK);
                draw.draw_text_ex(&font, line, pos + offset_opposite, font_size, DEFAULT_SPACING_X, Color::BLACK);
                draw.draw_text_ex(&font, line, pos - offset_opposite, font_size, DEFAULT_SPACING_X, Color::BLACK);
                draw.draw_text_ex(&font, line, pos + offset_same, font_size, DEFAULT_SPACING_X, Color::BLACK);
                draw.draw_text_ex(&font, line, pos, font_size, DEFAULT_SPACING_X, color);
                measure_weak_font_text(&font, line, font_size, DEFAULT_SPACING_X)
            } else {
                measure_weak_font_text(&font, "#", font_size, DEFAULT_SPACING_X)
            }
        };

        pos.y += size.y + outline_size + DEFAULT_SPACING_Y;
    }
}

pub fn draw_outline_text_right(
    text: &str, mut pos: Vector2, font_size: usize, font: &WeakFont,
    color: raylib::color::Color, outline_size: usize, draw: &mut impl RaylibDraw
) {    
    let font_size = font_size as f32;
    let outline_size = outline_size as f32;

    let offset_same     = Vector2 { x: outline_size, y: outline_size };
    let offset_opposite = Vector2 { x: outline_size, y: -outline_size };

    use raylib::color::Color;

    for line in text.split('\n') {
        let size = {
            if line != "" {
                let size = measure_weak_font_text(&font, line, font_size, DEFAULT_SPACING_X);
                let start = Vector2 { x: size.x + outline_size * 2.0 + DEFAULT_SPACING_X, y: 0.0 };
                draw.draw_text_ex(&font, line, pos - offset_same - start, font_size, DEFAULT_SPACING_X, Color::BLACK);
                draw.draw_text_ex(&font, line, pos + offset_opposite - start, font_size, DEFAULT_SPACING_X, Color::BLACK);
                draw.draw_text_ex(&font, line, pos - offset_opposite - start, font_size, DEFAULT_SPACING_X, Color::BLACK);
                draw.draw_text_ex(&font, line, pos + offset_same - start, font_size, DEFAULT_SPACING_X, Color::BLACK);
                draw.draw_text_ex(&font, line, pos - start, font_size, DEFAULT_SPACING_X, color);
                size
            } else {
                measure_weak_font_text(&font, "#", font_size, DEFAULT_SPACING_X)
            }
        };
        
        pos.y += size.y + outline_size + DEFAULT_SPACING_Y;
    }
}
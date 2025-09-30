use enum_dispatch::enum_dispatch;
use raylib::{color::Color, prelude::RaylibDraw, RaylibHandle, RaylibThread};
use crate::{gui::Gui, univm::object::ExecutionInterrupt, IdentityHashMap, Shared};

#[enum_dispatch(AnyVisual)]
pub trait Visual {
    /// The name of the visual style
    fn name(&self) -> &str;
    /// The default highlight color
    fn highlight_color(&self) -> Color;
    /// Draws the main array
    fn draw(&mut self, shared: &mut Shared, draw: &mut impl RaylibDraw, indices: &IdentityHashMap<usize, Color>);
    /// Draws the auxiliary arrays
    fn draw_aux(&mut self, shared: &mut Shared, draw: &mut impl RaylibDraw, indices: &IdentityHashMap<usize, Color>);

    /// Optional: Is called when the program requests the visual to show a configuration screen
    fn config(&mut self, _shared: &Shared, _gui: &mut Gui, _raylib: &mut RaylibHandle, _thread: &RaylibThread) -> Result<(), ExecutionInterrupt> {
        Ok(())
    }

    /// Optional: MUST be set to true if the visual implements config
    fn is_configurable(&self) -> bool {
        false
    }

    /// Optional: Is called when the program is initialized or the screen resolution is changed.
    /// Useful for preliminary setup, such as preparing resolution-bound data, or allocating textures
    fn init(&mut self, _shared: &Shared, _gui: &mut Gui, _raylib: &mut RaylibHandle, _thread: &RaylibThread) -> Result<(), ExecutionInterrupt> {
        Ok(())
    }

    /// Optional: Is called when the program exits and resources must be cleaned up
    fn unload(&mut self, _raylib: &mut RaylibHandle, _thread: &RaylibThread) {}
    
    /// Optional: Is called when the array size changed or a different kind of full update occurred
    fn prepare(&mut self, _shared: &mut Shared, _raylib: &mut RaylibHandle, _thread: &RaylibThread) {}
    /// Optional: Is called when the auxiliary visual is enabled, by adding a new auxiliary array for example
    fn on_aux_on(&mut self, _shared: &mut Shared, _raylib: &mut RaylibHandle, _thread: &RaylibThread) {}
    /// Optional: Is called when the auxiliary visual is disabled, by removing all auxiliary arrays for example
    fn on_aux_off(&mut self, _shared: &mut Shared, _raylib: &mut RaylibHandle, _thread: &RaylibThread) {}
    /// Optional: Is called when the current algorithm finishes running and any generation process should be finalized
    fn finalize(&mut self, _shared: &mut Shared, _raylib: &mut RaylibHandle, _thread: &RaylibThread) {}

    /// Optional: Is called before starting the draw process.
    /// Useful for setup that must happen before the draw handle is created
    fn pre_draw(&mut self, _shared: &mut Shared, _raylib: &mut RaylibHandle, _thread: &RaylibThread) {}
}

#[macro_export]
macro_rules! visual {
    {
        name            = $name: literal;
        highlight_color = $highlight_color: expr;

        $visual_struct: ident :: new($slf: ident) $new: block

        $(init($init_shared: ident, $init_gui: ident, $init_raylib: ident, $init_thread: ident) $init: block)?
        $(unload($unload_raylib: ident, $unload_thread: ident) $unload: block)?
        $(config($config_shared: ident, $config_gui: ident, $config_raylib: ident, $config_thread: ident) $config: block)?

        $(prepare($prepare_shared: ident, $prepare_raylib: ident, $prepare_thread: ident) $prepare: block)?
        $(on_aux_on($aux_on_shared: ident, $aux_on_raylib: ident, $aux_on_thread: ident) $on_aux_on: block)?
        $(on_aux_off($aux_off_shared: ident, $aux_off_raylib: ident, $aux_off_thread: ident) $on_aux_off: block)?
        $(finalize($finalize_shared: ident, $finalize_raylib: ident, $finalize_thread: ident) $finalize: block)?

        $(pre_draw($pre_draw_shared: ident, $pre_draw_raylib: ident, $pre_draw_thread: ident) $pre_draw: block)?
        
        draw($draw_shared: ident, $draw_rl: ident, $indices: ident) $draw: block
        draw_aux($draw_aux_shared: ident, $draw_aux_rl: ident, $aux_indices: ident) $draw_aux: block
    } => {
        impl $visual_struct {
            pub fn new() -> Self $new
        }

        impl $crate::visual::Visual for $visual_struct {
            fn name(&self) -> &str {
                $name
            }

            fn highlight_color(&self) -> raylib::color::Color {
                $highlight_color
            }
            
            $(
                fn init(
                    &mut $slf, 
                    $init_shared: &$crate::Shared, 
                    $init_gui: &mut $crate::gui::Gui,                  
                    $init_raylib: &mut raylib::prelude::RaylibHandle,
                    $init_thread: &raylib::prelude::RaylibThread
                ) -> Result<(), $crate::univm::object::ExecutionInterrupt> {
                    $crate::__filter_execution_interrupts!($init)
                }
            )?

            $(
                fn unload(
                    &mut $slf, 
                    $unload_raylib: &mut raylib::prelude::RaylibHandle,
                    $unload_thread: &raylib::prelude::RaylibThread
                ) {
                    $unload
                }
            )?

            $(
                fn is_configurable(&$slf) -> bool { true }
                fn config(
                    &mut $slf, 
                    $config_shared: &$crate::Shared, 
                    $config_gui: &mut $crate::gui::Gui,                  
                    $config_raylib: &mut raylib::prelude::RaylibHandle,
                    $config_thread: &raylib::prelude::RaylibThread
                ) -> Result<(), $crate::univm::object::ExecutionInterrupt> {
                    $crate::__filter_execution_interrupts!($config)
                }
            )?

            $(
                fn prepare(
                    &mut $slf, 
                    $prepare_shared: &mut $crate::Shared, 
                    $prepare_raylib: &mut raylib::prelude::RaylibHandle,
                    $prepare_thread: &raylib::prelude::RaylibThread
                ) {
                    $prepare
                }
            )?

            $(
                fn on_aux_on(
                    &mut $slf, 
                    $aux_on_shared: &mut $crate::Shared, 
                    $aux_on_raylib: &mut raylib::prelude::RaylibHandle,
                    $aux_on_thread: &raylib::prelude::RaylibThread
                ) {
                    $on_aux_on
                }
            )?

            $(
                fn on_aux_off(
                    &mut $slf, 
                    $aux_off_shared: &mut $crate::Shared, 
                    $aux_off_raylib: &mut raylib::prelude::RaylibHandle,
                    $aux_off_thread: &raylib::prelude::RaylibThread
                ) {
                    $on_aux_off
                }
            )?

            $(
                fn finalize(
                    &mut $slf, 
                    $finalize_shared: &mut $crate::Shared, 
                    $finalize_raylib: &mut raylib::prelude::RaylibHandle,
                    $finalize_thread: &raylib::prelude::RaylibThread
                ) {
                    $finalize
                }
            )?

            $(
                fn pre_draw(
                    &mut $slf, 
                    $pre_draw_shared: &mut $crate::Shared, 
                    $pre_draw_raylib: &mut raylib::prelude::RaylibHandle, 
                    $pre_draw_thread: &raylib::prelude::RaylibThread
                ) {
                    $pre_draw
                }
            )?

            fn draw(
                &mut $slf, 
                $draw_shared: &mut $crate::Shared,
                $draw_rl: &mut impl raylib::prelude::RaylibDraw,
                $indices: &$crate::IdentityHashMap<usize, raylib::color::Color>
            ) $draw

            fn draw_aux(
                &mut $slf, 
                $draw_aux_shared: &mut $crate::Shared,
                $draw_aux_rl: &mut impl raylib::prelude::RaylibDraw,
                $aux_indices: &$crate::IdentityHashMap<usize, raylib::color::Color>
            ) $draw_aux
        }
    };
}
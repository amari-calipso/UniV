use enum_dispatch::enum_dispatch;
use raylib::{color::Color, prelude::RaylibDraw, RaylibHandle, RaylibThread};
use crate::{gui::Gui, univm::object::ExecutionInterrupt, IdentityHashMap, Shared};

#[enum_dispatch(AnyVisual)]
pub trait Visual {
    /// The name of the visual style
    fn name(&self) -> &str;
    /// The default highlight color
    fn highlight_color(&self) -> Color;

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

    /// Optional: Is called before starting the draw process.
    /// Useful for setup that must happen before the draw handle is created
    fn pre_draw(&mut self, _shared: &mut Shared, _raylib: &mut RaylibHandle, _thread: &RaylibThread) {}
    
    /// Draws the main array
    fn draw(&mut self, shared: &mut Shared, draw: &mut impl RaylibDraw, indices: &IdentityHashMap<usize, Color>);
    /// Draws the auxiliary arrays
    fn draw_aux(&mut self, shared: &mut Shared, draw: &mut impl RaylibDraw, indices: &IdentityHashMap<usize, Color>);
}

#[macro_export]
macro_rules! visual {
    {
        name            = $name: literal;
        highlight_color = $highlight_color: expr;

        $visual_struct: ident :: new($slf: ident) $new: block

        $(init($init_shared: ident, $gui: ident, $init_raylib: ident, $init_thread: ident) $init: block)?
        $(unload($unload_raylib: ident, $unload_thread: ident) $unload: block)?

        $(prepare($prepare_shared: ident, $prepare_raylib: ident, $prepare_thread: ident) $prepare: block)?
        $(on_aux_on($aux_on_shared: ident, $aux_on_raylib: ident, $aux_on_thread: ident) $on_aux_on: block)?
        $(on_aux_off($aux_off_shared: ident, $aux_off_raylib: ident, $aux_off_thread: ident) $on_aux_off: block)?

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
                    $gui: &mut $crate::gui::Gui,                  
                    $init_raylib: &mut raylib::prelude::RaylibHandle,
                    $init_thread: &raylib::prelude::RaylibThread
                ) -> Result<(), $crate::univm::object::ExecutionInterrupt> {
                    use $crate::univm::object::ExecutionInterrupt;
                    let _res: Result<(), ExecutionInterrupt> = {
                        $init
                        #[allow(unreachable_code)] Ok(())
                    };

                    #[allow(unreachable_code)]
                    if let Err(e) = _res {
                        match e {
                            ExecutionInterrupt::Quit => return Err(e),
                            _ => panic!("Visual styles can only return ExecutionInterrupts of type Quit")
                        }
                    }

                    Ok(())
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
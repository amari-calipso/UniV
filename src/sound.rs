use enum_dispatch::enum_dispatch;
use raylib::{RaylibHandle, RaylibThread};

use crate::{gui::Gui, univm::object::ExecutionInterrupt, Shared};

#[enum_dispatch(AnySound)]
pub trait Sound {
    /// The name of the sound engine
    fn name(&self) -> &str;
    /// Produces a wave given a value, the maximum value it can assume, and the minimum length of the sample
    fn play(&mut self, value: i64, max: i64, sample_length: usize) -> Vec<i16>;

    /// Optional: Is called when the program requests the sound engine to show a configuration screen
    fn config(&mut self, _shared: &Shared, _gui: &mut Gui, _raylib: &mut RaylibHandle, _thread: &RaylibThread) -> Result<(), ExecutionInterrupt> {
        Ok(())
    }

    /// Optional: MUST be set to true if the sound engine implements config
    fn is_configurable(&self) -> bool {
        false
    }

    /// Optional: Is called when the sound engine is to be initialized.
    /// Useful for preliminary setup, such as loading settings
    fn prepare(&mut self, _shared: &Shared, _gui: &mut Gui, _raylib: &mut RaylibHandle, _thread: &RaylibThread) -> Result<(), ExecutionInterrupt> {
        Ok(())
    }
}

#[macro_export]
macro_rules! sound {
    {
        name = $name:literal;

        $sound_struct: ident :: new($slf: ident) $new: block

        $(prepare($prepare_shared: ident, $prepare_gui: ident, $prepare_raylib: ident, $prepare_thread: ident) $prepare: block)?
        $(config($config_shared: ident, $config_gui: ident, $config_raylib: ident, $config_thread: ident) $config: block)?

        play($value: ident, $max: ident, $sample_length: ident) $play: block
    } => {
        impl $sound_struct {
            pub fn new() -> Self $new
        }

        impl $crate::sound::Sound for $sound_struct {
            fn name(&self) -> &str {
                $name
            }

            $(
                fn prepare(
                    &mut $slf, 
                    $prepare_shared: &$crate::Shared,
                    $prepare_gui: &mut $crate::gui::Gui,
                    $prepare_raylib: &mut raylib::prelude::RaylibHandle, 
                    $prepare_thread: &raylib::prelude::RaylibThread,
                ) -> Result<(), $crate::univm::object::ExecutionInterrupt> {
                    $crate::__filter_execution_interrupts!($prepare)
                }
            )?

            $(
                fn is_configurable(&$slf) -> bool { true }
                fn config(
                    &mut $slf, 
                    $config_shared: &$crate::Shared,
                    $config_gui: &mut $crate::gui::Gui,
                    $config_raylib: &mut raylib::prelude::RaylibHandle, 
                    $config_thread: &raylib::prelude::RaylibThread,
                ) -> Result<(), $crate::univm::object::ExecutionInterrupt> {
                    $crate::__filter_execution_interrupts!($config)
                }
            )?


            fn play(&mut $slf, $value: i64, $max: i64, $sample_length: usize) -> Vec<i16> $play
        }
    };
}
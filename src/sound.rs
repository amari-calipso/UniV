use enum_dispatch::enum_dispatch;
use raylib::{RaylibHandle, RaylibThread};

use crate::{gui::Gui, univm::object::ExecutionInterrupt, Shared};

#[enum_dispatch(AnySound)]
pub trait Sound {
    /// The name of the sound engine
    fn name(&self) -> &str;

    /// Optional: Is called when the sound engine is to be initialized.
    /// Useful for preliminary setup, such as loading settings
    fn prepare(&mut self, _shared: &Shared, _gui: &mut Gui, _raylib: &mut RaylibHandle, _thread: &RaylibThread) -> Result<(), ExecutionInterrupt> {
        Ok(())
    }

    /// Produces a wave given a value, the maximum value it can assume, and the minimum length of the sample
    fn play(&mut self, value: i64, max: i64, sample_length: usize) -> Vec<i16>;
}

#[macro_export]
macro_rules! sound {
    {
        name = $name:literal;

        $sound_struct: ident :: new($slf: ident) $new: block

        $(prepare($shared: ident, $gui: ident, $raylib: ident, $thread: ident) $prepare: block)?

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
                    $shared: &$crate::Shared,
                    $gui: &mut $crate::gui::Gui,
                    $raylib: &mut raylib::prelude::RaylibHandle, 
                    $thread: &raylib::prelude::RaylibThread,
                ) -> Result<(), $crate::univm::object::ExecutionInterrupt> {
                    use $crate::univm::object::ExecutionInterrupt;
                    let _res: Result<(), ExecutionInterrupt> = {
                        $prepare
                        #[allow(unreachable_code)] Ok(())
                    };

                    #[allow(unreachable_code)]
                    if let Err(e) = _res {
                        match e {
                            ExecutionInterrupt::Quit => return Err(e),
                            _ => panic!("Sound engines can only return ExecutionInterrupts of type Quit")
                        }
                    }

                    Ok(())
                }
            )?

            fn play(&mut $slf, $value: i64, $max: i64, $sample_length: usize) -> Vec<i16> $play
        }
    };
}
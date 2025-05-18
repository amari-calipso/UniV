#[macro_export]
macro_rules! api_layer {
    {
        definitions($defs_globals: ident) $defs_body: block
        $(types($types_globals: ident) $types_body: block)?
    } => {
        pub fn define($defs_globals: &mut $crate::univm::environment::Environment) {
            $defs_body
        }

        pub fn define_analyzer($crate::expand_ident!($($types_globals)?): &mut $crate::compiler::environment::AnalyzerEnvironment) {
            $($types_body)?
        }
    };
}

#[macro_export]
macro_rules! api_fn_body {
    {
        $name:ident ($args:ident, $args_types:expr, $univ:ident, $task:ident) $(-> ($return_type:expr))?
            $body:block
    } => {
        #[allow(non_snake_case)]
        pub mod $name {
            #[allow(unused)] use $crate::compiler::type_system::UniLType;
            #[allow(unused)] use $crate::univm::object::*;
            #[allow(unused)] use $crate::univm::*;
            #[allow(unused)] use $crate::utils::object::*;
            #[allow(unused)] use $crate::utils::*;
            #[allow(unused)] use $crate::*;
            #[allow(unused)] use super::*;
            #[allow(unused)] use std::rc::Rc;
            #[allow(unused)] use std::cell::RefCell;

            pub fn args() -> Vec<UniLType> {
                Vec::from($args_types)
            }

            #[allow(dead_code)]
            pub fn return_type() -> UniLType {
                $crate::expand_type!($($return_type)?)
            }

            pub fn func(
                $univ: &mut $crate::UniV,
                #[allow(unused_mut)] mut $args: Vec<UniLValue>,
                $task: &mut Task
            ) -> Result<UniLValue, ExecutionInterrupt> {
                $body
            }
        }
    };
}

#[macro_export]
macro_rules! api_fn_obj {
    ($name: ident) => {
        {
            use std::rc::Rc;
            use $crate::univm::object;

            object::UniLValue::Object(
                Rc::new(std::cell::RefCell::new(
                    object::AnyObject::AnyCallable(
                        object::NativeCallable::new(Rc::new($name::func), $name::args().len() as u8).into()
                    )
                ))
            )
        }
    };
}

#[macro_export]
macro_rules! api_fn_types {
    ($name: ident) => {
        $crate::compiler::type_system::UniLType::Callable {
            args: $name::args(),
            return_type: Box::new($name::return_type())
        }
    };
}

#[macro_export]
macro_rules! api_def_fn {
    ($globals: ident, $name: ident) => {
        $globals.define(&std::rc::Rc::from(stringify!($name)), $crate::api_fn_obj!($name))
    };
}

#[macro_export]
macro_rules! api_field_def_fn {
    ($globals: ident, $name: ident) => {
        $globals.set(&std::rc::Rc::from(stringify!($name)), $crate::api_fn_obj!($name))
    };
}

#[macro_export]
macro_rules! api_typedef_fn {
    ($globals: ident, $name: ident) => {
        $globals.define(&std::rc::Rc::from(stringify!($name)), $crate::api_fn_types!($name))
    };
}

#[macro_export]
macro_rules! api_field_typedef_fn {
    ($obj: ident, $name: ident) => {
        $obj.insert(std::rc::Rc::from(stringify!($name)), $crate::api_fn_types!($name))
    };
}

#[macro_export]
macro_rules! api_layer_fn {
    {
        $(
            $name:ident ($args:ident, $args_types:expr, $univ:ident, $task:ident) $(-> ($return_type:expr))?
                $body:expr
        )+
    } => {
        $(
            $crate::api_fn_body! {
                $name($args, $args_types, $univ, $task) $(-> ($return_type))? {
                    $body
                }
            }
        )+

        $crate::api_layer! {
            definitions(globals) {
                $(
                    $crate::api_def_fn!(globals, $name);
                )+
            }
            types(globals) {
                $(
                    $crate::api_typedef_fn!(globals, $name);
                )+
            }
        }
    };
}
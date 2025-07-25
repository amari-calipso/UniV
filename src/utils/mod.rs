use std::time::{Duration, SystemTime};

pub mod lang;
pub mod gfx;
pub mod sound;
pub mod object;
pub mod api_hook;
pub mod language_hook;
pub mod colormaps;

#[macro_export]
macro_rules! expand_ident {
    ($x: ident) => {
        $x
    };
    () => {
        _
    };
}

#[macro_export]
macro_rules! expand_type {
    ($x:expr) => {
        $x
    };
    () => {
        $crate::compiler::type_system::UniLType::Any
    };
}

#[macro_export]
macro_rules! log {
    ($level: expr, $($arg: tt)*) => {
        println!(
            "{}UniV: {}", 
            match $level {
                raylib::ffi::TraceLogLevel::LOG_DEBUG => "DEBUG: ",
                raylib::ffi::TraceLogLevel::LOG_INFO => "INFO: ",
                raylib::ffi::TraceLogLevel::LOG_WARNING => "WARNING: ",
                raylib::ffi::TraceLogLevel::LOG_ERROR => "ERROR: ",
                _ => ""
            },
            format_args!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! get_expect {
    ($obj: expr) => {
        $obj.get().expect(concat!("Could not get Once expression: ", stringify!($obj)))
    };
}

#[macro_export]
macro_rules! get_expect_mut {
    ($obj: expr) => {
        $obj.get_mut().expect(concat!("Could not get Once expression: ", stringify!($obj)))
    };
}


#[macro_export]
macro_rules! program_dir {
    () => {
        $crate::get_expect!($crate::PROGRAM_DIR)
    };
}

#[macro_export]
macro_rules! config_dir {
    () => {
        $crate::program_dir!().join("config")
    };
}

#[macro_export]
macro_rules! expect_list {
    ($x: expr) => {
        {
            if let AnyObject::List(list) = &*$x {
                list
            } else {
                panic!("Expecting List")
            }
        }
    };
}

#[macro_export]
macro_rules! expect_list_mut {
    ($x: expr) => {
        {
            if let AnyObject::List(list) = &mut *$x {
                list
            } else {
                panic!("Expecting List")
            }
        }
    };
}

#[macro_export]
macro_rules! format_traceback {
    ($traceback: ident, $value: expr, $thread: expr) => {
        format!("{}[Thread {}] Exception: {}", $traceback, $thread, $value.stringify())
    };
}

#[macro_export]
macro_rules! expect_traceback {
    ($x: expr) => {
        {
            if let ExecutionInterrupt::Exception { value, traceback, thread } = $x {
                $crate::format_traceback!(traceback, value, thread)
            } else {
                panic!("Expecting exception")
            }
        }
    };
}

pub fn substring(string: &str, a: usize, b: usize) -> String {
    string.chars().skip(a).take(b - a).collect()
}

pub fn translate(value: f64, min: f64, max: f64, min_result: f64, max_result: f64) -> f64 {
    let delta_orig = max - min;
    let delta_out  = max_result - min_result;
    let scaled = (value - min) / delta_orig;
    min_result + (scaled * delta_out)
}

pub fn now_as_secs() -> f64 {
    SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64()
}

#[macro_export]
macro_rules! with_timer {
    ($univ: ident, $expr: expr) => {
        {
            let start = std::time::Instant::now();
            let res = $expr;
            $univ.time += start.elapsed().as_secs_f64() * 1000.0;
            res
        }
    };
}

pub fn duration_to_hms(duration: &Duration) -> String {
    let mut s = duration.as_secs();
    let mut h = 0u8;
    let mut m = 0u8;
    
    while s >= 60 {
        s -= 60;
        m += 1;
    }

    while m >= 60 {
        m -= 60;
        h += 1;
    }

    format!("{:02}:{:02}:{:02}", h, m, s)
}
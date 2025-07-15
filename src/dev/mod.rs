use std::{fs::File, io::Error};

use raylib::ffi::TraceLogLevel;
use bincode::encode_into_std_write;

use crate::{log, UniV};

#[cfg(test)]
mod tests;

pub fn compile_algos() -> Result<(), Error> {
    let mut errors = Vec::new();
    let bytecode = UniV::new().compile_algos(&mut errors);

    if errors.is_empty() {
        log!(TraceLogLevel::LOG_INFO, "Serializing bytecode");

        let mut f = File::create("algos.unib")?;
        encode_into_std_write(bytecode.unwrap(), &mut f, bincode::config::standard())
            .map_err(|e| Error::other(e.to_string()))?;

        log!(TraceLogLevel::LOG_INFO, "Compilation was successful");
        Ok(())
    } else {
        let mut error_buf = String::from("Something went wrong while loading algorithms:");

        for error in errors {
            error_buf.push('\n');
            error_buf.push_str(&error.to_string());
        }

        log!(TraceLogLevel::LOG_ERROR, "{}", error_buf);
        Err(Error::other("Compilation failed"))
    }
}

pub fn load_algos() -> Result<(), Error> {
    UniV::new().load_algos().expect("Could not load algorithms");
    Ok(())
}
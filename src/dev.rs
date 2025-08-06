use std::{fs::File, io::Error};

use raylib::ffi::TraceLogLevel;

use bincode::encode_into_std_write;

use crate::{log, UniV, utils::report_errors, compiler::analyzer::Analyzer, utils};

pub fn compile_algos() -> Result<(), Error> {
    let mut errors = Vec::new();
    let bytecode = UniV::new().compile_algos(&mut errors, false);

    if errors.is_empty() {
        log!(TraceLogLevel::LOG_INFO, "Serializing bytecode");

        let mut f = File::create("algos.unib")?;
        encode_into_std_write(bytecode.unwrap(), &mut f, bincode::config::standard())
            .map_err(|e| Error::other(e.to_string()))?;

        log!(TraceLogLevel::LOG_INFO, "Compilation was successful");
        Ok(())
    } else {
        report_errors("Something went wrong while loading algorithms", &errors);
        Err(Error::other("Compilation failed"))
    }
}

pub fn generate_headers() -> Result<(), Error> {
    let mut univ = UniV::new();

    let mut errors = Vec::new();
    let ast = univ.get_algos_ast(&mut errors);

    if errors.is_empty() {
        let mut analyzer = Analyzer::new(&univ.vm.globals.borrow());
        analyzer.analyze(&ast);

        if !analyzer.errors.is_empty() {
            report_errors("Something went wrong while loading algorithms", &errors);
            return Err(Error::other("Compilation failed"));
        }

        log!(TraceLogLevel::LOG_INFO, "Generating headers");
        UniV::generate_headers(analyzer.globals.borrow().get_locals())?;
        log!(TraceLogLevel::LOG_INFO, "Generation was successful");
        Ok(())
    } else {
        report_errors("Something went wrong while loading algorithms", &errors);
        Err(Error::other("Compilation failed"))
    }
}
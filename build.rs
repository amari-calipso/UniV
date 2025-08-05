mod dev_util; // shuts up rust-analyzer

use std::{fs::{create_dir, read_dir, File}, io::{Error, Write}, path::{Path, PathBuf}};

use convert_case::{Case, Casing};
use image::{imageops::FilterType, ImageFormat, ImageReader};

const INDENT_SIZE: usize = 4;
const ICON_SIZE: u32 = 32;

macro_rules! log {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format_args!($($tokens)*))
    };
}

macro_rules! info {
    ($($tokens: tt)*) => {
        log!("INFO: {}", format_args!($($tokens)*))
    };
}

const GENERATED_MOD_RS_HEADER: &str = concat!(
    "use crate::*;\n\n",
    "#[enum_dispatch::enum_dispatch]\n",
    "pub enum "
);

fn push_indent(str: &mut String, level: usize) {
    for _ in 0 .. level * INDENT_SIZE {
        str.push(' ');
    }
}

fn generate_module(name: &str, folder: &str, type_: &str, src: &Path) -> Result<(), Error> {
    let mut mods     = String::new();
    let mut imports  = String::new();
    let mut sum_type = String::from(GENERATED_MOD_RS_HEADER);
    sum_type.push_str(type_);
    sum_type.push_str(" {\n");

    let mut load_fn = format!("pub fn load() -> Vec<{}> {{\n", type_);
    push_indent(&mut load_fn, 1);
    load_fn.push_str("vec![\n");

    let folder_path = src.join(folder);

    let mut count = 0;
    for file in read_dir(&folder_path)? {
        let path = PathBuf::from(file?.file_name());
        let stem = path.file_stem()
            .expect(format!("Invalid {} file name", name).as_ref())
            .to_str()
            .expect(format!("Invalid {} file name", name).as_ref());

        if let Some(ext) = path.extension() {
            if ext != "rs" || stem == "mod" {
                continue;
            }
        } else {
            continue;
        }

        info!("Found {} \"{}\"", name, stem);
        count += 1;

        mods.push_str("mod ");
        mods.push_str(stem);
        mods.push_str(";\n");

        let pascal_name = stem.to_case(Case::Pascal);

        imports.push_str("use ");
        imports.push_str(stem);
        imports.push_str("::");
        imports.push_str(&pascal_name);
        imports.push_str(";\n");

        push_indent(&mut sum_type, 1);
        sum_type.push_str(&pascal_name);
        sum_type.push_str(",\n");

        push_indent(&mut load_fn, 2);
        load_fn.push_str(&pascal_name);
        load_fn.push_str("::new().into(),\n");
    }

    if count == 0 {
        return Err(Error::other(format!("No {} module found", name)));
    }

    sum_type.push_str("}\n");

    push_indent(&mut load_fn, 1);
    load_fn.push_str("]\n}");

    let mut f = File::create(folder_path.join("mod.rs"))?;
    f.write_all(format!("{}\n{}\n{}\n{}", mods, imports, sum_type, load_fn).as_bytes())?;

    Ok(())
}

fn generate_language_module(folder: &str, src: &Path, msg: &str) -> Result<(), Error> {
    let mut mods    = String::new();
    let mut load_fn = String::from("pub fn define(layers: &mut std::collections::HashMap<std::rc::Rc<str>, crate::LanguageLayerFn>) {\n");

    let mut generate_headers_fn = String::from("#[cfg(feature = \"dev\")]\n");
    generate_headers_fn.push_str("pub fn generate_headers(\n");
    push_indent(&mut generate_headers_fn, 1);
    generate_headers_fn.push_str("globals: &std::collections::HashMap<std::rc::Rc<str>, crate::compiler::type_system::UniLType>\n");
    generate_headers_fn.push_str(") -> Result<(), std::io::Error> {\n");

    let folder_path = src.join(folder);

    for file in read_dir(&folder_path)? {
        let path = PathBuf::from(file?.file_name());
        let stem = path.file_stem()
            .expect(format!("Invalid {msg} layer file name").as_ref())
            .to_str()
            .expect(format!("Invalid {msg} layer file name").as_ref());

        if let Some(ext) = path.extension() {
            if ext != "rs" || stem == "mod" {
                continue;
            }
        } else {
            continue;
        }

        info!("Found {msg} layer \"{}\"", stem);

        mods.push_str("mod ");
        mods.push_str(stem);
        mods.push_str(";\n");

        push_indent(&mut load_fn, 1);
        load_fn.push_str(stem);
        load_fn.push_str("::define(layers);\n");

        push_indent(&mut generate_headers_fn, 1);
        generate_headers_fn.push_str("crate::__write_header_file!(");
        generate_headers_fn.push_str(stem);
        generate_headers_fn.push_str(", globals, \"headers\");\n");
    }

    load_fn.push('}');

    push_indent(&mut generate_headers_fn, 1);
    generate_headers_fn.push_str("Ok(())\n");
    generate_headers_fn.push('}');

    let mut f = File::create(folder_path.join("mod.rs"))?;
    f.write_all(format!("{}\n{}\n\n{}", mods, load_fn, generate_headers_fn).as_bytes())?;

    Ok(())
}

fn generate_api_layers_module(folder: &str, src: &Path) -> Result<(), Error> {
    let mut mods = String::from("");
    let mut load_fn = String::from("pub fn define(globals: &mut crate::univm::environment::Environment) {\n");
    let mut load_analyzer_fn = String::from("pub fn define_analyzer(globals: &mut crate::compiler::environment::AnalyzerEnvironment) {\n");

    let folder_path = src.join(folder);

    for file in read_dir(&folder_path)? {
        let path = PathBuf::from(file?.file_name());
        let stem = path.file_stem()
            .expect(format!("Invalid API layer file name").as_ref())
            .to_str()
            .expect(format!("Invalid API layer file name").as_ref());

        if let Some(ext) = path.extension() {
            if ext != "rs" || stem == "mod" {
                continue;
            }
        } else {
            continue;
        }

        info!("Found API layer \"{}\"", stem);

        mods.push_str("mod ");
        mods.push_str(stem);
        mods.push_str(";\n");

        push_indent(&mut load_fn, 1);
        load_fn.push_str(&stem);
        load_fn.push_str("::define(globals);\n");

        push_indent(&mut load_analyzer_fn, 1);
        load_analyzer_fn.push_str(&stem);
        load_analyzer_fn.push_str("::define_analyzer(globals);\n");
    }

    load_fn.push('}');
    load_analyzer_fn.push('}');

    let mut f = File::create(folder_path.join("mod.rs"))?;
    f.write_all(format!("{}\n{}\n\n{}", mods, load_fn, load_analyzer_fn).as_bytes())?;

    Ok(())
}

fn main() -> Result<(), Error> {
    println!("cargo::rerun-if-changed=src/");
    println!("cargo::rerun-if-changed=resources/");
    static_vcruntime::metabuild();

    let src = PathBuf::from("src");

    let resources = PathBuf::from("resources");
    let gen_folder = resources.join("gen");
    if !gen_folder.exists() {
        info!("\"gen\" folder does not exist. Creating it");
        create_dir(&gen_folder)?;
    }

    info!("Generating visuals module");
    generate_module("visual", "visuals", "AnyVisual", &src)?;

    info!("Generating sounds module");
    generate_module("sound", "sounds", "AnySound", &src)?;

    info!("Generating API layers module");
    generate_api_layers_module("api_layers", &src)?;

    info!("Generating language layers module");
    generate_language_module("language_layers", &src, "language")?;

    info!("Generating embeddable resized logo");
    let logo = ImageReader::open(resources.join("logo.png"))?.decode()
        .map_err(|e| Error::other(e.to_string()))?;

    let mut f = File::create(gen_folder.join("logo.png"))?;
    logo.resize(ICON_SIZE, ICON_SIZE, FilterType::Nearest)
        .write_to(&mut f, ImageFormat::Png)
        .map_err(|e| Error::other(e.to_string()))?;

    info!("Building UniV");
    Ok(())
}
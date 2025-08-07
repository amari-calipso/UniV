use std::{collections::HashMap, env, fs, io::{Error, ErrorKind}, path::{Path, PathBuf}, process::Command};

// UNIX-like: rustc dev_util.rs -o dev_util && ./dev_util <command>
// Windows: rustc dev_util.rs -o dev_util.exe && dev_util <command>

fn copy_dir_recursive(src: &PathBuf, dst: &Path) -> Result<(), Error> {
    for entry in fs::read_dir(src)? {
        let path = src.join(&entry?.file_name());

        if path.is_file() {
            fs::copy(&path, &dst.join(path.file_name().unwrap()))?;
        } else {
            let dst_new = dst.join(path.file_name().unwrap());
            fs::create_dir(&dst_new)?;
            copy_dir_recursive(&path, &dst_new)?;
        }
    }

    Ok(())
}

fn clone_dir(src: &PathBuf, dst: &Path) -> Result<(), Error> {
    fs::create_dir(dst)?;
    copy_dir_recursive(src, dst)
}

fn get_filename(target: &Option<String>) -> &str {
    if let Some(target) = &target {
        if target.contains("-windows-") {
            "univ.exe"
        } else {
            "univ"
        }
    } else {
        if cfg!(windows) {
            "univ.exe"
        } else {
            "univ"
        }
    }
}

fn generate_headers(as_release: bool) -> Result<(), Error> {
    println!("Generating headers for third-party languages");

    let mut command = Command::new("cargo");
    command.arg("run");

    if as_release {
        command.arg("--release");
    }

    command.args([
        "--features", "dev",
        "--", "--generate-headers"
    ]);

    if !command.status()?.success() {
        return Err(Error::other("Compilation failed"));
    }

    Ok(())
}

fn release(target: &Option<String>, lite: bool) -> Result<(), Error> {
    let mut command = Command::new("cargo");
    command.arg("build").arg("--release");

    if lite {
        command.arg("--features").arg("lite");
    }

    if let Some(target) = target {
        command.arg("--target").arg(target);
    }

    println!("Calling cargo");
    if !command.status()?.success() {
        return Err(Error::other("Build failed"));
    }

    let dist = PathBuf::from("dist");
    if dist.exists() {
        println!("\"dist\" folder exists. Deleting");
        fs::remove_dir_all(&dist)?;
    }

    println!("Creating \"dist\" folder");
    fs::create_dir(&dist)?;

    println!("Copying executable to \"dist\" folder");
    let mut executable = PathBuf::from("target").join("release");

    if let Some(target) = &target {
        executable.push(target);
    }

    let filename = get_filename(&target);
    executable.push(&filename);
    fs::copy(&executable, dist.join(&filename))?;

    if !lite {
        println!("Copying algorithms to \"dist\" folder");
        clone_dir(&PathBuf::from("algos"), &dist.join("algos"))?;
        println!("Copying headers to \"dist\" folder");
        clone_dir(&PathBuf::from("headers"), &dist.join("headers"))?;
    }
    
    println!("Copying automations to \"dist\" folder");
    clone_dir(&PathBuf::from("automations"), &dist.join("automations"))?;
    println!("Copying profiles to \"dist\" folder");
    clone_dir(&PathBuf::from("profiles"), &dist.join("profiles"))?;

    println!("Copying license to \"dist\" folder");
    fs::copy("LICENSE", &dist.join("LICENSE"))?;
    println!("Copying font license to \"dist\" folder");
    fs::copy(&PathBuf::from("resources").join("FONTLICENSE"), &dist.join("FONTLICENSE"))?;

    Ok(())
}

fn release_full(target: &Option<String>, no_gen_headers: bool) -> Result<(), Error> {
    if !no_gen_headers {
        generate_headers(true)?;
    }

    release(target, false)
}

fn compile_algos(as_release: bool, headers: bool) -> Result<(), Error> {
    println!("Compiling algorithms into bytecode");

    let mut command = Command::new("cargo");
    command.arg("run");

    if as_release {
        command.arg("--release");
    }

    command.args([
        "--features", "dev",
        "--", "--compile-algos"
    ]);

    if headers {
        command.arg("--with-headers");
    }

    if !command.status()?.success() {
        return Err(Error::other("Compilation failed"));
    }

    Ok(())
}

fn release_lite(target: &Option<String>, with_headers: bool) -> Result<(), Error> {
    compile_algos(true, with_headers)?;
    release(target, true)
}

fn run_lite(with_headers: bool) -> Result<(), Error> {
    compile_algos(false, with_headers)?;

    if Command::new("cargo")
        .arg("run")
        .arg("--features")
        .arg("lite")
        .status()?
        .success() 
    {
        Ok(())
    } else {
        Err(Error::other("Non-zero return code"))
    }    
}

macro_rules! ok_if_notfound {
    ($expr: expr) => {
        if let Err(e) = $expr {
            if !matches!(e.kind(), ErrorKind::NotFound) {
                return Err(e);
            }
        }
    };
}

fn clean() -> Result<(), Error> {
    if !Command::new("cargo").arg("clean").status()?.success() {
        return Err(Error::other("cargo clean failed"));
    }

    ok_if_notfound!(fs::remove_file(PathBuf::from("./algos.unib")));
    ok_if_notfound!(fs::remove_dir_all(PathBuf::from("./resources/gen")));
    ok_if_notfound!(fs::remove_dir_all(PathBuf::from("./headers")));
    Ok(())
}

macro_rules! commands {
    (
        ($args: ident, $args_map: ident);

        $($name: literal -> $action: expr)+
    ) => {
        if false { unreachable!() }
        $(
            else if let Some(idx) = $args_map.get($name) {
                $args.remove(*idx);
                $args_map.remove($name);
                $action;
            }
        )+
        else {
            println!("No option specified. Use {}", stringify!($($name,)+));
            return Err(Error::other("No option specified"));
        }
    };
}

#[allow(dead_code)] // shuts up rust-analyzer (it thinks this is a module for build.rs)
fn main() -> Result<(), Error> {
    let mut args: Vec<String> = env::args().collect();
    let mut args_map: HashMap<String, usize> = args.iter().enumerate().map(|(i, x)| (x.clone(), i)).collect();

    // only effective for --release-lite: generates headers while compiling algorithms
    let with_headers = args_map.contains_key("--with-headers");

    // only effective for --release: avoids generating headers before compiling (useful if compiling both versions)
    let no_gen_headers = args_map.contains_key("--no-gen-headers");
    
    // only effective for --release and --release-lite: specify custom target
    let target = {
        if let Some(idx) = args_map.get("--target") {
            args.remove(*idx);

            if *idx >= args.len() {
                return Err(Error::other("Target argument was supplied but no target was specified"));
            }

            Some(args.remove(*idx))
        } else {
            None
        }
    };

    commands! {
        (args, args_map);
        
        "--release"      -> release_full(&target, no_gen_headers)?
        "--release-lite" -> release_lite(&target, with_headers)?
        "--run-lite"     -> run_lite(with_headers)?
        "--gen-headers"  -> generate_headers(false)?
        "--clean"        -> clean()?
    };

    println!("Done!");
    Ok(())
}
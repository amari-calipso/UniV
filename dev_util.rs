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

fn release(args: &mut Vec<String>, args_map: &mut HashMap<String, usize>, lite: bool) -> Result<(), Error> {
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

    let mut command = Command::new("cargo");
    command.arg("build").arg("--release");

    if lite {
        command.arg("--features").arg("lite");
    }

    if let Some(target) = &target {
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
    }
    
    println!("Copying automations to \"dist\" folder");
    clone_dir(&PathBuf::from("automations"), &dist.join("automations"))?;
    println!("Copying profiles to \"dist\" folder");
    clone_dir(&PathBuf::from("profiles"), &dist.join("profiles"))?;

    Ok(())
}

fn compile_algos() -> Result<(), Error> {
    println!("Compiling algorithms into bytecode");

    let mut command = Command::new("cargo");
    command.arg("run").arg("--").arg("--compile-algos");

    if !command.status()?.success() {
        return Err(Error::other("Compilation failed"));
    }

    Ok(())
}

fn release_lite(args: &mut Vec<String>, args_map: &mut HashMap<String, usize>) -> Result<(), Error> {
    compile_algos()?;
    release(args, args_map, true)
}

fn run_lite() -> Result<(), Error> {
    compile_algos()?;

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

    commands! {
        (args, args_map);
        
        "--release"      -> release(&mut args, &mut args_map, false)?
        "--release-lite" -> release_lite(&mut args, &mut args_map)?
        "--run-lite"     -> run_lite()?
        "--clean"        -> clean()?
    };

    println!("Done!");
    Ok(())
}
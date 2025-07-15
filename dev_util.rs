use std::{cmp::min, collections::HashMap, env, fs::{self, File}, io::{Error, ErrorKind, Write}, path::{Path, PathBuf}, process::Command, str};

// UNIX-like: rustc dev_util.rs -o dev_util && ./dev_util <command>
// Windows: rustc dev_util.rs -o dev_util.exe && dev_util <command>

const ALGO_TEST_HEADER: &str = r#"
use std::{path::PathBuf, rc::Rc};
use crate::{automation::AutomationMode, get_expect, UniV, PROGRAM_DIR};
"#;

const ALGO_TEST_TEMPLATE: &str = r#"
#[test]
fn run_all_sorts_$d$_$s$() {
    let _ = PROGRAM_DIR.set(PathBuf::new()); // if it's already set, ignore
    
    let mut univ = UniV::new();
    univ.load_algos().expect("Failed to load algorithms");
    univ.shuffles.remove("Run automation"); // we don't care about this for tests
    univ.init_gui_algos(); // sync removal with gui

    univ.gui.run_all_sorts.speed = f64::INFINITY; // skip all render calls     
    univ.automation_interpreter.mode = AutomationMode::RunSorts;

    univ.gui.run_all_sorts.distribution = $d$;
    univ.gui.run_all_sorts.shuffle = $s$;

    univ.execute_automation(
        Rc::clone(&get_expect!(univ.run_all_sorts).source),
        Rc::clone(&get_expect!(univ.run_all_sorts).filename)
    ).expect("Failed to run automation");
}
"#;

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
    command.args(["build", "--release"]);

    if lite {
        command.args(["--features", "lite"]);
    }

    if let Some(target) = &target {
        command.args(["--target", target]);
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

    println!("Copying license to \"dist\" folder");
    fs::copy("LICENSE", &dist.join("LICENSE"))?;
    println!("Copying font license to \"dist\" folder");
    fs::copy(&PathBuf::from("resources").join("FONTLICENSE"), &dist.join("FONTLICENSE"))?;

    Ok(())
}

fn compile_algos() -> Result<(), Error> {
    println!("Compiling algorithms into bytecode");

    let mut command = Command::new("cargo");
    command.args([
        "run", 
        "--features", "dev",
        "--", "--compile-algos"
    ]);

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
        .args([
            "run",
            "--features", "lite"
        ])
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

fn test_algos() -> Result<(), Error> {
    println!("Counting algorithms");

    let mut command = Command::new("cargo");
    command.args([
        "run", 
        "--features", "dev",
        "--", "--load-algos"
    ]);

    let output = command.output()?;

    let stdout_output = str::from_utf8(&output.stdout)
        .expect("stdout contained invalid UTF-8");

    let stderr_output = str::from_utf8(&output.stderr)
        .expect("stderr contained invalid UTF-8");

    if !output.status.success() {
        println!("{}{}", stdout_output, stderr_output);
        return Err(Error::other("Compilation failed"));
    }
    
    let mut n_distributions = 0;
    let mut n_shuffles = 0;

    for line in stdout_output.split('\n') {
        if let Some(remaining) = line.strip_prefix("INFO: UniV: Loaded ") {
            let (n, type_) = remaining.split_once(' ')
                .expect("Unexpected output");

            match type_ {
                "distributions" => n_distributions = n.parse().expect("Unexpected output"),
                "shuffles" => n_shuffles = n.parse().expect("Unexpected output"),
                _ => continue
            }
        }
    }

    println!("Found {} distributions", n_distributions);
    println!("Found {} shuffles", n_shuffles);
    
    if n_distributions == 0 || n_shuffles == 0 {
        println!("No tests to run");
        return Err(Error::other("No tests to run"));
    }

    println!("Generating tests");

    {
        let mut test_file = File::create("./src/dev/tests/generated.rs")?;
        test_file.write_all(ALGO_TEST_HEADER.as_bytes())?;

        for distribution in 0 .. n_distributions {
            for shuffle in 0 .. n_shuffles {
                let code = ALGO_TEST_TEMPLATE
                    .replace("$d$", &distribution.to_string())
                    .replace("$s$", &shuffle.to_string());

                test_file.write_all(code.as_bytes())?;
            }
        }
    }

    let n_threads = min(n_distributions * n_shuffles, std::thread::available_parallelism()?.into());

    println!("Running tests with {} threads", n_threads);

    unsafe {
        env::set_var("RUST_BACKTRACE", "full");
    }

    command = Command::new("cargo");
    command.args([
        "test",
        "--features", "dev",
        "--release",
        "--", 
        "--test-threads", &n_threads.to_string()
    ]);
    
    if !command.status()?.success() {
        return Err(Error::other("Non-zero return code"));
    }

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
        "--test-algos"   -> test_algos()?
    };

    println!("Done!");
    Ok(())
}
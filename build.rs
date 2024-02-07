use std::{env, fs};
use std::path;

fn main() -> Result<(), std::io::Error> {
    println!("cargo:rerun-if-changed=src/templates");
    println!("cargo:rerun-if-changed=src/static");

    let d = get_cargo_target_dir().unwrap();

    let bp= path::PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    if let Err(e) = recursive_copy(bp.join("static"), &d.join("static")) {
        return Err(e)
    }

    if let Err(e) = recursive_copy(bp.join("templates"), &d.join("templates")) {
        return Err(e)
    }

    return Ok(());
}

fn get_cargo_target_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let out_dir = path::PathBuf::from(std::env::var("OUT_DIR")?);
    let profile = env::var("PROFILE")?;
    let mut target_dir = None;
    let mut sub_path = out_dir.as_path();
    while let Some(parent) = sub_path.parent() {
        if parent.ends_with(&profile) {
            target_dir = Some(parent);
            break;
        }
        sub_path = parent;
    }
    let target_dir = target_dir.ok_or("not found")?;
    Ok(target_dir.to_path_buf())
}

fn recursive_copy(src: impl AsRef<path::Path>, dest: impl AsRef<path::Path>) -> Result<(), std::io::Error> {
    fs::create_dir_all(&dest).unwrap();
    
    for file in fs::read_dir(src).unwrap() {
        match file {
            Ok(f) => {
                match f.file_type()?.is_dir() {
                    true => {
                        recursive_copy(f.path(), dest.as_ref().join(f.file_name())).unwrap();
                    },
                    false => {
                        fs::copy(f.path(), dest.as_ref().join(f.file_name())).unwrap();
                    }
                }
            },
            Err(e) => return Err(e),
        };
    };

    return Ok(());
}

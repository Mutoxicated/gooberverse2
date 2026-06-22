use std::{ffi::OsStr, os::windows::fs::MetadataExt};

macro_rules! pln {
    ($($tokens:tt)*) => {
        println!("cargo::warning={}", format!($($tokens)*))
    };
}

fn main() {
    let cwd = std::env::current_dir().unwrap();
    pln!("Starting build script...");
    pln!("Cwd: {}", cwd.display());

    let mesh_assets_dir = cwd.join("assets/meshes/");
    let read_dir = mesh_assets_dir.read_dir();
    assert!(read_dir.is_ok());
    let mut has_blender_files = false;
    for a in read_dir.unwrap() {
        match a {
            Err(x) => pln!("Unexpected error reading files: {}", x),
            Ok(x) => {
                let md = x.metadata().unwrap();
                let path = x.path();
                let ext = path.extension().unwrap_or_default();
                if md.is_file() && ext == OsStr::new("blend") {
                    has_blender_files = true;
                    break;
                }
            }
        }
    }
    if !has_blender_files {
        pln!("No blender files spotted. Exiting early.");
        return;
    }

    let explicit_blender_path = std::env::var("BLENDER_PATH");
    if let Ok(p) = explicit_blender_path {
        
    }
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("Failed to get target os");
    pln!("{}!", target_os);
    pln!("Trying to find blender...");

    let home = std::env::home_dir();
    pln!("Home: {:?}", home);
    if home.is_none() {
        println!(
            "cargo::error=Couldn't find the home directory, in this case you need to pass the blender path as an env variable."
        );
        return;
    }
}

// use std::{ffi::OsStr, os::windows::fs::MetadataExt, path::PathBuf, process::Command};

// macro_rules! pln {
//     ($($tokens:tt)*) => {
//         println!("cargo::warning={}", format!($($tokens)*))
//     };
// }

// fn main() {
//     let cwd = std::env::current_dir().unwrap();
//     pln!("Starting build script...");
//     pln!("Cwd: {}", cwd.display());

//     let mesh_assets_dir = cwd.join("assets/meshes/");
//     let read_dir = mesh_assets_dir.read_dir();
//     assert!(read_dir.is_ok());

//     let mut blender_files: Vec<String> = Vec::new();
//     let blend_extension = OsStr::new("blend");
//     for a in read_dir.unwrap() {
//         match a {
//             Err(x) => pln!("Unexpected error reading files: {}", x),
//             Ok(x) => {
//                 let md = x.metadata().unwrap();
//                 let path = x.path();
//                 let ext = path.extension().unwrap_or_default();
//                 if md.is_file() && ext == blend_extension {
//                     blender_files.push(path.to_string_lossy().into_owned());
//                     break;
//                 }
//             }
//         }
//     }
//     if blender_files.is_empty() {
//         pln!("No blender files spotted. Exiting early.");
//         return;
//     }
//     pln!("Trying to find blender...");

//     fn export_blend_files(cwd: &PathBuf, blender_path: PathBuf, blender_files: Vec<String>) {
//         pln!("Starting exportation of blender files...");
//         let gltf_exporter_path = cwd.join(".utils/glTF-Blender-IO/");
//         let utils_path = cwd.join(".utils/");
//         if std::fs::read_dir(&gltf_exporter_path).is_err() {
//             pln!("No './utils/glTF-Blender-IO' path! Creating one...");
//             let _ = std::fs::create_dir(&utils_path);
//             let out = Command::new("git")
//                 .arg("clone")
//                 .arg("https://github.com/KhronosGroup/glTF-Blender-IO")
//                 .arg(&gltf_exporter_path)
//                 .output();
//             pln!("{out:?}");
//         }
//     }

//     let mut blender_path: PathBuf = OsStr::new("").into();
//     let explicit_blender_path = std::env::var("BLENDER_PATH");
//     if let Ok(p) = explicit_blender_path {
//         blender_path = p.into();
//         export_blend_files(&cwd, blender_path, blender_files);
//         return;
//     }

//     let p = std::env::var("PATH");
//     if let Ok(x) = p {
//         let paths = x.split(';');
//         blender_path = paths
//             .into_iter()
//             .find(|a| a.contains("Blender"))
//             .unwrap_or_default()
//             .into();
//         if !blender_path.as_os_str().is_empty() {
//             export_blend_files(&cwd, blender_path, blender_files);
//             return;
//         }
//     }

//     let target_os = std::env::var("CARGO_CFG_TARGET_OS").expect("Failed to get target os");
//     pln!("{}!", target_os);

//     match target_os.as_str() {
//         "windows" => {
//             let dir = std::fs::read_dir("C:/Program Files/").unwrap_or_else(|err| panic!("{err}"));
//             for d in dir {
//                 if d.is_err() {
//                     continue;
//                 }
//                 let entry = d.unwrap();
//                 let path = entry.path();
//                 if path.is_file() {
//                     continue;
//                 }
//                 if path.ends_with("Blender Foundation") {
//                     let mut bdir = path.read_dir().unwrap_or_else(|err| panic!("{err}"));
//                     blender_path = bdir.next().unwrap().unwrap().path();
//                     pln!("Found the blender path! {}", blender_path.clone().display());
//                     break;
//                 }
//             }
//         }
//         _ => {}
//     }
//     if !blender_path.as_os_str().is_empty() {
//         export_blend_files(&cwd, blender_path, blender_files);
//         return;
//     }
//     if target_os == "windows" {
//         println!(
//             "cargo::error=Couldn't find the blender path. The script checked for 'C:/Program Files'"
//         );
//     }
// }
fn main() {}


use std::{
  env,
  path::PathBuf,
};

fn main() {
  assert!(cfg!(target_os = "windows"), "Building only supported on Windows.");

  let cubism_core_dir = env::var("LIVE2D_CUBISM_CORE_DIR").expect("Environment variable LIVE2D_CUBISM_CORE_DIR not found !");

  const WRAPPER_HEADER: &str = "src/wrapper.h";

  println!("cargo:rerun-if-changed={}", WRAPPER_HEADER);

  let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

  let core_platform_lib_dir_path = {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    
    let arch_dir_name =
      match target_os.as_str() {
        "windows" => target_arch.as_str(),
        "android" =>
          match target_arch.as_str() {
            "aarch64" => "arm64-v8a",
            "arm" => "armeabi-v7a",
            v => v,
          },
        "linux" => target_arch.as_str(),
        target_os => panic!("Unexpected target_os: {}", target_os),
      };
    let platform_lib_dir_name =
      match target_os.as_str() {
        "windows" => format!("{}/142", arch_dir_name),
        _ => arch_dir_name.to_owned(),
      };

    PathBuf::from(&cubism_core_dir).join("lib").join(&target_os).join(platform_lib_dir_name)
  };
  let core_platform_lib_name = {
    match target_os.as_str() {
      "windows" => "Live2DCubismCore_MD",
      _ => "Live2DCubismCore"
    }
  };

  println!("cargo:rustc-link-search=native={}", core_platform_lib_dir_path.to_str().unwrap());
  println!("cargo:rustc-link-lib=static={}", core_platform_lib_name);

  let core_include_dir_path = PathBuf::from(&cubism_core_dir).join("include");

  let bindings_builder = bindgen::Builder::default()
    .header(WRAPPER_HEADER)
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .clang_arg(format!("-I{}", core_include_dir_path.to_str().unwrap()));

  let bindings = bindings_builder.generate().expect("Unable to generate bindings !");

  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_dir.join("bindings.rs"))
    .expect("Failed to write bindings !");
}

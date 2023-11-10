extern crate bindgen;
extern crate cc;

use std::error::Error;
use std::path::{Path, PathBuf};

use bindgen::Bindings;

fn build_lib<O, LD, L, const N: usize, const M: usize>(out_dir: O, library_dirs: LD, libraries: L, defines: &[(&str, &str); N], include_dirs: &[String; M]) -> Result<(), Box<dyn Error>>
where
  O: AsRef<Path>,
  LD: Iterator,
  LD::Item: AsRef<Path>,
  L: Iterator,
  L::Item: AsRef<str>
{
  let mut build = cc::Build::default();

  build
    .cpp(true)
    .file("wrapper.cpp")
    .include(out_dir)
    .flag("-std=c++11")
    .flag("-w");

  for library_dir in library_dirs {
    build.flag(format!("-L{}", library_dir.as_ref().display()).as_str());
  }

  for library in libraries {
    build.flag(format!("-l{}", library.as_ref()).as_str());
  }

  build
    .cargo_metadata(true)
    .static_flag(true);
  
  for (key, value) in defines.iter() {
    build.define(*key, Some(*value));
  }
  
  for include_dir in include_dirs.iter() {
    build.include(include_dir);
  }

  
  build.compile("vkfft");

  Ok(())
}

#[cfg(feature = "vendored")]
fn build_vkfft() -> PathBuf {
  let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/VkFFT/");
  println!("cargo:rerun-if-changed={}", dir.display());
  let dst = cmake::build(dir);
  dst
}

fn gen_wrapper<F, const N: usize, const M: usize>(file: F,  defines: &[(&str, &str); N], include_dirs: &[String; M]) -> Result<Bindings, Box<dyn Error>>
where
  F: AsRef<Path>,
{
  let base_args = [
    "-std=c++11".to_string(),
  ];
  
  let defines: Vec<String> = defines.iter().map(|(k, v)| {
    format!("-D{}={}", k, v)
  }).collect();

  let include_dirs: Vec<String> = include_dirs.iter()
    .map(|s| format!("-I{}", s))
    .collect();

  let clang_args = base_args 
    .iter()
    .chain(defines.iter())
    .chain(include_dirs.iter());

  println!("{:?}", clang_args);

  


  let res = bindgen::Builder::default()
    .clang_args(clang_args)
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .header(file.as_ref().to_str().unwrap())
    .allowlist_recursively(true)
    .allowlist_type("VkFFTConfiguration")
    .allowlist_type("VkFFTLaunchParams")
    .allowlist_type("VkFFTResult")
    .allowlist_type("VkFFTSpecializationConstantsLayout")
    .allowlist_type("VkFFTPushConstantsLayout")
    .allowlist_type("VkFFTAxis")
    .allowlist_type("VkFFTPlan")
    .allowlist_type("VkFFTApplication")
    .allowlist_function("VkFFTSync")
    .allowlist_function("VkFFTAppend")
    .allowlist_function("VkFFTPlanAxis")
    .allowlist_function("initializeVkFFT")
    .allowlist_function("deleteVkFFT")
    .allowlist_function("VkFFTGetVersion")
    
    .generate();
  
  let bindings = match res {
    Ok(x) => x,
    Err(_) => {
      eprintln!("Failed to generate bindings.");
      std::process::exit(1);
    }
  };

  Ok(bindings)
}

fn main() -> Result<(), Box<dyn Error>> {
  let vkfft_root = std::env::var("VKFFT_ROOT");
  let out_dir = std::env::var("OUT_DIR")?;
  let out_dir = PathBuf::from(out_dir);

  #[cfg(not(feature = "vendored"))]
  let library_dirs = {
    let vkfft_root = vkfft_root?;
    [
      format!("{}/build/glslang-main/glslang", vkfft_root),
      format!("{}/build/glslang-main/glslang/OSDependent/Unix", vkfft_root),
      format!("{}/build/glslang-main/glslang/OGLCompilersDLL", vkfft_root),
      format!("{}/build/glslang-main/SPIRV", vkfft_root),
    ]
  };

  #[cfg(feature = "vendored")]
  let (library_dirs, vkfft_root) = {
    let vkfft_libroot = if let Ok(ref r) = vkfft_root {
      PathBuf::from(format!("{r}/build"))
    } else {
      build_vkfft()
    };
    let vkfft_root = if let Ok(r) = vkfft_root {
      PathBuf::from(r)
    } else {
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor/VkFFT/")
    };
    ([
      vkfft_libroot.join("glslang-main/glslang"),
      vkfft_libroot.join("glslang-main/glslang/OSDependent/Unix"),
      vkfft_libroot.join("glslang-main/OGLCompilersDLL"),
      vkfft_libroot.join("glslang-main/SPIRV"),
    ], vkfft_root)
  };

  let libraries = [
    "glslang",
    "MachineIndependent",
    "OSDependent",
    "GenericCodeGen",
    "OGLCompiler",
    "vulkan",
    "SPIRV"
  ];

  for library_dir in library_dirs.iter() {
    println!("cargo:rustc-link-search={}", library_dir.display());
  }

  for library in libraries.iter() {
    println!("cargo:rustc-link-lib={}", library);
  }


  println!("cargo:rerun-if-changed=wrapper.cpp");
  println!("cargo:rerun-if-changed=build.rs");

  let include_dirs = [
    format!("{}", vkfft_root.join("vkFFT").display()),
    format!("{}", vkfft_root.join("glslang-main/glslang/Include").display())
  ];

  let defines = [
    ("VKFFT_BACKEND", "0"),
    ("VK_API_VERSION", "11")
  ];

  let wrapper = std::fs::read_to_string(vkfft_root.join("vkFFT/vkFFT.h"))
    .map_err(|e| format!("couldn't read {}: {e}", vkfft_root.join("vkFFT/vkFFT.h").display()))?
    .replace("static inline", "");

  let rw = out_dir.join("vkfft_rw.hpp");
  std::fs::write(&rw, wrapper.as_str())?;

  build_lib(&out_dir, library_dirs.iter(), libraries.iter(), &defines, &include_dirs)?;

  let bindings = gen_wrapper(&rw, &defines, &include_dirs)?;
  bindings.write_to_file(out_dir.join("bindings.rs"))?;
  
  Ok(())
}

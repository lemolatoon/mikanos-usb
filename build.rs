use std::{io, process::Command};

use fs_extra::dir::CopyOptions;
use glob::glob;
use walkdir::WalkDir;

fn main() {
    let cargo_home = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let cargo_home1 = cargo_home.clone();
    let handle = std::thread::spawn(move || {
        let generated_lib_path = std::path::Path::new(&cargo_home)
            .join("target")
            .join("stdlib")
            .join("x86_64-elf")
            .join("lib");
        if !generated_lib_path.exists() {
            // check cache
            if std::path::Path::new(&cargo_home)
                .join(".cache")
                .join("stdlib")
                .exists()
            {
                let copy_options = CopyOptions::new().overwrite(true);
                let to = std::path::Path::new(&cargo_home).join("target");
                std::fs::create_dir_all(&to)?;
                fs_extra::dir::copy(
                    std::path::Path::new(&cargo_home)
                        .join(".cache")
                        .join("stdlib"),
                    to,
                    &copy_options,
                )
                .unwrap();
                return io::Result::Ok(true);
            }
            // delete ./docker-container-id if exists
            let docker_container_id_path =
                std::path::Path::new(&cargo_home).join("docker-container-id");
            if docker_container_id_path.exists() {
                std::fs::remove_file(docker_container_id_path)?;
            }
            let successed = Command::new("./build.sh")
                .current_dir(cargo_home.clone())
                .status()
                .map(|e| e.success())?;
            if !successed {
                return io::Result::Ok(true);
            }
            for entry in WalkDir::new(generated_lib_path) {
                let entry = entry.unwrap();
                if entry.file_type().is_dir() {
                    continue;
                }
                let path = entry.path();
                println!("processing {}", path.display());
                let file_name = path.file_name().unwrap().to_str().unwrap();
                assert!(file_name.starts_with("lib"));
                let renamed = path.with_file_name(format!("libstd{}", &file_name[3..]));
                std::os::unix::fs::symlink(path, renamed)?;
            }
            // cache
            let from = std::path::Path::new(&cargo_home)
                .join("target")
                .join("stdlib");
            let to = std::path::Path::new(&cargo_home).join(".cache");
            std::fs::create_dir_all(&to)?;
            println!("copying {:?} to {:?}", from, to);
            let copy_options = CopyOptions::new().overwrite(true);
            fs_extra::dir::copy(from, to, &copy_options).unwrap();
            io::Result::Ok(true)
        } else {
            Ok(true)
        }
    });
    let path_lib = format!("{}/target/stdlib/x86_64-elf/lib", &cargo_home1);
    let path_include = format!("{}/target/stdlib/x86_64-elf/include", &cargo_home1);
    let path_include_cpp = format!("{}/target/stdlib/x86_64-elf/include/c++/v1", &cargo_home1);
    let cxx_flags: String = format!(
        "-Wall -g -ffreestanding -mno-red-zone -fno-exceptions -std=c++17 -fpermissive -fno-rtti -L {} -I {} -I {} -I {} -I {}/external -w",
        path_lib, path_include_cpp, path_include, &cargo_home1, &cargo_home1
    );
    let mut cc = cxx_build::bridge("src/lib.rs");
    for flag in cxx_flags.split_whitespace() {
        cc.flag(flag);
    }
    let file_iter = glob("usb/**/*.cpp")
        .unwrap()
        .chain(glob("external/**/*.cpp").unwrap());

    let file_iter = file_iter.chain(glob("test_cpp/**/*.cpp").unwrap());
    let file_iter: Vec<_> = file_iter.map(|e| e.unwrap()).collect();
    assert!(handle.join().unwrap().unwrap());
    cc.files(file_iter.clone()).compile("usb");

    for file in file_iter {
        println!("cargo:rerun-if-changed={}", file.display());
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
}

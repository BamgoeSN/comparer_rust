use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RunLang {
    C,
    Cpp,
    Python,
    Java,
    Go,
    Rust,
}

pub fn compile(
    lang: RunLang,
    code_path: impl AsRef<OsStr> + AsRef<Path>,
    exec_path: impl AsRef<OsStr> + AsRef<Path>,
) -> Option<PathBuf> {
    // Compilation can run asynchronously
    match lang {
        RunLang::C => {
            Command::new("gcc")
                .arg(code_path)
                .arg("-o")
                .arg(&exec_path)
                .args(["-O2", "-Wall", "-lm", "-static", "-std=gnu++20"])
                .spawn()
                .and_then(|mut x| x.wait())
                .expect("Failed to execute compilation");
        }
        RunLang::Cpp => {
            Command::new("g++")
                .arg(code_path)
                .arg("-o")
                .arg(&exec_path)
                .args(["-O2", "-Wall", "-lm", "-static", "-std=gnu++20"])
                .spawn()
                .and_then(|mut x| x.wait())
                .expect("Failed to execute compilation");
        }
        RunLang::Python => {
            Command::new("cp")
                .arg(code_path)
                .arg(&exec_path)
                .spawn()
                .and_then(|mut x| x.wait())
                .expect("Failed to execute compilation");
        }
        RunLang::Java => {
            todo!();
            let code_dir: &Path = code_path.as_ref();
            let mut code_dir = code_dir.to_owned();
            code_dir.pop();

            let mut manifest = code_dir.clone();
            manifest.push("manifest");
            manifest.set_extension("txt");

            Command::new("javac")
                .args(["-encoding", "utf-8"])
                .arg(&code_path)
                .spawn()
                .and_then(|mut x| x.wait())
                .expect("Failed to execute compilation");

            let mut all_class = code_dir.clone();
            all_class.push("*.class");

            let exec_path_str: &Path = exec_path.as_ref();
            println!(
                "{}",
                format!(
                    "jar -cvmf {} {} {}",
                    manifest.to_str().unwrap(),
                    exec_path_str.to_str().unwrap(),
                    all_class.to_str().unwrap()
                )
            );
            Command::new("bash")
                .arg("-c")
                .arg(format!(
                    "jar -cvmf {} {} {}",
                    manifest.to_str().unwrap(),
                    exec_path_str.to_str().unwrap(),
                    all_class.to_str().unwrap()
                ))
                .spawn()
                .and_then(|mut x| x.wait())
                .expect("Failed to execute compilation");

            return None;

            Command::new("rm").arg(&all_class);
        }
        RunLang::Go => {
            Command::new("go")
                .arg("build")
                .arg("-o")
                .arg(&exec_path)
                .arg(code_path)
                .spawn()
                .and_then(|mut x| x.wait())
                .expect("Failed to execute compilation");
        }
        RunLang::Rust => {
            Command::new("rustc")
                .args(["--edition", "2021", "-O", "-o"])
                .arg(&exec_path)
                .arg(&code_path)
                .spawn()
                .and_then(|mut x| x.wait())
                .expect("Failed to execute compilation");
        }
    }
    let out: &Path = exec_path.as_ref();
    if out.exists() {
        Some(out.to_owned())
    } else {
        None
    }
}

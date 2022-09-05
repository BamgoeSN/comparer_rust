use std::{
    env,
    ffi::OsStr,
    fs, io,
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
            let code_dir: &Path = code_path.as_ref();
            let mut code_dir = code_dir.to_owned();
            code_dir.pop();
            let code_dir = code_dir.to_str()?;
            // let mut manifest = code_dir.clone();
            // manifest.push("manifest");
            // manifest.set_extension("txt");
            // let mut all_class = code_dir.clone();
            // all_class.push("*.class");

            Command::new("javac")
                .args(["-encoding", "utf-8"])
                .arg(&code_path)
                .spawn()
                .and_then(|mut x| x.wait())
                .expect("Failed to execute compilation");

            let exec_path: &Path = exec_path.as_ref();
            let exec_path = absolute_path(exec_path).ok()?;
            let exec_path = exec_path.to_str()?;
            Command::new("bash")
                .arg("-c")
                .arg(format!(
                    "cd {code_dir} && jar -cvmf manifest.txt {exec_path} *.class && rm *.class"
                ))
                .spawn()
                .and_then(|mut x| x.wait())
                .expect("Failed to execute compilation");
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

pub fn absolute_path(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };

    Ok(absolute_path)
}

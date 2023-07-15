use std::{
    env,
    ffi::OsStr,
    io::{self, Result},
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

impl TryFrom<&str> for RunLang {
    type Error = std::io::Error;
    fn try_from(value: &str) -> Result<Self> {
        use io::ErrorKind::*;
        use RunLang::*;
        match value {
            "c" => Ok(C),
            "cpp" | "c++" => Ok(Cpp),
            "py" | "python" | "pypy" => Ok(Python),
            "java" => Ok(Java),
            "go" | "golang" => Ok(Go),
            "rust" | "rs" => Ok(Rust),
            _ => Err(io::Error::new(InvalidInput, "Wrong language name")),
        }
    }
}

pub fn compile(
    lang: RunLang,
    code_path: impl AsRef<Path>,
    exec_dir: impl AsRef<Path>,
    exec_name: impl AsRef<Path>,
) -> Result<PathBuf> {
    let code_path = absolute_path(code_path)?;
    let exec_dir = absolute_path(exec_dir)?;

    let exec_path: PathBuf = {
        let mut exec_path = exec_dir;
        exec_path.push(&exec_name);
        exec_path.set_extension(match lang {
            RunLang::Python => "py",
            RunLang::Java => "jar",
            _ => "exe",
        });
        exec_path
    };

    // Compilation can run asynchronously
    match lang {
        RunLang::C => {
            Command::new("gcc")
                .arg(&code_path)
                .arg("-o")
                .arg(&exec_path)
                .args(["-O2", "-Wall", "-lm", "-static", "-std=gnu11"])
                .spawn()
                .and_then(|mut x| x.wait())
                .expect("Failed to execute compilation");
        }
        RunLang::Cpp => {
            Command::new("g++")
                .arg(&code_path)
                .arg("-o")
                .arg(&exec_path)
                .args(["-O2", "-Wall", "-lm", "-static", "-std=gnu++17"])
                .spawn()
                .and_then(|mut x| x.wait())
                .expect("Failed to execute compilation");
        }
        RunLang::Python => {
            Command::new("cp")
                .arg(&code_path)
                .arg(&exec_path)
                .spawn()
                .and_then(|mut x| x.wait())
                .expect("Failed to execute compilation");
        }
        RunLang::Java => {
            let mut code_dir = code_path.clone();
            code_dir.pop();
            let code_dir = code_dir.to_str().unwrap();

            Command::new("javac")
                .args(["-encoding", "utf-8"])
                .arg(&code_path)
                .spawn()
                .and_then(|mut x| x.wait())
                .expect("Failed to execute compilation");

            let exec_path = absolute_path(&exec_path)?;
            let exec_path = exec_path.to_str().unwrap();
            Command::new("bash")
                .arg("-c")
                .arg(format!(
                    "cd {code_dir} && jar -cf {exec_path} *.class && rm *.class"
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
                .arg(&code_path)
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

    if exec_path.exists() {
        Ok(exec_path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Requested executable couldn't be generated",
        ))
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

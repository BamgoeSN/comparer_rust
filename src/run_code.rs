use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, Result, Write},
    path::{Path, PathBuf},
};

use rand::random;
use sha2::{Digest, Sha256};
use subprocess::*;

pub fn run(exec: impl AsRef<OsStr>, input: &str, dir_input: impl AsRef<Path>) -> Result<String> {
    let input_loc = generate_file(&dir_input, input)?;
    let input_file = fs::File::open(&input_loc)?;

    let outerr = Exec::shell(exec)
        .stdin(input_file)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .capture()
        .map_err(|x| match x {
            PopenError::IoError(y) => {
                io::Error::new(y.kind(), "IoError while running child process")
            }
            PopenError::LogicError(y) => io::Error::new(
                io::ErrorKind::Other,
                format!("LogicError while running child process: {}", y),
            ),
            _ => io::Error::new(
                io::ErrorKind::Other,
                "Unknown error while running child process",
            ),
        })?
        .stdout_str();

    fs::remove_file(input_loc)?;

    Ok(outerr)
}

fn generate_file(dir: impl AsRef<Path>, content: &str) -> Result<PathBuf> {
    let mut file_path = dir.as_ref().to_owned();
    file_path.push(random_name());
    file_path.set_extension("txt");
    let mut file = File::create(file_path.clone())?;
    file.write_all(content.as_bytes())?;
    Ok(file_path)
}

fn random_name() -> String {
    let mut buf = String::new();
    for _ in 0..10 {
        buf.push_str(&format!("{}", random::<i64>()));
    }

    let mut hasher = Sha256::new();
    hasher.update(&buf);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter;

    #[tokio::test]
    async fn aplusb() {
        let inputs = vec!["1 2", "3 5", "2 6", "4 1"];
        let outputs: Vec<_> = inputs
            .iter()
            .map(|s| {
                let arr: Vec<i32> = s.split_whitespace().map(|x| x.parse().unwrap()).collect();
                format!("{}", arr[0] + arr[1])
            })
            .collect();

        let handles: Vec<_> = inputs
            .iter()
            .map(|s| {
                tokio::spawn(async {
                    run("./src/test-binary/aplusb.exe", s, "./src/test-binary/temp/")
                })
            })
            .collect();

        let mut returns: Vec<_> = Vec::new();
        for h in handles.into_iter() {
            let x = h.await.unwrap().unwrap();
            returns.push(x);
        }

        for (a, b) in iter::zip(outputs.iter(), returns.iter()) {
            assert_eq!(a.trim(), b.trim());
        }
    }
}

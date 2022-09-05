use core::time;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Stdio,
};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt, Result},
    process::Command,
    time::timeout,
};

pub async fn run(
    command: impl AsRef<OsStr>,
    args: &[impl AsRef<OsStr>],
    input: &str,
    dir_input: impl AsRef<Path>,
    duration: time::Duration,
) -> Result<String> {
    let input_loc = generate_file(&dir_input, input).await?;
    let input_file = fs::File::open(&input_loc).await?;

    let mut proc = Command::new(command)
        .args(args)
        .stdin(Stdio::from(input_file.into_std().await))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if timeout(duration, proc.wait()).await.is_err() {
        return Ok("Timeout".to_owned());
    }

    proc.wait().await?;

    let mut out = String::new();
    proc.stdout.unwrap().read_to_string(&mut out).await?;

    fs::remove_file(input_loc).await?;

    Ok(out)
}

async fn generate_file(dir: impl AsRef<Path>, content: &str) -> Result<PathBuf> {
    let mut file_path = dir.as_ref().to_owned();
    file_path.push(random_name());
    file_path.set_extension("txt");
    let mut file = File::create(file_path.clone()).await?;
    file.write_all(content.as_bytes()).await?;
    Ok(file_path)
}

fn random_name() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

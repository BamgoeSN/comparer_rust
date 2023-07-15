use core::time;
use std::{
    borrow::Borrow,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt, Result},
    process::Command,
    time::timeout,
};

use crate::core::compile::RunLang;

pub async fn run(
    command: impl AsRef<OsStr>,
    args: &[impl AsRef<OsStr>],
    input: &str,
    dir_input: impl AsRef<Path>,
    duration: time::Duration,
) -> Result<String> {
    let input_loc = generate_file_with_random_name(&dir_input, input).await?;
    let input_file = fs::File::open(&input_loc).await?;

    let mut proc = Command::new(command)
        .args(args)
        .stdin(Stdio::from(input_file.into_std().await))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if timeout(duration, proc.wait()).await.is_err() {
        fs::remove_file(input_loc).await?;
        return Ok("Timeout".to_owned());
    }

    proc.wait().await?;

    let mut out = String::new();
    proc.stdout.unwrap().read_to_string(&mut out).await?;

    let mut err = String::new();
    proc.stderr.unwrap().read_to_string(&mut err).await?;

    fs::remove_file(input_loc).await?;

    if err.is_empty() {
        Ok(out)
    } else {
        Ok(err)
    }
}

async fn generate_file_with_random_name(dir: impl AsRef<Path>, content: &str) -> Result<PathBuf> {
    let mut file_path = dir.as_ref().to_owned();
    file_path.push(random_name());
    file_path.set_extension("txt");
    let mut file = File::create(file_path.clone()).await?;
    file.write_all(content.as_bytes()).await?;
    Ok(file_path)
}

pub async fn get_results(
    lang: RunLang,
    prog: impl AsRef<Path>,
    inputs: impl Borrow<[String]>,
    time_limit: Duration,
) -> Vec<String> {
    let inputs = inputs.borrow();
    let prog = prog.as_ref();
    let mut cr_handles: Vec<_> = Vec::with_capacity(inputs.len());

    for input in inputs.iter().cloned() {
        let h = match lang {
            RunLang::Python => {
                let arr: Vec<_> = vec![prog.to_owned()];
                tokio::spawn(async move {
                    run("python3", &arr, &*input, "./compile/temp/", time_limit).await
                })
            }

            RunLang::Java => {
                let arr: Vec<_> = vec![
                    "-classpath".to_owned(),
                    prog.to_owned().to_str().unwrap().to_owned(),
                    "Main".to_owned(),
                ];
                tokio::spawn(async move {
                    run("java", &arr, &*input, "./compile/temp/", time_limit).await
                })
            }

            _ => {
                let prog = prog.to_owned();
                tokio::spawn(async move {
                    run(
                        prog,
                        &[] as &[String],
                        &*input,
                        "./compile/temp/",
                        time_limit,
                    )
                    .await
                })
            }
        };

        cr_handles.push(h);
    }

    let mut arr: Vec<String> = Vec::with_capacity(cr_handles.len());
    for h in cr_handles {
        let x = h.await.unwrap().unwrap();
        arr.push(x);
    }
    arr
}

fn random_name() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

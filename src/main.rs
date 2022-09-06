use std::{fmt::Write, path::Path, sync::Arc, time::Duration};

use clap::{Parser, Subcommand};
use comparer_rust::{
    compile::{compile, RunLang},
    inputgen::generate_multi,
    run_code,
    string::process_str,
};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use parking_lot::Mutex;
use tokio::io::Result;

const TC_DEFAULT: usize = 100;
const BATCH_SIZE: usize = 10;
const TIME_LIMIT: u64 = 2000; // ms

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate testcases of a given number and prints it out for debugging purpose
    Inputdebug {
        /// The number of testcases to generate
        num: usize,
    },
    /// Compare the outputs of two programs and check if they're equal
    Compare {
        /// Language of the code of the correct answer
        cr: String,
        /// Language of the code of the wrong answer
        wr: String,
        /// The number of testcases (defaults at 100)
        tc: Option<usize>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inputdebug { num } => {
            for (tc, h) in generate_multi(num).await.enumerate() {
                println!("Testcase {}\n{}", tc, h.await?);
            }
        }
        Commands::Compare { cr, wr, tc } => {
            let tc = tc.unwrap_or(TC_DEFAULT);
            let cr_lang: RunLang = cr.as_str().try_into()?;
            let wr_lang: RunLang = wr.as_str().try_into()?;

            let wrong_count: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
            let sent_count = wrong_count.clone();

            let cr_code_path = match cr_lang {
                RunLang::C => "./compile/cr/main.c",
                RunLang::Cpp => "./compile/cr/main.cpp",
                RunLang::Python => "./compile/cr/main.py",
                RunLang::Java => "./compile/cr/Main.java",
                RunLang::Go => "./compile/cr/main.go",
                RunLang::Rust => "./compile/cr/main.rs",
            };

            let wr_code_path = match wr_lang {
                RunLang::C => "./compile/wr/main.c",
                RunLang::Cpp => "./compile/wr/main.cpp",
                RunLang::Python => "./compile/wr/main.py",
                RunLang::Java => "./compile/wr/Main.java",
                RunLang::Go => "./compile/wr/main.go",
                RunLang::Rust => "./compile/wr/main.rs",
            };

            let cr_prog = compile(cr_lang, cr_code_path, "./compile/cr/", "cr")?;
            let wr_prog = compile(wr_lang, wr_code_path, "./compile/wr/", "wr")?;

            let pb = ProgressBar::new(tc as u64);
            pb.set_style(ProgressStyle::with_template("{spinner:.blue} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} ({eta})  Found: {wrong_count:<7}")
                .unwrap()
                .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
                .with_key("wrong_count", move |_: &ProgressState, w: &mut dyn Write| write!(w, "{}", *sent_count.lock()).unwrap())
                .progress_chars("=>-"));

            for start in (0..tc).step_by(BATCH_SIZE) {
                let end = (start + BATCH_SIZE).min(tc);
                let batch = end - start;

                let mut inputs: Vec<String> = Vec::with_capacity(batch);
                for h in generate_multi(batch).await {
                    inputs.push(h.await?);
                }

                let cr_results: Vec<String> = get_results(cr_lang, cr_prog.clone(), &inputs).await;
                let wr_results: Vec<String> = get_results(wr_lang, wr_prog.clone(), &inputs).await;

                let wrongs: Vec<usize> = (0..batch)
                    .filter(|&i| process_str(&cr_results[i]) != process_str(&wr_results[i]))
                    .collect();

                for &i in wrongs.iter() {
                    *wrong_count.lock() += 1;
                    println!("Input");
                    println!("{}", inputs[i]);
                    println!("Correct Answer");
                    println!("{}", cr_results[i]);
                    println!("Wrong Output");
                    println!("{}", wr_results[i]);
                    println!("");
                }

                pb.inc((end - start) as u64);
            }

            let wrong_count = *wrong_count.lock();
            match wrong_count {
                0 => eprintln!("No wrong answers found"),
                1 => eprintln!("1 wrong ansewr found"),
                _ => eprintln!("{wrong_count} wrong answers found"),
            };
        }
    }

    Ok(())
}

pub async fn get_results(lang: RunLang, prog: impl AsRef<Path>, inputs: &[String]) -> Vec<String> {
    let mut cr_handles: Vec<_> = Vec::with_capacity(inputs.len());

    for input in inputs.iter().cloned() {
        let h = match lang {
            RunLang::Python => {
                let arr: Vec<_> = vec![prog.as_ref().to_owned()];
                tokio::spawn(async move {
                    run_code::run(
                        "python3",
                        &arr,
                        input,
                        "./compile/temp/",
                        Duration::from_millis(TIME_LIMIT),
                    )
                    .await
                })
            }

            RunLang::Java => {
                let arr: Vec<_> = vec![
                    "-classpath".to_owned(),
                    prog.as_ref().to_owned().to_str().unwrap().to_owned(),
                    "Main".to_owned(),
                ];
                tokio::spawn(async move {
                    run_code::run(
                        "java",
                        &arr,
                        input,
                        "./compile/temp/",
                        Duration::from_millis(TIME_LIMIT),
                    )
                    .await
                })
            }

            _ => {
                let prog = prog.as_ref().to_owned();
                tokio::spawn(async move {
                    run_code::run(
                        prog,
                        &[] as &[String],
                        input,
                        "./compile/temp/",
                        Duration::from_millis(TIME_LIMIT),
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

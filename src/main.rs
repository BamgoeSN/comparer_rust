use std::time::Duration;

use clap::{Parser, Subcommand};
use comparer_rust::{
    compile::{compile, RunLang},
    inputgen::generate_multi,
    run_code,
};
use futures::future;
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

            let cr_code_path = match cr_lang {
                RunLang::C => "./compile/cr/main.c",
                RunLang::Cpp => "./compile/cr/main.cpp",
                RunLang::Python => "./compile/cr/main.py",
                RunLang::Java => "./compile/cr/main.java",
                RunLang::Go => "./compile/cr/main.go",
                RunLang::Rust => "./compile/cr/main.rs",
            };

            let wr_code_path = match wr_lang {
                RunLang::C => "./compile/wr/main.c",
                RunLang::Cpp => "./compile/wr/main.cpp",
                RunLang::Python => "./compile/wr/main.py",
                RunLang::Java => "./compile/wr/main.java",
                RunLang::Go => "./compile/wr/main.go",
                RunLang::Rust => "./compile/wr/main.rs",
            };

            let cr_prog = compile(cr_lang, cr_code_path, "./compile/cr/", "cr")?;
            let wr_prog = compile(wr_lang, wr_code_path, "./compile/wr/", "wr")?;

            let mut wr_cnt: usize = 0;

            for start in (0..tc).step_by(BATCH_SIZE) {
                let end = (start + BATCH_SIZE).min(tc);
                let batch = end - start;

                let mut inputs: Vec<String> = Vec::with_capacity(batch);
                for h in generate_multi(batch).await {
                    inputs.push(h.await?);
                }

                let cr_results: Vec<String> = async {
                    let cr_handles: Vec<_> = inputs
                        .iter()
                        .map(|input| {
                            tokio::spawn(run_code::run(
                                cr_prog.clone(),
                                &[] as &[String],
                                input.to_owned(),
                                "./compile/temp/",
                                Duration::from_millis(TIME_LIMIT),
                            ))
                        })
                        .collect();
                    let mut arr: Vec<_> = Vec::with_capacity(batch);
                    for h in cr_handles {
                        arr.push(h.await.unwrap().unwrap());
                    }
                    arr
                }
                .await;

                let wr_results: Vec<String> = async {
                    let wr_handles: Vec<_> = inputs
                        .iter()
                        .map(|input| {
                            tokio::spawn(run_code::run(
                                wr_prog.clone(),
                                &[] as &[String],
                                input.to_owned(),
                                "./compile/temp/",
                                Duration::from_millis(TIME_LIMIT),
                            ))
                        })
                        .collect();
                    let mut arr: Vec<_> = Vec::with_capacity(batch);
                    for h in wr_handles {
                        arr.push(h.await.unwrap().unwrap());
                    }
                    arr
                }
                .await;
            }

            todo!()
        }
    }

    Ok(())
}

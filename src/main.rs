use std::time::Duration;

use clap::{Parser, Subcommand};
use comparer_rust::{
    compile::{compile, RunLang},
    inputgen::generate_multi,
    run_code,
    string::process_str,
};
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

            let mut wrong_count: usize = 0;

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

            for start in (0..tc).step_by(BATCH_SIZE) {
                let end = (start + BATCH_SIZE).min(tc);
                let batch = end - start;

                let mut inputs: Vec<String> = Vec::with_capacity(batch);
                for h in generate_multi(batch).await {
                    inputs.push(h.await?);
                }

                let cr_results: Vec<String> = {
                    let mut cr_handles: Vec<_> = Vec::with_capacity(batch);
                    for input in inputs.iter().cloned() {
                        let h = if let RunLang::Python = cr_lang {
                            let arr: Vec<_> = vec![cr_prog.clone()];
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
                        } else {
                            let prog = cr_prog.clone();
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
                        };
                        cr_handles.push(h);
                    }
                    let mut arr: Vec<String> = Vec::with_capacity(batch);
                    for h in cr_handles {
                        let x = h.await.unwrap().unwrap();
                        arr.push(x);
                    }
                    arr
                };

                let wr_results: Vec<String> = {
                    let mut wr_handles: Vec<_> = Vec::with_capacity(batch);
                    for input in inputs.iter().cloned() {
                        let h = if let RunLang::Python = wr_lang {
                            let arr: Vec<_> = vec![wr_prog.clone()];
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
                        } else {
                            let prog = wr_prog.clone();
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
                        };
                        wr_handles.push(h);
                    }
                    let mut arr: Vec<String> = Vec::with_capacity(batch);
                    for h in wr_handles {
                        let x = h.await.unwrap().unwrap();
                        arr.push(x);
                    }
                    arr
                };

                let wrongs: Vec<usize> = (0..batch)
                    .filter(|&i| process_str(&cr_results[i]) != process_str(&wr_results[i]))
                    .collect();

                for &i in wrongs.iter() {
                    wrong_count += 1;
                    println!("Input");
                    println!("{}", inputs[i]);
                    println!("Correct Answer");
                    println!("{}", cr_results[i]);
                    println!("Wrong Output");
                    println!("{}", wr_results[i]);
                    println!("");
                }
            }

            match wrong_count {
                0 => println!("No wrong answers detected"),
                1 => println!("1 wrong ansewr detected"),
                _ => println!("{wrong_count} wrong answers detected"),
            };
        }
    }

    Ok(())
}

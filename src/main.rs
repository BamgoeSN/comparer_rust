use std::{fmt::Write, sync::Arc, time::Duration};

use clap::{Parser, Subcommand};
use comparer_rust::{
    core::{
        compile::{compile, RunLang},
        run_code::get_results,
        string::process_str,
    },
    inputgen::generate_multi,
};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use parking_lot::RwLock;
use tokio::io::Result;

const TC_DEFAULT: usize = 100;
const BATCH_SIZE: usize = 10;
const TIME_LIMIT_DEFAULT: i64 = 2000; // ms

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
        /// Time limit in milliseconds (defaults at 2000)
        tl: Option<i64>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inputdebug { num } => {
            input_debug(num).await?;
        }
        Commands::Compare { cr, wr, tc, tl } => {
            compare(
                &cr,
                &wr,
                tc.unwrap_or(TC_DEFAULT),
                tl.unwrap_or(TIME_LIMIT_DEFAULT),
            )
            .await?;
        }
    }

    Ok(())
}

async fn get_actual_time_limit(lang: RunLang, tl: i64) -> Duration {
    use RunLang::*;
    let rtl = match lang {
        C | Cpp | Rust => tl,
        Go => tl + 2000,
        Java => 2 * tl + 1000,
        Python => 3 * tl + 2000,
    };
    Duration::from_millis(if rtl < 0 { 0 } else { rtl.unsigned_abs() })
}

async fn input_debug(num: usize) -> Result<()> {
    for (tc, h) in generate_multi(num).await.enumerate() {
        println!("Testcase {}\n```\n{}\n```", tc, h.await?);
    }
    Ok(())
}

async fn compare(cr: &str, wr: &str, tc: usize, tl: i64) -> Result<()> {
    let cr_lang: RunLang = cr.try_into()?;
    let wr_lang: RunLang = wr.try_into()?;

    let cr_tl = get_actual_time_limit(cr_lang, tl).await;
    let wr_tl = get_actual_time_limit(wr_lang, tl).await;

    let wrong_count = Arc::new(RwLock::new(0usize));
    let wrong_writer = wrong_count.clone();
    let sent_count = wrong_count.clone();

    let code_file = match cr_lang {
        RunLang::C => "main.c",
        RunLang::Cpp => "main.cpp",
        RunLang::Python => "main.py",
        RunLang::Java => "Main.java",
        RunLang::Go => "main.go",
        RunLang::Rust => "main.rs",
    };
    let cr_code_path = format!("./compile/cr/{code_file}");
    let wr_code_path = format!("./compile/wr/{code_file}");

    let cr_prog = compile(cr_lang, &cr_code_path, "./compile/cr/", "cr")?;
    let wr_prog = compile(wr_lang, &wr_code_path, "./compile/wr/", "wr")?;

    let pb = ProgressBar::new(tc as u64);
    pb.set_style(ProgressStyle::with_template("{spinner:.blue} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} ({eta})  Found: {wrong_count:<7}")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .with_key("wrong_count", move |_: &ProgressState, w: &mut dyn Write| write!(w, "{}", *sent_count.read()).unwrap())
        .progress_chars("=> "));

    for start in (0..tc).step_by(BATCH_SIZE) {
        let end = (start + BATCH_SIZE).min(tc);
        let batch = end - start;

        let mut inputs: Vec<String> = Vec::with_capacity(batch);
        for h in generate_multi(batch).await {
            inputs.push(h.await?);
        }
        let inputs: Arc<[String]> = inputs.into();

        let cr_results: Vec<String> = get_results(cr_lang, &cr_prog, inputs.clone(), cr_tl).await;
        let wr_results: Vec<String> = get_results(wr_lang, &wr_prog, inputs.clone(), wr_tl).await;

        let wrongs: Vec<usize> = (0..batch)
            .filter(|&i| process_str(&cr_results[i]) != process_str(&wr_results[i]))
            .collect();

        *wrong_writer.write() += wrongs.len();
        for &i in wrongs.iter() {
            println!("Input");
            println!("{}", inputs[i]);
            println!("Correct Answer");
            println!("{}", cr_results[i]);
            println!("Wrong Output");
            println!("{}", wr_results[i]);
            println!();
        }

        pb.inc((end - start) as u64);
    }

    let wrong_count = *wrong_count.read();
    match wrong_count {
        0 => eprintln!("No wrong answers found"),
        x => eprintln!("# of wrong answers: {x}"),
    };

    Ok(())
}

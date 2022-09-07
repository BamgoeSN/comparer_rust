use rand::{thread_rng, Rng};
use std::{fmt::Write, future::Future};
use tokio::task::JoinError;

use crate::core::string::process_str;

#[allow(unused)]
pub async fn generate_input() -> String {
    let mut buf = String::new();
    let mut rng = thread_rng();

    macro_rules! out { ($($arg:tt)*) => { write!(buf, $($arg)*).unwrap(); }; }
    macro_rules! outln { ($($arg:tt)*) => { writeln!(buf, $($arg)*).unwrap(); }; }

    let n: usize = rng.gen_range(1..=5);
    outln!("{n}");
    for _ in 0..n {
        let start: u32 = rng.gen_range(0..=1000);
        let end: u32 = rng.gen_range(start..=1000);
        outln!("{start} {end}");
    }

    process_str(&buf)
}

/// Returns an iterator of "num" spawned tasks of generating inputs.
pub async fn generate_multi(
    num: usize,
) -> impl Iterator<Item = impl Future<Output = Result<String, JoinError>>> {
    let arr: Vec<_> = (0..num).map(|_| tokio::spawn(generate_input())).collect();
    arr.into_iter()
}

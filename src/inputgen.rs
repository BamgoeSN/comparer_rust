#![allow(unused)]

use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{fmt::Write, future::Future};
use tokio::task::JoinError;

use crate::{core::string::process_str, genhelper::tree_gen::Tree};

pub async fn generate_input() -> String {
    let mut buf = String::new();
    let mut rng = thread_rng();

    macro_rules! print { ($($arg:tt)*) => { write!(buf, $($arg)*).unwrap(); }; }
    macro_rules! println { ($($arg:tt)*) => { writeln!(buf, $($arg)*).unwrap(); }; }

    let n: usize = rng.gen_range(1..=50000);
    println!("{}", n);

    process_str(&buf)
}

/// Returns an iterator of "num" spawned tasks of generating inputs.
pub async fn generate_multi(
    num: usize,
) -> impl Iterator<Item = impl Future<Output = Result<String, JoinError>>> {
    (0..num).map(|_| tokio::spawn(generate_input()))
}

use rand::{seq::SliceRandom, thread_rng};
use std::{fmt::Write, future::Future};
use tokio::task::JoinError;

pub async fn generate_input() -> String {
    let mut buf = String::new();
    let mut rng = thread_rng();

    let n: usize = 2;
    writeln!(buf, "{n}").unwrap();
    let mut arr: Vec<_> = (1..=n).collect();
    arr.shuffle(&mut rng);
    for v in arr {
        writeln!(buf, "{v}").unwrap();
    }

    buf
}

/// Returns an iterator of "num" spawned tasks of generating inputs.
pub async fn generate_multi(
    num: usize,
) -> impl Iterator<Item = impl Future<Output = Result<String, JoinError>>> {
    let arr: Vec<_> = (0..num).map(|_| tokio::spawn(generate_input())).collect();
    arr.into_iter()
}

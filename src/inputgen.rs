use core::future::Future;

use rand::{thread_rng, Rng};
use tokio::task::JoinError;

pub async fn generate_input() -> String {
    // a+b
    let a: i32 = thread_rng().gen_range(1..=10);
    let b: i32 = thread_rng().gen_range(1..=10);
    format!("{a} {b}\n")
}

/// Returns an iterator of "num" spawned tasks of generating inputs.
pub async fn generate_multi(
    num: usize,
) -> impl Iterator<Item = impl Future<Output = Result<String, JoinError>>> {
    let arr: Vec<_> = (0..num).map(|_| tokio::spawn(generate_input())).collect();
    arr.into_iter()
}

use rand::{thread_rng, Rng};

pub fn generate_input() -> String {
    // a+b
    let a: i32 = thread_rng().gen_range(1..=10);
    let b: i32 = thread_rng().gen_range(1..=10);
    format!("{a} {b}\n")
}

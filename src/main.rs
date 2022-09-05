use comparer_rust::{compile::compile, run_code::run};

fn main() {
    let x = compile(
        comparer_rust::compile::RunLang::Java,
        "./compile/java/Main.java",
        "./compile/java/test.jar",
    );
    println!("{:?}", x);
}

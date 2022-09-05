use comparer_rust::compile::compile;

fn main() {
    let x = compile(
        comparer_rust::compile::RunLang::C,
        "./compile/cpp/main.c",
        "./compile/cpp/test.exe",
    );
    println!("{:?}", x);
}

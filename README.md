# comparer-rust
comparer-rust generates randum inputs, runs in two different codes and compare inputs.
The comparison is based on the way it's done in online judges for competitive programming.
![image](https://user-images.githubusercontent.com/60645387/188674182-4e4fe1ce-de72-4285-b2e3-7b62942b3f9c.png)


## General Workflow
1. Write a Rust code which returns random testcases in `pub async fn generate_input() -> String` from `src/inputgen.rs`. You can generate a few testcases with this function for testing by executing `cargo run -- inputdebug <NUM>`.
2. Put a code which outputs the correct answer in `compile/cr`, and the one with the possibly wrong answer in `compile/wr`. The name of the code should be `Main.java` if the code is in java, or `main.<ext>` otherwise.
3. Run `cargo run --release -- compare <CR> <WR> [TC]`. `<CR>` and `<WR>` is a language each code is written in. [TC] is an optional value for the number of testcases to test, which defaults in 100.


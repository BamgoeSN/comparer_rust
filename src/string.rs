use std::fmt::Write;

pub fn process_str(s: &str) -> String {
    let mut builder = String::new();
    for l in s.lines() {
        writeln!(builder, "{}", l.trim()).unwrap();
    }
    builder
}

use std::fmt::Write;

pub fn process_str(s: &str) -> String {
    let mut builder = String::new();
    for l in s.lines() {
        writeln!(builder, "{}", l.trim_end()).unwrap();
    }
    let trimmed_len = builder.trim_end().len();
    while builder.len() > trimmed_len + 1 {
        builder.pop();
    }
    builder
}

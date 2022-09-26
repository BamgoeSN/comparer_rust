pub mod core;
pub mod genhelper;
pub mod inputgen;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{iter, sync::Arc, time::Duration};
    use tokio::io::Result;

    #[tokio::test]
    async fn check_run_code() -> Result<()> {
        let inputs = vec![
            "1 2", "3 5", "2 6", "4 1", "1 2", "3 5", "2 6", "4 1", "1 2", "3 5", "2 6", "4 1",
        ];
        let arcs: Vec<_> = inputs.iter().map(|&s| Arc::new(s)).collect();
        let outputs: Vec<_> = inputs
            .iter()
            .map(|s| {
                let arr: Vec<i32> = s.split_whitespace().map(|x| x.parse().unwrap()).collect();
                format!("{}", arr[0] + arr[1])
            })
            .collect();

        let handles: Vec<_> = arcs
            .into_iter()
            .map(|s| {
                tokio::spawn(core::run_code::run(
                    "./src/test-binary/aplusb.exe",
                    &[] as &[String],
                    *s,
                    "./src/test-binary/temp/",
                    Duration::from_secs(10),
                ))
            })
            .collect();

        let mut returns: Vec<_> = Vec::new();
        for h in handles {
            let x = h.await??;
            returns.push(x);
        }

        for (a, b) in iter::zip(outputs.iter(), returns.iter()) {
            assert_eq!(a.trim(), b.trim());
        }

        Ok(())
    }
}

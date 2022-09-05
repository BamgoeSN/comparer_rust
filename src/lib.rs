pub mod compile;
pub mod run_code;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{iter, time::Duration};

    #[tokio::test]
    async fn test_run_code() {
        let inputs = vec![
            "1 2", "3 5", "2 6", "4 1", "1 2", "3 5", "2 6", "4 1", "1 2", "3 5", "2 6", "4 1",
        ];
        let outputs: Vec<_> = inputs
            .iter()
            .map(|s| {
                let arr: Vec<i32> = s.split_whitespace().map(|x| x.parse().unwrap()).collect();
                format!("{}", arr[0] + arr[1])
            })
            .collect();

        let handles: Vec<_> = inputs
            .iter()
            .map(|s| {
                tokio::spawn(run_code::run(
                    "./src/test-binary/aplusb.exe",
                    &[] as &[String],
                    s,
                    "./src/test-binary/temp/",
                    Duration::from_secs(10),
                ))
            })
            .collect();

        let mut returns: Vec<_> = Vec::new();
        for h in handles {
            let x = h.await.unwrap().unwrap();
            returns.push(x);
        }

        println!("{:?}", returns);
        for (a, b) in iter::zip(outputs.iter(), returns.iter()) {
            assert_eq!(a.trim(), b.trim());
        }
    }
}

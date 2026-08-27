use std::io::{self, Write};

/// Stampa un prompt e legge una riga da stdin
pub async fn read_line(prompt: &str) -> Result<String, io::Error> {
    print!("{}", prompt);
    io::stdout().flush()?;

    tokio::task::spawn_blocking(|| {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    })
    .await
    .unwrap_or_else(|e| Err(io::Error::new(io::ErrorKind::Other, e)))
}

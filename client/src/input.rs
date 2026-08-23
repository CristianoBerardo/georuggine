use std::io::{self, Write};

/// Stampa un prompt e legge una riga da stdin, restituendola già "trimmata".
pub fn read_line(prompt: &str) -> Result<String, io::Error> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

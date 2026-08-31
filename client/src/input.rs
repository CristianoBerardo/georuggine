use crate::console;
use std::io::{self, Write};

// Stampa un prompt e legge una riga da stdin, segnalando al coordinatore
// della console che l'utente sta scrivendo: i messaggi in arrivo nel
// frattempo vengono accodati invece di interrompere la riga.
// Da usare per input di contenuto (messaggi, username, ...).
pub async fn read_line(prompt: &str) -> Result<String, io::Error> {
    print!("{}", prompt);
    io::stdout().flush()?;

    console::begin_typing();
    let result = blocking_read_line().await;
    console::end_typing();

    result
}

// Come read_line, ma non segnala l'inizio di una digitazione. Da usare per
// i prompt a scelta (scelte del menu): non c'è nulla da proteggere, e
// vogliamo che un messaggio in arrivo mentre si è fermi su questo prompt
// venga mostrato subito invece di aspettare che l'utente prema Invio.
pub async fn read_choice(prompt: &str) -> Result<String, io::Error> {
    print!("{}", prompt);
    io::stdout().flush()?;
    blocking_read_line().await
}

// Funzione che si occupa della lettura vera e propria della riga
async fn blocking_read_line() -> Result<String, io::Error> {
    tokio::task::spawn_blocking(|| {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    })
    .await
    .unwrap_or_else(|e| Err(io::Error::new(io::ErrorKind::Other, e)))
}

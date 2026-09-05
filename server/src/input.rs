// use crate::console;
// use std::io::{self, Write};

// // Stampa un prompt e legge una riga da stdin, restituendola già "trimmata".
// // Segnala al coordinatore della console che l'operatore sta scrivendo: i
// // messaggi in arrivo nel frattempo vengono accodati invece di interrompere
// // la riga. Da usare per input di contenuto (messaggi, username, ...).
// pub fn read_line(prompt: &str) -> Result<String, io::Error> {
//     print!("{}", prompt);
//     io::stdout().flush()?;

//     console::begin_typing();
//     let mut input = String::new();
//     let result = io::stdin().read_line(&mut input);
//     console::end_typing();

//     result.map(|_| input.trim().to_string())
// }

// // Come read_line, ma non segnala l'inizio di una digitazione. Da usare per
// // i prompt a scelta: non c'è nulla da proteggere, e vogliamo che un evento in
// // arrivo mentre si è fermi su questo prompt venga mostrato subito invece di
// // aspettare che l'operatore prema Invio.
// pub fn read_choice(prompt: &str) -> Result<String, io::Error> {
//     print!("{}", prompt);
//     io::stdout().flush()?;
//     let mut input = String::new();
//     io::stdin().read_line(&mut input)?;
//     Ok(input.trim().to_string())
// }

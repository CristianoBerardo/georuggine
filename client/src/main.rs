use tokio::io::BufReader;
use tokio::net::TcpStream;

mod auth;
mod input;
mod listener;
mod menu;
mod messaging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== GeoRuggine Client CLI ===");

    // 1. Connessione al server TCP
    println!("\nConnessione a 127.0.0.1:8080 in corso...");
    let stream = match TcpStream::connect("127.0.0.1:8080").await {
        Ok(s) => {
            println!("Connessione stabilita con successo!");
            s
        }
        Err(e) => {
            eprintln!("Impossibile connettersi al server: {}", e);
            return Ok(());
        }
    };

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // 2. Login o registrazione
    let authenticated = auth::authenticate(&mut reader, &mut writer).await?;
    if !authenticated {
        println!("\nOperazione completata. Disconnessione.");
        return Ok(());
    }

    // 3. Avvio del listener per i messaggi asincroni dal server
    let listener_handle = tokio::spawn(listener::listen(reader));

    // 4. Menu principale
    menu::menu(&mut writer).await?;

    // L'utente ha scelto di disconnettersi: non ha senso aspettare che il
    // listener se ne accorga da solo (aspetterebbe che sia il server a
    // chiudere la connessione), lo terminiamo subito.
    listener_handle.abort();
    println!("\nOperazione completata. Disconnessione.");
    Ok(())
}

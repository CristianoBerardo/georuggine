use client::auth::authenticate;
use client::listener::listen;
use client::messaging::send_message;
use client::movement_sim::MovementStatus;
use client::movement_sim::movement_sim;
use client::tools::movement_file_picker::next_movement_file;
use client::tools::read_movement_data::read_movement_data;
use client::ui::main_ui::{ExitReason, run};
use common::protocol::{ClientMessage, ServerMessage};
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::mpsc::channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== GeoRuggine Client CLI ===");

    let movement_file = next_movement_file();
    let positions = read_movement_data(movement_file)?;

    // L'intera sessione (connessione, autenticazione, menu principale) viene
    // ripetuta da capo quando l'utente elimina il proprio account, così da
    // tornare alla schermata di login/registrazione senza chiudere il processo.
    loop {
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
        let reader = BufReader::new(reader);

        // 2. Canale MPSC per inviare messaggi al server da vari tasks
        let (client_msg_tx, mut client_msg_rx) = channel::<ClientMessage>(100);

        // Canale per inoltrare i messaggi del server ad auth
        let (server_msg_tx, mut server_msg_rx) = channel::<ServerMessage>(100);

        // Canale per notificare alla UI gli errori dei task in background (writer, movement_sim)
        let (client_error_tx, client_error_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        // 3. Task dedicato alla scrittura: riceve da rx e chiama send_message su writer
        let writer_error_tx = client_error_tx.clone();
        let writer_handle = tokio::spawn(async move {
            while let Some(msg) = client_msg_rx.recv().await {
                if let Err(e) = send_message(&mut writer, &msg).await {
                    let message = format!("[WRITER] Errore nell'invio del messaggio: {}", e);
                    let _ = writer_error_tx.send(message);
                    break;
                }
            }
        });

        // 4. Avvio immediato del listener per i messaggi asincroni dal server
        let listener_handle = tokio::spawn(listen(reader, server_msg_tx));

        // 5. Login o registrazione
        let Some(username) = authenticate(&client_msg_tx, &mut server_msg_rx).await? else {
            println!("\nOperazione completata. Disconnessione.");
            listener_handle.abort();
            drop(client_msg_tx);
            let _ = writer_handle.await;
            return Ok(());
        };

        // 6. Avvio della simulazione del movimento
        let (movement_status_tx, mut movement_status_rx) =
            tokio::sync::watch::channel(MovementStatus::default());
        let movement_handle = tokio::spawn(movement_sim(
            positions.clone(),
            client_msg_tx.clone(),
            movement_status_tx,
            client_error_tx,
        ));
        // Menu principale della main_ui
        let exit_reason = run(
            username,
            &client_msg_tx,
            &mut server_msg_rx,
            &mut movement_status_rx,
            client_error_rx,
        )
        .await?;
        // Pulizia e chiusura ordinata della sessione
        movement_handle.abort();
        listener_handle.abort();

        // `tx` (il sender originale) e tutti i suoi clone rimasti attivi negli
        // altri task vanno droppati prima di aspettare `writer_handle`
        drop(client_msg_tx);
        let _ = writer_handle.await;

        match exit_reason {
            ExitReason::UserQuit => {
                println!("\nUscita dall'applicazione. Disconnessione dal server...");
                return Ok(());
            }
            ExitReason::ConnectionLost => {
                eprintln!("\nConnessione al server persa. Uscita dall'applicazione.");
                return Ok(());
            }
            ExitReason::AccountDeleted => {
                println!("\nAccount eliminato. Torno alla schermata di login/registrazione...");
                // continua il loop: si riconnette e mostra di nuovo l'auth
            }
        }
    }
}

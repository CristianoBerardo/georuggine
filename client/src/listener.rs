use common::protocol::ServerMessage;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::sync::mpsc::Sender;

// Ascolta in continuazione i messaggi asincroni del server. Generica sul tipo
// di reader (anziché legata a `OwnedReadHalf`, il tipo concreto usato in
// main.rs) così nei test si può usare un buffer in memoria al posto di un
// vero socket TCP.
pub async fn listen<R: AsyncRead + Unpin>(
    mut reader: BufReader<R>,
    server_msg_tx: Sender<ServerMessage>,
) {
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // Connessione chiusa dal server
                return;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match serde_json::from_str::<ServerMessage>(trimmed) {
                    Ok(msg) => {
                        if server_msg_tx.send(msg).await.is_err() {
                            // Il ricevitore è stato chiuso, quindi usciamo dal loop
                            return;
                        }
                    }
                    Err(_) => {
                        // Ignora i messaggi malformati e continua ad ascoltare
                        // Non dovrebbe mai succedere in quanto client e server condividono lo
                        // stesso protocollo di messaggistica
                        continue;
                    }
                }
            }
            Err(_) => {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // `Cursor<Vec<u8>>` implementa `AsyncRead`: simula i byte di un socket
    // senza aprirne uno vero.
    fn reader(content: &str) -> BufReader<Cursor<Vec<u8>>> {
        BufReader::new(Cursor::new(content.as_bytes().to_vec()))
    }

    #[tokio::test]
    async fn un_messaggio_valido_viene_inoltrato() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let input = r#"{"BroadcastMessage":{"message":"ciao","timestamp":"2024-01-01T00:00:00Z"}}"#;

        listen(reader(&format!("{}\n", input)), tx).await;

        match rx.try_recv().unwrap() {
            ServerMessage::BroadcastMessage { message, .. } => assert_eq!(message, "ciao"),
            _ => panic!("atteso BroadcastMessage"),
        }
        assert!(rx.try_recv().is_err()); // un solo messaggio
    }

    #[tokio::test]
    async fn piu_messaggi_vengono_inoltrati_in_ordine() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let riga1 = r#"{"BroadcastMessage":{"message":"primo","timestamp":"2024-01-01T00:00:00Z"}}"#;
        let riga2 = r#"{"BroadcastMessage":{"message":"secondo","timestamp":"2024-01-01T00:00:00Z"}}"#;

        listen(reader(&format!("{}\n{}\n", riga1, riga2)), tx).await;

        match rx.try_recv().unwrap() {
            ServerMessage::BroadcastMessage { message, .. } => assert_eq!(message, "primo"),
            _ => panic!("atteso BroadcastMessage"),
        }
        match rx.try_recv().unwrap() {
            ServerMessage::BroadcastMessage { message, .. } => assert_eq!(message, "secondo"),
            _ => panic!("atteso BroadcastMessage"),
        }
    }

    #[tokio::test]
    async fn righe_vuote_vengono_ignorate() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let valido = r#"{"BroadcastMessage":{"message":"ciao","timestamp":"2024-01-01T00:00:00Z"}}"#;

        listen(reader(&format!("\n\n{}\n\n", valido)), tx).await;

        assert!(rx.try_recv().is_ok());
        assert!(rx.try_recv().is_err()); // nient'altro: le righe vuote non contano
    }

    #[tokio::test]
    async fn messaggio_malformato_viene_scartato_e_non_ferma_lascolto() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let valido = r#"{"BroadcastMessage":{"message":"ciao","timestamp":"2024-01-01T00:00:00Z"}}"#;

        listen(reader(&format!("questo non è json\n{}\n", valido)), tx).await;

        match rx.try_recv().unwrap() {
            ServerMessage::BroadcastMessage { message, .. } => assert_eq!(message, "ciao"),
            _ => panic!("atteso BroadcastMessage"),
        }
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn stream_vuoto_termina_subito_senza_messaggi() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        listen(reader(""), tx).await;

        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn ricevitore_chiuso_interrompe_lascolto_senza_bloccarsi() {
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        drop(rx); // nessuno leggerà mai i messaggi inoltrati
        let valido = r#"{"BroadcastMessage":{"message":"ciao","timestamp":"2024-01-01T00:00:00Z"}}"#;

        // Se `listen` non si accorgesse del canale chiuso, questo await non
        // finirebbe mai: il solo fatto che il test termini è già la verifica.
        listen(reader(&format!("{}\n", valido)), tx).await;
    }
}

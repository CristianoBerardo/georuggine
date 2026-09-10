mod support;

use chrono::Utc;
use common::protocol::{ClientMessage, ServerMessage};
use support::{connect, recv, send, spawn_test_server};

#[tokio::test]
async fn un_messaggio_diretto_arriva_solo_al_destinatario() {
    let (addr, state, _chat_rx) = spawn_test_server().await;

    // Mario si registra
    let (mut mario_reader, mut mario_writer) = connect(addr).await;
    send(
        &mut mario_writer,
        &ClientMessage::Register {
            username: "mario".to_string(),
            password: "password_mario".to_string(),
        },
    )
    .await;
    match recv(&mut mario_reader).await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // Anna si registra
    let (mut anna_reader, mut anna_writer) = connect(addr).await;
    send(
        &mut anna_writer,
        &ClientMessage::Register {
            username: "anna".to_string(),
            password: "password_anna".to_string(),
        },
    )
    .await;
    match recv(&mut anna_reader).await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // Simuliamo l'operatore che manda un messaggio privato solo a mario,
    // prendendo direttamente il suo canale da `state.connections`
    let messaggio_per_mario = ServerMessage::DirectMessage {
        message: "Messaggio privato per mario".to_string(),
        timestamp: Utc::now(),
    };
    {
        let connections = state.connections.read().await;
        let tx = connections
            .get("mario")
            .expect("mario deve essere connesso");
        tx.send(messaggio_per_mario)
            .expect("invio al canale di mario deve riuscire");
    }

    // Mario deve ricevere il messaggio
    match recv(&mut mario_reader).await {
        ServerMessage::DirectMessage { message, .. } => {
            assert_eq!(message, "Messaggio privato per mario");
        }
        other => panic!("atteso DirectMessage, arrivato {other:?}"),
    }

    // Anna non deve ricevere nulla: aspettiamo poco tempo, se non arriva
    // nulla il timeout scade (Err) ed è quello che ci aspettiamo
    let esito = tokio::time::timeout(
        std::time::Duration::from_millis(200),
        recv(&mut anna_reader),
    )
    .await;
    assert!(
        esito.is_err(),
        "anna non doveva ricevere nulla, invece ha ricevuto qualcosa"
    );
}

#[tokio::test]
async fn un_broadcast_arriva_a_tutti_gli_utenti_connessi() {
    let (addr, state, _chat_rx) = spawn_test_server().await;

    let usernames = ["mario", "anna", "luigi"];
    let mut clients = Vec::new();
    for username in usernames {
        let (mut reader, mut writer) = connect(addr).await;
        send(
            &mut writer,
            &ClientMessage::Register {
                username: username.to_string(),
                password: format!("password_{username}"),
            },
        )
        .await;
        match recv(&mut reader).await {
            ServerMessage::AuthResult { success, .. } => assert!(success),
            other => panic!("atteso AuthResult per {username}, arrivato {other:?}"),
        }
        clients.push((username, reader, writer));
    }

    // Simuliamo l'operatore che manda un broadcast a tutti i connessi
    let testo = "Messaggio per tutti".to_string();
    {
        let connections = state.connections.read().await;
        for tx in connections.values() {
            let msg = ServerMessage::BroadcastMessage {
                message: testo.clone(),
                timestamp: Utc::now(),
            };
            tx.send(msg).expect("invio broadcast deve riuscire");
        }
    }

    // Ogni utente deve ricevere lo stesso messaggio broadcast
    for (username, mut reader, _writer) in clients {
        match recv(&mut reader).await {
            ServerMessage::BroadcastMessage { message, .. } => {
                assert_eq!(
                    message, testo,
                    "l'utente {username} non ha ricevuto il broadcast corretto"
                );
            }
            other => panic!("atteso BroadcastMessage per {username}, arrivato {other:?}"),
        }
    }
}

#[tokio::test]
async fn chat_message_da_non_autenticato_non_produce_risposta() {
    let (addr, _state, _chat_rx) = spawn_test_server().await;

    let (mut reader, mut writer) = connect(addr).await;
    // Nessun Login/Register prima di questo: la connessione non è autenticata.
    send(
        &mut writer,
        &ClientMessage::ChatMessage {
            message: "ciao".to_string(),
            timestamp: Utc::now(),
        },
    )
    .await;

    // Non deve arrivare nessuna risposta (né errore né altro): aspettiamo
    // poco e ci aspettiamo che scada il timeout.
    let esito =
        tokio::time::timeout(std::time::Duration::from_millis(200), recv(&mut reader)).await;
    assert!(
        esito.is_err(),
        "un client non autenticato non doveva ricevere nulla per un ChatMessage"
    );
}

#[tokio::test]
async fn chat_message_autenticato_arriva_alloperatore() {
    let (addr, _state, mut chat_rx) = spawn_test_server().await;

    let (mut reader, mut writer) = connect(addr).await;
    send(
        &mut writer,
        &ClientMessage::Register {
            username: "mario".to_string(),
            password: "password_mario".to_string(),
        },
    )
    .await;
    match recv(&mut reader).await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    send(
        &mut writer,
        &ClientMessage::ChatMessage {
            message: "ciao operatore".to_string(),
            timestamp: Utc::now(),
        },
    )
    .await;

    let incoming = chat_rx
        .recv()
        .await
        .expect("l'operatore doveva ricevere il messaggio in chat");
    assert_eq!(incoming.from_username, "mario");
    assert_eq!(incoming.message, "ciao operatore");
}

#[tokio::test]
async fn utente_disconnesso_sparisce_dalla_mappa_delle_connessioni() {
    let (addr, state, _chat_rx) = spawn_test_server().await;

    let (reader, mut writer) = connect(addr).await;
    send(
        &mut writer,
        &ClientMessage::Register {
            username: "mario".to_string(),
            password: "password_mario".to_string(),
        },
    )
    .await;
    let mut reader = reader;
    match recv(&mut reader).await {
        ServerMessage::AuthResult { success, .. } => assert!(success),
        other => panic!("atteso AuthResult, arrivato {other:?}"),
    }

    // A questo punto mario deve essere nella mappa
    assert!(state.connections.read().await.contains_key("mario"));

    // Chiudiamo la connessione per davvero, droppando entrambe le metà
    drop(reader);
    drop(writer);

    // Diamo tempo al server di accorgersi della disconnessione e ripulire
    let mut tentativi_rimasti = 20;
    loop {
        if !state.connections.read().await.contains_key("mario") {
            break;
        }
        tentativi_rimasti -= 1;
        assert!(
            tentativi_rimasti > 0,
            "mario doveva sparire da state.connections dopo la disconnessione"
        );
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

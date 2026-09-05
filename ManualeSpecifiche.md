# Manuale Specifiche

## Architettura generale

GeoRuggine è organizzato come workspace Cargo con tre crate: `common`, `client` e `server`.

- **`common`**: non ha dipendenze dagli altri due crate ed è importato sia da `client` sia da `server`. Contiene esclusivamente i tipi condivisi (modello dati e protocollo di comunicazione), così che entrambe le parti manipolino le stesse strutture dati e non possano disallinearsi nella serializzazione.
- **`client`**: applicazione che si connette al server via TCP, gestisce autenticazione, simulazione del movimento e interfaccia testuale (TUI). Dipende da `common` per i messaggi scambiati con il server.
- **`server`**: applicazione che accetta connessioni TCP dai client, gestisce autenticazione (hashing delle password con Argon2) e persistenza su database SQLite (`sqlx`), calcola le statistiche di movimento a partire dai `TrackPoint` ricevuti e offre all'operatore un'interfaccia testuale (TUI) per inviare messaggi broadcast/unicast. Dipende da `common` per il modello dati e il protocollo di comunicazione con il client.

## 1. Common

`common` espone due moduli ([lib.rs](georuggine/common/src/lib.rs)):

- **`models`** ([models.rs](georuggine/common/src/models.rs)): entità — `User`, `Position`, `PositionWithoutTimestamp`, `TrackPoint`, `MovementStats`. Sono strutture dati "passive", senza logica, condivise sia per la comunicazione di rete sia per la persistenza.
- **`protocol`** ([protocol.rs](georuggine/common/src/protocol.rs)): definisce i messaggi scambiabili tra client e server come due enum serializzabili in JSON tramite `serde`:
  - `ClientMessage`: `Register`, `Login`, `PositionUpdate`, `ChatMessage`, `QueryStats`;
  - `ServerMessage`: `AuthResult`, `BroadcastMessage`, `DirectMessage`, `StatsResult`, `Error` (con `ErrorContext` per distinguere errori di statistiche, chat o generali).

Il fatto che entrambi i lati importino le stesse enum garantisce che client e server restino sincronizzati: un cambiamento al protocollo si riflette a livello di tipo su entrambi i crate.

## 2. Client

Il client comunica con il server su una connessione TCP (`127.0.0.1:8080` di default), scambiando istanze di `ClientMessage`/`ServerMessage` serializzate in JSON. Tutta l'orchestrazione avviene in [main.rs](georuggine/client/src/main.rs), che resta il task asincrono principale (non viene mai spawnato con `tokio::spawn`): è lui a creare i canali, avviare gli altri task e attendere in sequenza autenticazione e UI.

#### 2.1 Sequenza di avvio (in `main.rs`)

1. **Lettura del percorso da simulare** (sincrona, prima ancora di connettersi): [tools/movement_file_picker.rs](georuggine/client/src/tools/movement_file_picker.rs) sceglie a rotazione uno dei tre CSV in `movement_data/` tramite un indice salvato in `client/.movement_index`, poi [tools/read_movement_data.rs](georuggine/client/src/tools/read_movement_data.rs) lo deserializza in `Vec<PositionWithoutTimestamp>` (coppie lat/lon, senza timestamp).
2. **Connessione TCP**: il client prova a connettersi al socket del server con `TcpStream::connect("127.0.0.1:8080")`; se il tentativo fallisce, il client stampa l'errore e termina subito (nessun task è ancora stato avviato).
3. **Split dello stream**: `stream.into_split()` separa `OwnedReadHalf` e `OwnedWriteHalf`, così lettura e scrittura possono procedere su task diversi senza contendersi lo stream.
4. **Creazione dei canali**: vengono creati due canali `mpsc` con capacità 100 per la comunicazione dei messaggi tra client e server — `client_msg_tx`/`client_msg_rx` e `server_msg_tx`/`server_msg_rx`.
5. **`tokio::spawn` del task `writer`**: viene cretao un loop in `main.rs` che consuma `client_msg_rx` e per ogni messaggio chiama [`messaging::send_message`](georuggine/client/src/messaging.rs) sulla metà in scrittura.
6. **`tokio::spawn` del task `listener`**: esegue [`listener::listen(reader, server_msg_tx)`](georuggine/client/src/listener.rs), che legge righe dalla metà in lettura con `read_line`, deserializza ogni riga non vuota in `ServerMessage` e la inoltra su `server_msg_tx`. Un `Ok(0)` (server che chiude la connessione) o un errore di rete fanno terminare il task, che droppa `server_msg_tx`: il conseguente `None` da `server_msg_rx.recv()` è il segnale — per auth e per la UI — che la connessione è caduta.
7. **Autenticazione**: viene eseguita nel task principale: [`auth::authenticate(&client_msg_tx, &mut server_msg_rx)`](georuggine/client/src/auth.rs) è un wrapper che delega a `ui::auth_ui::run`, la quale invia `Register`/`Login` su `client_msg_tx` e attende il corrispondente `AuthResult` da `server_msg_rx`. Ritorna `Option<String>`: `None` se l'utente esce prima di autenticarsi con successo — in tal caso il `listener` viene abortito, `client_msg_tx` viene droppato e si attende la chiusura del `writer` prima di terminare il processo.
8. **Se l'autenticazione riesce**: viene creato un canale `watch::channel(MovementStatus::default())` (`movement_status_tx`/`movement_status_rx`) e viene fatto `tokio::spawn` di [`movement_sim(positions, client_msg_tx.clone(), movement_status_tx)`](georuggine/client/src/movement_sim.rs) che riceve i dati **già letti** al passo 1 e un **clone** di `client_msg_tx`, condividendo così lo stesso canale verso il `writer` usato dalla UI.
9. **Schermata principale**: ancora nel task principale viene chiamata la funzione che mostra la ui [`ui::main_ui::run(username, &client_msg_tx, &mut server_msg_rx, &mut movement_status_rx)`](georuggine/client/src/ui/main_ui.rs).
10. **Chiusura**: al ritorno da `main_ui::run` (es. `Esc`), `movement_handle.abort()` ferma la simulazione e `drop(client_msg_tx)` chiude il lato invio del canale verso il `writer`; quest'ultimo esce dal proprio loop non appena tutti i sender sono stati droppati e il canale si svuota, e viene atteso (`writer_handle.await`) prima che il processo termini. Il task `listener` non viene atteso esplicitamente in questo percorso: termina da solo quando il processo esce, o prima se il server chiude per primo la connessione.

#### 2.2 Topologia: task e canali

Dopo il passo 8, sono attivi tre task in background più il task principale che in quel momento sta eseguendo `ui::main_ui::run`:

```
Invio messaggi verso il server
------------------------------------------------------------------------------------------------------------------------
┌───────────────────────────┐   mpsc<ClientMessage>
│ main_ui (task principale) │ ──────────┐
│  invia ChatMessage /      │           │                           ┌─────────────┐
│  QueryStats               │           ├──--client_msg_tx/rx ─────▶│ writer task │
└───────────────────────────┘           │                           │ messaging:: │───▶  TCP server
┌───────────────────────────┐           │                           │ send_message│
│ movement_sim (task)       │ ──────────┘                           └─────────────┘
│  invia PositionUpdate     │
│  ogni ~30s                │
└───────────────────────────┘

Ricezione messaggi dal server
------------------------------------------------------------------------------------------------------------------------
                          TCP        ┌──────────────────┐   mpsc<ServerMessage>   ┌───────────────────────────┐
              server ───────────────▶│ listener task    │ ──────────────────────▶ │ main_ui (task principale) │
                                     │ listener::listen │                         │  DirectMessage/Broadcast/ │
                                     └──────────────────┘                         │  StatsResult/Error        │
                                                                                  └───────────────────────────┘

Stato locale della simulazione
------------------------------------------------------------------------------------------------------------------------
┌───────────────────────────────┐  watch<MovementStatus>  ┌───────────────────────────┐
│ movement_sim (task)           │ ───────────────────────▶│ main_ui (task principale) │
│ pubblica stato corrente       │                         │ mostra riquadro "Stato    │
│ (InMovimento/Fermo/Problema)  │                         │  movimento"               │
└───────────────────────────────┘                         └───────────────────────────┘
```

All'interno di `ui::main_ui::run` questi tre flussi in entrata convergono in un unico `tokio::select!` (si veda [main_ui.rs](georuggine/client/src/ui/main_ui.rs)), che ad ogni iterazione attende contemporaneamente: un evento da tastiera (`EventStream` di crossterm), un messaggio su `server_msg_rx`, o un aggiornamento su `movement_status_rx`; il primo che si presenta viene gestito e lo schermo viene ridisegnato.

Avere un solo task dedicato alla scrittura e uno alla lettura del socket evita accessi concorrenti allo stream TCP; il disaccoppiamento tramite canali permette a UI, autenticazione e simulazione di produrre/consumare messaggi senza conoscersi direttamente né bloccarsi a vicenda, e senza bisogno di stato condiviso protetto da lock.

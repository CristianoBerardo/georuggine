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

Il client comunica con il server su una connessione TCP (`127.0.0.1:8080` di default), scambiando istanze di `ClientMessage`/`ServerMessage` serializzate in JSON. Tutta l'orchestrazione avviene in [main.rs](georuggine/client/src/main.rs), che resta il task asincrono principale: è lui a creare i canali, avviare gli altri task e attendere in sequenza autenticazione e UI.

### 2.1 Sequenza di avvio (in `main.rs`)

**Fase A — Preparazione dei dati (sincrona, prima di aprire la rete)**

1. **Scelta del percorso da simulare**: [`tools::movement_file_picker::next_movement_file`](georuggine/client/src/tools/movement_file_picker.rs) legge da `client/.movement_index` l'indice dell'ultimo file usato, seleziona a rotazione il CSV successivo tra i tre disponibili in `movement_data/` (`torino-asti.csv`, `terni-basilicata.csv`, `dati-prof.csv`) e riscrive l'indice incrementato sul file, cosicché esecuzioni successive del client propongano un percorso diverso a rotazione.
2. **Lettura del CSV**: [`tools::read_movement_data::read_movement_data`](georuggine/client/src/tools/read_movement_data.rs) apre il file scelto, deserializza ogni riga (colonne `lat`, `lon`) e produce un `Vec<PositionWithoutTimestamp>` — coordinate senza timestamp, che verrà assegnato in seguito, punto per punto, dal task di simulazione. Un errore di lettura/parsing qui è fatale e interrompe l'avvio prima ancora di provare la connessione.

**Fase B — Apertura della connessione TCP**

1. **Connessione**: `TcpStream::connect("127.0.0.1:8080")`. In caso di errore (server non raggiungibile) il client stampa il messaggio e ritorna subito da `main`: non è stato ancora creato alcun canale né alcun task, quindi non serve alcuna operazione di pulizia.
2. **Split dello stream**: `stream.into_split()` separa la connessione in `OwnedReadHalf` (avvolto poi in un `BufReader`) e `OwnedWriteHalf`, così che lettura e scrittura sul socket possano avvenire in task diversi senza mai contendersi lo stream né richiedere un lock.

**Fase C — Creazione dei canali e dei task di I/O**

1. **Canali**: vengono creati due `mpsc::channel` con capacità 100 — `client_msg_tx`/`client_msg_rx` di tipo `Sender<ClientMessage>`/`Receiver<ClientMessage>` e `server_msg_tx`/`server_msg_rx` di tipo `Sender<ServerMessage>`/`Receiver<ServerMessage>`.
2. **`tokio::spawn` del task `writer`**: è un loop, definito in `main.rs`, che fa `while let Some(msg) = client_msg_rx.recv().await` e per ogni messaggio invoca [`messaging::send_message(&mut writer, &msg)`](georuggine/client/src/messaging.rs) sulla metà in scrittura; quest'ultima funzione serializza il messaggio in JSON con `serde_json`, aggiunge il carattere di terminazione `\n` e scrive con `write_all` seguito da `flush`. Un errore di invio stampa un messaggio diagnostico e interrompe il loop (quindi il task).
3. **`tokio::spawn` del task `listener`**: esegue [`listener::listen(reader, server_msg_tx)`](georuggine/client/src/listener.rs), un loop che legge una riga alla volta con `read_line`, ignora le righe vuote e deserializza le altre in `ServerMessage`; un messaggio non deserializzabile viene scartato silenziosamente (non dovrebbe accadere, dato che client e server condividono lo stesso `common::protocol`). Il task termina quando `read_line` ritorna `Ok(0)` (il server ha chiuso la connessione) o un errore di I/O; in entrambi i casi `server_msg_tx` viene droppato, e il successivo `None` restituito da `server_msg_rx.recv()` è il modo in cui chi è in attesa (prima `auth`, poi la UI) scopre che la connessione è caduta.

**Fase D — Autenticazione**

1. [`auth::authenticate(&client_msg_tx, &mut server_msg_rx)`](georuggine/client/src/auth.rs) è un wrapper che delega interamente a [`ui::auth_ui::run`](georuggine/client/src/ui/auth_ui.rs) le operazioni di autenticazione: quest'ultima disegna la schermata di scelta Login/Registrazione, invia `ClientMessage::Login`/`Register` su `client_msg_tx` in risposta ai tasti dell'utente e attende l'`AuthResult` corrispondente da `server_msg_rx` dentro un `tokio::select!` che ascolta contemporaneamente tastiera e rete. In caso di registrazione riuscita torna alla schermata di login; in caso di login riuscito ritorna `Ok(Some(username))`.
2. **Uscita anticipata**: se `authenticate` ritorna `Ok(None)` (l'utente ha annullato con `Esc`, oppure canale rotto/connessione persa), `main` stampa un messaggio, esegue `listener_handle.abort()`, droppa `client_msg_tx` (facendo terminare il loop del `writer` non appena il canale si svuota) e attende `writer_handle.await` prima di ritornare `Ok(())`: in questo ramo il client non arriva mai a spawnare la simulazione del movimento né la UI principale.

**Fase E — Avvio della simulazione e della schermata principale**

1.  **Canale di stato**: viene creato un `watch::channel(MovementStatus::default())` (`movement_status_tx`/`movement_status_rx`); a differenza degli `mpsc` visti sopra, un canale `watch` conserva solo l'ultimo valore pubblicato, adatto a uno "stato corrente" letto a intervalli dalla UI.
2.  **`tokio::spawn` di `movement_sim`**: [`movement_sim(positions, client_msg_tx.clone(), movement_status_tx)`](georuggine/client/src/movement_sim.rs) riceve i dati **già letti** in Fase A (non accede lui stesso al filesystem) e un **clone** di `client_msg_tx`: scrive quindi sullo stesso canale verso il `writer` usato dall'autenticazione/UI, motivo per cui il `writer` deve restare in ascolto finché _tutti_ i mittenti (UI e simulazione) non hanno droppato la propria copia del sender.
3.  **Schermata principale**: sempre nel task principale, [`ui::main_ui::run(username, &client_msg_tx, &mut server_msg_rx, &mut movement_status_rx)`](georuggine/client/src/ui/main_ui.rs) prende possesso degli stessi `client_msg_tx`/`server_msg_rx` usati per l'autenticazione (passaggio di proprietà: prima li usa `auth_ui`, poi `main_ui`) più il nuovo `movement_status_rx`.

**Fase F — Chiusura**

Al ritorno da `main_ui::run` (uscita con `Esc` o connessione persa), `main` esegue in ordine: `movement_handle.abort()` (ferma subito la simulazione, anche a metà di uno `sleep`), poi `drop(client_msg_tx)` (chiude il lato invio del canale verso il `writer`) e infine `writer_handle.await` (attende che il `writer` esca dal proprio loop, cosa che accade non appena tutti i sender sono stati droppati e il canale si è svuotato). Il task `listener` **non** viene atteso né abortito esplicitamente in questo ramo: termina da sé quando il processo esce, oppure prima se è il server a chiudere per primo la connessione.

### 2.2 Topologia: task e canali

Nell'aapplicazione funzionante sono dunque attivi tre task in background più il task principale che in quel momento sta eseguendo `ui::main_ui::run`:

```
Invio messaggi verso il server
------------------------------------------------------------------------------------------------------------------------
┌───────────────────────────┐   mpsc<ClientMessage>
│ main_ui (task principale) │ ──────────┐                           ┌─────────────┐
│  invia ChatMessage /      │           │                           │ writer task │
│  QueryStats               │           ├──--client_msg_tx/rx ─────▶│ messaging:: │───▶  TCP server
└───────────────────────────┘           │                           │ send_message│
┌───────────────────────────┐           │                           └─────────────┘
│ movement_sim (task)       │ ──────────┘
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

### 2.3 Mappa dei file

Indice di riferimento rapido ai file del client.

| File                                                                                   | Ruolo                                                                                                                                                                                                       |
| -------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`main.rs`](georuggine/client/src/main.rs)                                             | Punto di ingresso: legge il CSV, apre la connessione, crea i canali, avvia `writer`/`listener`/`movement_sim`, esegue in sequenza autenticazione e UI principale, gestisce la chiusura ordinata (Fasi A–F). |
| [`auth.rs`](georuggine/client/src/auth.rs)                                             | Wrapper: inoltra `client_msg_tx`/`server_msg_rx` a `ui::auth_ui::run`.                                                                                                                                      |
| [`listener.rs`](georuggine/client/src/listener.rs)                                     | Task di lettura: legge righe dal socket, deserializza `ServerMessage`, le inoltra su un canale `mpsc`.                                                                                                      |
| [`messaging.rs`](georuggine/client/src/messaging.rs)                                   | Funzione di invio: serializza un `ClientMessage` in JSON e lo scrive sul socket (usata dal loop `writer` in `main.rs`).                                                                                     |
| [`movement_sim.rs`](georuggine/client/src/movement_sim.rs)                             | Simulazione del movimento: ogni ~30s invia un `PositionUpdate` con la posizione successiva della rotta e pubblica lo stato corrente (`MovementState::InMovimento`/`Fermo`/`Problema`) su un canale `watch`. |
| [`tools/mod.rs`](georuggine/client/src/tools/mod.rs)                                   | Dichiara i sottomoduli `movement_file_picker` e `read_movement_data`.                                                                                                                                       |
| [`tools/movement_file_picker.rs`](georuggine/client/src/tools/movement_file_picker.rs) | Seleziona a rotazione uno dei CSV in `movement_data/`, persistendo l'indice in `client/.movement_index`.                                                                                                    |
| [`tools/read_movement_data.rs`](georuggine/client/src/tools/read_movement_data.rs)     | Deserializza un CSV di coordinate in `Vec<PositionWithoutTimestamp>`.                                                                                                                                       |
| [`ui/mod.rs`](georuggine/client/src/ui/mod.rs)                                         | Dichiara i sottomoduli `auth_ui`, `main_ui`, `terminal_guard`.                                                                                                                                              |
| [`ui/terminal_guard.rs`](georuggine/client/src/ui/terminal_guard.rs)                   | Guardia RAII: il suo `Drop` chiama `ratatui::restore()`, così il terminale torna allo stato normale anche se si esce da un ramo di errore o per panic.                                                      |
| [`ui/auth_ui.rs`](georuggine/client/src/ui/auth_ui.rs)                                 | Loop della schermata di login/registrazione: `tokio::select!` tra eventi tastiera e `AuthResult` dal server.                                                                                                |
| [`ui/auth_ui/state.rs`](georuggine/client/src/ui/auth_ui/state.rs)                     | Stato del form (`App`): schermata corrente (`Screen`), campo con il focus (`Focus`), valori dei campi, messaggi di errore/informazione.                                                                     |
| [`ui/auth_ui/input.rs`](georuggine/client/src/ui/auth_ui/input.rs)                     | Traduce gli eventi tastiera in azioni (`Outbound::SendLogin`/`SendRegister`/`Cancel`/`None`) aggiornando lo stato del form.                                                                                 |
| [`ui/auth_ui/draw.rs`](georuggine/client/src/ui/auth_ui/draw.rs)                       | Rendering `ratatui` della schermata di login/registrazione a partire dallo stato `App`.                                                                                                                     |
| [`ui/main_ui.rs`](georuggine/client/src/ui/main_ui.rs)                                 | Loop della schermata principale: `tokio::select!` tra eventi tastiera, `server_msg_rx` e `movement_status_rx` (si veda §2.2).                                                                               |
| [`ui/main_ui/state.rs`](georuggine/client/src/ui/main_ui/state.rs)                     | Stato della schermata principale (`App`): riquadro con il focus (`Panel`), log di chat/broadcast, periodo statistiche selezionato, ultimo risultato statistiche, stato movimento corrente.                  |
| [`ui/main_ui/input.rs`](georuggine/client/src/ui/main_ui/input.rs)                     | Traduce gli eventi tastiera in azioni (`Outbound::SendChat`/`QueryStats`/`Quit`/`None`) aggiornando lo stato dei riquadri.                                                                                  |
| [`ui/main_ui/draw.rs`](georuggine/client/src/ui/main_ui/draw.rs)                       | Rendering `ratatui` di tutti i riquadri della schermata principale a partire dallo stato `App`.                                                                                                             |

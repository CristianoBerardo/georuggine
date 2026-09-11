# Manuale Specifiche

## Architettura generale

GeoRuggine è organizzato come workspace Cargo con tre crate: `common`, `client` e `server`.

- **`common`**: non ha dipendenze dagli altri due crate ed è importato sia da `client` sia da `server`. Contiene esclusivamente i tipi condivisi (modello dati e protocollo di comunicazione), così che entrambe le parti manipolino le stesse strutture dati e non possano disallinearsi nella serializzazione.
- **`client`**: applicazione che si connette al server via TCP, gestisce autenticazione, simulazione del movimento e interfaccia testuale (TUI). Dipende da `common` per i messaggi scambiati con il server.
- **`server`**: applicazione che accetta connessioni TCP dai client, gestisce autenticazione (hashing delle password con Argon2) e persistenza su database SQLite (`sqlx`), calcola le statistiche di movimento a partire dai `TrackPoint` ricevuti e offre all'operatore un'interfaccia testuale (TUI) per inviare messaggi broadcast/unicast. Dipende da `common` per il modello dati e il protocollo di comunicazione con il client.

## 1. Common

`common` espone due moduli ([lib.rs](common/src/lib.rs)):

- **`models`** ([models.rs](common/src/models.rs)): entità — `User`, `Position`, `PositionWithoutTimestamp`, `TrackPoint`, `MovementStats`. Sono strutture dati "passive", senza logica, condivise sia per la comunicazione di rete sia per la persistenza.
- **`protocol`** ([protocol.rs](common/src/protocol.rs)): definisce i messaggi scambiabili tra client e server come due enum serializzabili in JSON tramite `serde`:
  - `ClientMessage`: `Register`, `Login`, `PositionUpdate`, `ChatMessage`, `DeleteAccount`;
  - `ServerMessage`: `AuthResult`, `BroadcastMessage`, `DirectMessage`, `Error` (con `ErrorContext` per distinguere errori di chat o generali), `AccountDeleted` (esito, con eventuale motivo di fallimento, della richiesta `DeleteAccount`).

  Lo stesso modulo definisce anche `AuthAction` (`Login`/`Register`, usato per distinguere le due varianti di autenticazione) e `TimePeriod` (`Today`/`ThisWeek`/`ThisMonth`, usato lato server per calcolare le statistiche di movimento su intervalli programmabili).

Il fatto che entrambi i lati importino le stesse enum garantisce che client e server restino sincronizzati: un cambiamento al protocollo si riflette a livello di tipo su entrambi i crate.

## 2. Client

### 2.1 Introduzione

Il client GeoRuggine simula un dispositivo installato su un veicolo che si vuole monitorare: si autentica presso il server, invia periodicamente la propria posizione e permette all'utente di comunicare con l'amministratore e di gestire il proprio account, tutto tramite un'interfaccia testuale (TUI).

La comunicazione con il server avviene su una connessione `TCP` verso l'indirizzo letto dalla variabile d'ambiente `GEORUGGINE_SERVER_ADDR`, oppure `127.0.0.1:8080` se la variabile non è impostata, scambiando messaggi JSON — le stesse `ClientMessage`/`ServerMessage` definite in `common::protocol` (1) — così che client e server restino sempre sincronizzati sullo stesso "vocabolario". Per non bloccare mai l'interfaccia mentre si aspetta la rete, né viceversa bloccare la rete mentre si aspetta un tasto, il client si appoggia alla programmazione asincrona di `Tokio`, distribuendo lettura del socket, scrittura del socket, simulazione del movimento e interfaccia utente su task separati che comunicano tra loro tramite canali, invece che con stato condiviso e lock. L'interfaccia stessa è realizzata con `Ratatui`, che si occupa di disegnare i riquadri e di consegnare gli eventi da tastiera in modo asincrono, nascondendo la complessità della gestione diretta del terminale.

> Per "movimento" si intende qui una simulazione: il client non legge da un vero sensore GPS, ma da un file CSV di coordinate già pronto, e le invia al server esattamente come farebbe un dispositivo reale. In questo caso i file CSV disponibili sono 3 e vengono scelti a rotazione ogni volta che un nuovo client si collega.

### 2.2 Sequenza di avvio (main.rs)

All'avvio, in [main.rs](client/src/main.rs), il client attraversa questi passaggi. Solo i passi 2-9 sono racchiusi in un `loop` che riparte da capo (dal punto 2) se l'utente elimina il proprio account, invece di terminare il processo; il passo 1 viene invece eseguito una sola volta, prima del loop:

1. **Lettura dei dati di movimento**, prima ancora di aprire la rete: [`tools::movement_file_picker::next_movement_file`](client/src/tools/movement_file_picker.rs) sceglie a rotazione uno dei tre CSV disponibili in `movement_data/`, leggendo e riscrivendo un indice su `client/.movement_index` così che ogni esecuzione proponga un percorso diverso; [`tools::read_movement_data::read_movement_data`](client/src/tools/read_movement_data.rs) lo apre e produce le coordinate (senza timestamp, che verrà assegnato più avanti, punto per punto, dalla simulazione). Un errore qui è fatale: il client si ferma prima ancora di provare a collegarsi al server.
2. **Connessione TCP** (`TcpStream::connect`) all'indirizzo determinato leggendo `GEORUGGINE_SERVER_ADDR` (o `127.0.0.1:8080` se non impostata, si veda 2.1). Se il server non è raggiungibile, il client stampa l'errore e termina subito: non è stato ancora creato nessun canale né task, quindi non c'è nulla da ripulire.
3. **Divisione dello stream** (`stream.into_split()`) in una metà di lettura e una di scrittura, così che i due task descritti nei punti successivi possano lavorare in parallelo sullo stesso socket senza mai contendersi l'accesso né richiedere un lock.
4. **Creazione dei canali**: due `mpsc::channel` con capacità 100, uno per i messaggi in uscita verso il server (`ClientMessage`) e uno per quelli in arrivo (`ServerMessage`).
5. **Avvio del task `writer`**: legge dal canale in uscita e, per ogni messaggio, chiama [`messaging::send_message`](client/src/messaging.rs), che lo serializza in JSON e lo scrive sul socket.
6. **Avvio del task `listener`**: esegue [`listener::listen`](client/src/listener.rs), che legge il socket riga per riga, deserializza ogni riga in un `ServerMessage` e lo inoltra sul canale in arrivo; termina quando la connessione si chiude, e con esso il canale — è così che chi aspetta un messaggio (prima l'autenticazione, poi la UI principale) si accorge che la connessione è caduta.
7. **Autenticazione**: [`auth::authenticate`](client/src/auth.rs) delega alla schermata di login/registrazione ([`ui::auth_ui::run`](client/src/ui/auth_ui.rs)), che scambia `Login`/`Register`/`AuthResult` con il server finché l'utente non ottiene l'accesso o annulla con Esc. Se annulla (o la connessione cade prima di autenticarsi), il client si ferma qui: chiude i task già avviati e termina, senza mai arrivare ad avviare la simulazione o la schermata principale.
8. **Avvio della simulazione e della schermata principale**: viene creato un canale `watch` per lo stato del movimento (conserva solo l'ultimo valore pubblicato, adatto a uno stato letto a intervalli), spawnato il task [`movement_sim`](client/src/movement_sim.rs) (che invia una `PositionUpdate` ogni ~30 secondi sullo stesso canale usato dal `writer`), e infine avviata la schermata principale ([`ui::main_ui::run`](client/src/ui/main_ui.rs), si veda 2.3).
9. **Chiusura o ripetizione**: quando la schermata principale termina, restituisce il motivo (`ExitReason`: `UserQuit`, `ConnectionLost` o `AccountDeleted`). In ogni caso il client ferma la simulazione e il `listener`, e aspetta che il `writer` finisca di svuotarsi. Se il motivo è `AccountDeleted`, il `loop` riparte dal punto 2 (nuova connessione, nuova autenticazione); altrimenti il processo termina, stampando un messaggio coerente con il motivo.

### 2.3 Ciclo principale (ui/main_ui.rs)

Una volta autenticato, il cuore dell'applicazione è il loop di [`ui::main_ui::run`](client/src/ui/main_ui.rs): ad ogni iterazione ridisegna la schermata, poi si mette in attesa — tramite `tokio::select!`, che permette di reagire al primo evento pronto tra più sorgenti asincrone senza bisogno di un ciclo di polling attivo — di uno tra:

- un **evento da tastiera** (`EventStream` di `crossterm`);
- un **messaggio dal server** (`server_msg_rx`, alimentato dal task `listener`);
- un **aggiornamento dello stato del movimento** (`movement_status_rx`, alimentato dal task `movement_sim`);
- un **errore locale** (`client_error_rx`, dai task `writer` e `movement_sim`).

Un evento da tastiera viene tradotto da `handle_key` (in [`input.rs`](client/src/ui/main_ui/input.rs)) in un `Outbound`, che indica cosa fare: `SendChat` (manda un `ChatMessage`), `DeleteAccount` (manda un `ClientMessage::DeleteAccount`), `Quit` (esce), oppure `None` (l'evento ha già aggiornato lo stato interno e non richide altro — es. lo spostamento tra riquadri con Tab). Un messaggio dal server aggiorna invece direttamente lo stato dell'`App`: `DirectMessage`/`BroadcastMessage` finiscono nei rispettivi log, `AccountDeleted` fa terminare il loop (in caso di successo) o mostra l'errore nel riquadro "Elimina account" (in caso di fallimento), `Error` finisce nel log Errori con un'etichetta di sistema.

Gestito uno di questi rami, il loop riparte da capo: ridisegna e torna in attesa.

### 2.4 Topologia: task e canali

Nell'applicazione funzionante sono dunque attivi tre task in background più il task principale che in quel momento sta eseguendo `ui::main_ui::run`:

#### Invio dei messaggi verso il server

```
┌───────────────────────────┐   mpsc<ClientMessage>
│ main_ui (task principale) │ ──────────┐                           ┌─────────────┐
│  invia ChatMessage /      │           │                           │ writer task │    TCP
│  DeleteAccount            │           ├────client_msg_tx/rx ─────▶│ messaging:: │───────────▶ server
└───────────────────────────┘           │                           │ send_message│
┌───────────────────────────┐           │                           └─────────────┘
│ movement_sim (task)       │ ──────────┘
│  invia PositionUpdate     │
│  ogni ~30s                │
└───────────────────────────┘
```

#### Ricezione messaggi dal server

```
              TCP        ┌──────────────────┐   mpsc<ServerMessage>   ┌───────────────────────────┐
  server ───────────────▶│ listener task    │ ───server_msg_tx/rx───▶ │ main_ui (task principale) │
                         │ listener::listen │                         │  DirectMessage/Broadcast/ │
                         └──────────────────┘                         │  AccountDeleted/Error     │
                                                                      └───────────────────────────┘

```

#### Stato locale della simulazione

```
┌───────────────────────────────┐    watch<MovementStatus>    ┌───────────────────────────┐
│ movement_sim (task)           │ ───────────────────────────▶│ main_ui (task principale) │
│ pubblica stato corrente       │                             │ mostra riquadro "Stato    │
│ (InMovimento/Fermo/Problema)  │                             │  movimento"               │
└───────────────────────────────┘                             └───────────────────────────┘
```

#### Errori locali

```
┌───────────────────────────┐   mpsc<String> (unbounded)
│ writer task               │ ──────────┐
│  errore di invio          │           │                           ┌───────────────────────────┐
└───────────────────────────┘           ├────client_error_tx/rx────▶│ main_ui (task principale) │
┌───────────────────────────┐           │                           │  mostra nel log Errori    │
│ movement_sim (task)       │ ──────────┘                           └───────────────────────────┘
│ errore di invio posizione │
└───────────────────────────┘
```

A differenza degli altri due, questo canale ha **due produttori** (`writer` e `movement_sim`), entrambi liberi di segnalare un problema locale (es. un invio fallito) senza dover conoscere né bloccare l'altro task.

Questi flussi convergono tutti nel `tokio::select!` di `ui::main_ui::run` descritto in 2.3. Avere un solo task dedicato alla scrittura e uno alla lettura del socket evita accessi concorrenti allo stream TCP; il disaccoppiamento tramite canali permette a UI, autenticazione e simulazione di produrre/consumare messaggi senza conoscersi direttamente né bloccarsi a vicenda, e senza bisogno di stato condiviso protetto da lock.

### 2.5 Mappa dei file

Indice di riferimento rapido ai file del client.

| File                                                                        | Ruolo                                                                                                                                                                                                                                                                          |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| [`main.rs`](client/src/main.rs)                                             | Punto di ingresso: legge il CSV, poi ripete in un `loop` l'intera sequenza connessione/canali/`writer`+`listener`+`movement_sim`/autenticazione/UI principale (2.2); il `loop` riparte automaticamente se l'utente elimina il proprio account, altrimenti il processo termina. |
| [`auth.rs`](client/src/auth.rs)                                             | Wrapper: inoltra `client_msg_tx`/`server_msg_rx` a `ui::auth_ui::run`.                                                                                                                                                                                                         |
| [`listener.rs`](client/src/listener.rs)                                     | Task di lettura: legge righe dal socket, deserializza `ServerMessage`, le inoltra su un canale `mpsc`.                                                                                                                                                                         |
| [`messaging.rs`](client/src/messaging.rs)                                   | Funzione di invio: serializza un `ClientMessage` in JSON e lo scrive sul socket (usata dal loop `writer` in `main.rs`).                                                                                                                                                        |
| [`movement_sim.rs`](client/src/movement_sim.rs)                             | Simulazione del movimento: ogni ~30s invia un `PositionUpdate` con la posizione successiva della rotta e pubblica lo stato corrente (`MovementState::InMovimento`/`Fermo`/`Problema`) su un canale `watch`.                                                                    |
| [`tools/mod.rs`](client/src/tools/mod.rs)                                   | Dichiara i sottomoduli `movement_file_picker` e `read_movement_data`.                                                                                                                                                                                                          |
| [`tools/movement_file_picker.rs`](client/src/tools/movement_file_picker.rs) | Seleziona a rotazione uno dei CSV in `movement_data/`, persistendo l'indice in `client/.movement_index`.                                                                                                                                                                       |
| [`tools/read_movement_data.rs`](client/src/tools/read_movement_data.rs)     | Deserializza un CSV di coordinate in `Vec<PositionWithoutTimestamp>`.                                                                                                                                                                                                          |
| [`ui/mod.rs`](client/src/ui/mod.rs)                                         | Dichiara i sottomoduli `auth_ui`, `main_ui`, `size_control`, `terminal_guard`.                                                                                                                                                                                                 |
| [`ui/size_control.rs`](client/src/ui/size_control.rs)                       | Definisce le dimensioni minime del terminale (`MIN_WIDTH`/`MIN_HEIGHT`) e la funzione che verifica se lo spazio disponibile è sufficiente, usata da entrambe le schermate per mostrare l'avviso di terminale troppo piccolo.                                                   |
| [`ui/terminal_guard.rs`](client/src/ui/terminal_guard.rs)                   | Guardia RAII: il suo `Drop` chiama `ratatui::restore()`, così il terminale torna allo stato normale anche se si esce da un ramo di errore o per panic.                                                                                                                         |
| [`ui/auth_ui.rs`](client/src/ui/auth_ui.rs)                                 | Loop della schermata di login/registrazione: `tokio::select!` tra eventi tastiera e `AuthResult` dal server.                                                                                                                                                                   |
| [`ui/auth_ui/state.rs`](client/src/ui/auth_ui/state.rs)                     | Stato del form (`App`): schermata corrente (`Screen`), campo con il focus (`Focus`), valori dei campi, messaggi di errore/informazione.                                                                                                                                        |
| [`ui/auth_ui/input.rs`](client/src/ui/auth_ui/input.rs)                     | Traduce gli eventi tastiera in azioni (`Outbound::SendLogin`/`SendRegister`/`Cancel`/`None`) aggiornando lo stato del form.                                                                                                                                                    |
| [`ui/auth_ui/draw.rs`](client/src/ui/auth_ui/draw.rs)                       | Rendering `ratatui` della schermata di login/registrazione a partire dallo stato `App`.                                                                                                                                                                                        |
| [`ui/main_ui.rs`](client/src/ui/main_ui.rs)                                 | Loop della schermata principale: `tokio::select!` tra eventi tastiera, `server_msg_rx`, `movement_status_rx` e `client_error_rx` (si veda 2.3). Ritorna un `ExitReason` (`UserQuit` / `ConnectionLost` / `AccountDeleted`) invece di `()`.                                     |
| [`ui/main_ui/state.rs`](client/src/ui/main_ui/state.rs)                     | Stato della schermata principale (`App`): riquadro attivo (`Panel`, incluso `DeleteAccount`), log di chat/broadcast/errori, stato movimento corrente, stato del flusso di eliminazione account (`DeleteAccountStep` e campi correlati).                                        |
| [`ui/main_ui/input.rs`](client/src/ui/main_ui/input.rs)                     | Traduce gli eventi tastiera in azioni (`Outbound::SendChat`/`DeleteAccount`/`Quit`/`None`) aggiornando lo stato dei riquadri; gestisce anche il flusso a più passi (password + conferma) del riquadro Elimina account, intercettando `Esc` per annullare senza chiudere l'app. |
| [`ui/main_ui/draw.rs`](client/src/ui/main_ui/draw.rs)                       | Rendering `ratatui` di tutti i riquadri della schermata principale a partire dallo stato `App`.                                                                                                                                                                                |

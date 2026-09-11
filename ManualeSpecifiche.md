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

## 2. Server

### 2.1 Introduzione

Il server GeoRuggine è stato progettato per fornire un servizio di messaggistica **unicast bidirezionale** e **broadcast** verso i client e ricevere da questi ultimi dati GPS per il monitoraggio della **sosta**, del **movimento**, della **disconnessione** e della **velocità**.

Tutto questo avviene tramite connessione `TCP` gestita dal server attraverso un socket di ascolto (`TcpListener`).
Più client possono connettersi simultaneamente grazie all'adozione della programmazione asincrona e al runtime `Tokio`, che gestisce le operazioni di I/O di rete in modo non bloccante e concorrente.

L'utilizzo della libreria `Ratatui` permette di avere una semplice interfaccia da terminale (TUI - Text User Interface) che nasconde la complessità della gestione degli eventi da tastiera e la suddivisione, nel medesimo terminale, in aree di visualizzazione differenti.

> Per client o utente si intende un qualsiasi dispositivo o software installato sul veicolo che si vuole monitorare. In questo progetto il movimento è stato effettuato leggendo un file con coordinate GPS.

### 2.2 Sequenza di avvio (main.rs)

La prima parte di avvio del server è una sequenza di inizializzazione che permette di configurare il server:

1. Avvio di un logger ([`start_cpu_logger`](server/src/logging/cpu_logger.rs)) per la registrazione del consumo di CPU dell'applicativo.
   La funzione di log inizia subito facendo uno spawn di un thread separato dal main. Utilizzando la libreria `sysinfo` vengono raccolte le informazioni del sistema ad intervalli di campionamento di 2 minuti.
   I risultati sono scritti nel file `cpu_metrics.log` che contiene le informazioni quali:
   - **timestamp**
   - **utilizzo CPU totale dell'app**
   - **utilizzo CPU normalizzato**
   - **consumo globale della CPU del server**

   > Sysinfo riporta il consumo della CPU per processo senza tenere conto di quanti core sono presenti nel sistema. Ad esempio con 8 core si potrebbero verificare valori di CPU fino a 800%, per questo motivo è stato normalizzato questo valore dividendo per il numero di core presenti nel sistema.

2. Connessione al database `sqlite`. La scelta di utilizzare sqlite è stata dettata dalla semplicità di utilizzo, dalla leggerezza del database stesso e anche dalla semplicità dei dati che vengono salvati nel database. La scelta di altri database come ad esempio quelli documentali, come MongoDB, è stata scartata in quanto uno schema flessibile dei dati non è necessario per il caso d'uso del progetto

3. Avvio di diversi canali di comunicazione:
   - Un canale `shutdown_tx` utilizzato per inviare a tutti i client connessi un avviso che il server sta per chiudere la connessione. Utile per dare tempo al client di predisporre eventuali azioni prima della chiusura della connessione.
   - Un canale `connections_notify` che notifica quando le connessioni al server cambiano. Questo serve per poter far in modo che il server possa aggiornare le informazioni di stato.
   - Un canale unbounded per la gestione, in particolare della TUI, dei messaggi provenienti dalle chat dei vari client connessi.
   - Un canale unbounded per la gestione degli errori provenienti da [`handle_connection`](server/src/handlers/handle_connection.rs)

4. Inizializzazione dello stato dell'applicazione con tutti gli utenti presenti nel database.
5. Creazione di un canale oneshot utilizzato internamente per comunicare quando il servizio TCP di ascolto è pronto per ricevere connessioni.
6. Spawn di thread tokio separato per la gestione delle connessioni ed ascolto di eventuali errori provenienti da `handle_connection`.
7. Una volta che la connessione TCP è pronta, e dunque il server in ascolto, viene avviata la TUI

### 2.3 Sequenza di avvio (main_ui.rs)

Questo file è il cuore della TUI.
All'inizio il flusso si presenta sequenziale per via delle varie inizializzazioni:

1. Si crea un'istanza di [`App`](server/src/ui/main_ui/state.rs) che contiene lo stato dell'applicazione. A questa viene passato lo stato creato precedentemente in [`main.rs`](server/src/main.rs) per permettere di inizializzare con i nomi degli utenti presenti nel database.
2. Inizializzazione di ratatui.
3. Gestione tramite il tratto Drop della chiusura pulita del terminale. Viene invocata automaticamente da Rust ed evita che sul terminale rimanga l'interfaccia TUI.
4. La libreria `crossterm` permette di ricevere eventi asincroni da tastiera grazie alla creazione di un `EventStream`.
5. Sottoscrizione al canale creato precedentemente in `main.rs` per la ricezione di nuove connessioni e disconnessioni dei client, permettendo una iterazione del loop e, dunque, la visualizzazione dei nuovi cambiamenti.

#### Loop principale

Come prima cosa vengono aggiornati i dati prima della visualizzazione su schermo:

1. Aggiornamento dell'indice della selezione delle statistiche da visualizzare.
2. Aggiornamento della lista degli utenti presenti nel database con il proprio stato (`Sconnesso`, `Fermo`, `InMovimento`, `Problema`) utilizzato per la visualizzazione lato server.
3. Vengono letti i nomi degli utenti connessi.
4. Caricamento delle vecchie chat degli utenti connessi al database.
5. Visualizzazione a schermo della TUI.
6. Gestione degli eventi da tastiera grazie all'utilizzo di `tokio::select!` che permette la gestione di eventi asincroni senza consumare cicli di CPU.
   - La funzione [`handle_key`](server/src/ui/main_ui/input.rs) riceve un `KeyEvent` in input e ritorna un enum `Outbound` che può indicare `Quit`, `SendChat`, `SendBroadcast` oppure `None`. Questa funzione utilizza il focus per permettere la **selezione**, la **scrittura** e l'**invio** di messaggi verso i client connessi:
     - **Esc**: ritorna un `Outbound::Quit`.
     - **Tab**: gestisce, attraverso una macchina a stati finiti, dove si trova il focus così da permettere a [`draw.rs`](server/src/ui/main_ui/draw.rs) di disegnare a schermo il bordo colorato della selezione attiva.
     - **BackTab**: uguale a Tab ma gestisce il verso inverso della selezione.

7. Una volta capito quale evento da tastiera è stato prodotto, questo viene gestito tramite un match:
   - `Outbound::Quit`: vengono inviati a tutti i client messaggi broadcast di chiusura connessione. Si invia anche un messaggio vuoto di shutdown e si prevedono 300ms per la chiusura della connessione.
   - `Outbound::SendChat`: invia al client selezionato il messaggio scritto nella casella di testo.
   - `Outbound::SendBroadcast`: invia a tutti i client connessi il messaggio scritto nella casella broadcast di testo.
   - `Outbound::QueryStats`: vengono aggiornate le statistiche dello specifico user selezionato.

> Se non fosse un evento da tastiera, come ad esempio la ricezione di un messaggio da parte di un client, il canale chat_rx verrebbe notificato e verrebbe selezionato l'utente corretto, pushato il nuovo messaggio nella chat ed aggiornato il numero di eventuali notifiche che l'amministratore non ha ancora letto.
>
> Eventuali errori provenienti dal canale `error_rx` vengono stampati direttamente nella TUI, nell'apposito box di errori.

Una volta intrapreso uno di questi 4 rami:

- **Tasto premuto** (`maybe_event = term_events.next()`)
- **Messaggio chat in arrivo** (`chat_rx.recv()`)
- **Notifica di errore** (`error_rx.recv()`)
- **Variazione dei client connessi** (`connections_rx.changed()`)

il loop riparte da capo: aggiorna i dati, ridisegna la TUI e si mette di nuovo in attesa di eventi.

### 2.4 Mappa dei file

| File                                                                                  | Ruolo                                                                                                                                                                                                                                                                                                                             |
| ------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`main.rs`](server/src/main.rs)                                                       | Entry point del server: inizializza il logger CPU, apre il database SQLite, istanzia lo stato condiviso `AppState` con i relativi canali (broadcast, watch, mpsc), avvia il listener TCP in background e avvia l'interfaccia terminale TUI.                                                                                       |
| [`network.rs`](server/src/network.rs)                                                 | Gestisce l'ascolto di rete TCP: si lega all'indirizzo configurato, notifica l'avvio tramite canale oneshot, accetta le connessioni in ingresso dai client in un loop asincrono e genera per ciascuna un task Tokio dedicato a `handle_connection`.                                                                                |
| [`state.rs`](server/src/state.rs)                                                     | Definisce lo stato globale condiviso `AppState` (pool SQLite, mappa delle connessioni attive protetta da `RwLock`, stato utenti in tempo reale e canali di notifica/shutdown) e le strutture dati correlate (`UserStatus`, `Info`, `IncomingChat`, `IncomingError`).                                                              |
| [`messaging.rs`](server/src/messaging.rs)                                             | Fornisce le primitive I/O a basso livello sul socket TCP: serializza e invia messaggi `ServerMessage` in formato JSON newline-delimited (`send_message`) e legge e deserializza a righe i `ClientMessage` ricevuti (`receive_message`).                                                                                           |
| [`auth.rs`](server/src/auth.rs)                                                       | Modulo per la sicurezza e l'hashing delle credenziali: implementa `hash_password` e `verify_password` basate su algoritmo crittografico Argon2 e sale casuale (`OsRng`), garantendo che le password non siano mai memorizzate in chiaro.                                                                                          |
| [`db.rs`](server/src/db.rs)                                                           | Layer di accesso ai dati su SQLite tramite SQLx: gestisce l'inizializzazione dello schema (`init_schema`), le operazioni CRUD sugli utenti (`insert_user`, `delete_user`, `get_all_users`) e il salvataggio/recupero della cronologia delle posizioni GPS (`insert_track_point`).                                                 |
| [`stats.rs`](server/src/stats.rs)                                                     | Calcola le metriche di movimento dalla cronologia dei punti GPS: calcola distanze con la formula di Haversine, velocità media, tempo in movimento e sosta (escludendo disconnessioni >35s) per rispondere alle richieste di statistiche dalla TUI.                                                                                |
| [`user_status.rs`](server/src/user_status.rs)                                         | Gestisce la mappa in memoria dello stato operativo degli utenti (`Sconnesso`, `Fermo`, `InMovimento`, `Problema`): aggiorna gli stati correnti, incrementa i contatori di permanenza temporale e notifica in tempo reale la TUI delle variazioni.                                                                                 |
| [`bin/script_hashing.rs`](server/src/bin/script_hashing.rs)                           | Utility CLI per generare offline gli hash Argon2 delle password, utilizzata per creare le credenziali degli utenti predefiniti da inserire nel database iniziale.                                                                                                                                                                 |
| [`handlers/handle_connection.rs`](server/src/handlers/handle_connection.rs)           | Gestisce il ciclo di vita di una singola connessione client: riceve i pacchetti TCP, li smista all'handler specifico, gestisce un watchdog timer (35s) per rilevare l'inattività (stato `Problema`) e pulisce connessioni e stati alla disconnessione o allo shutdown.                                                            |
| [`handlers/handle_delete_account.rs`](server/src/handlers/handle_delete_account.rs)   | Gestisce la cancellazione dell'account (`DeleteAccount`): verifica la password tramite hash su DB, rimuove l'utente e i suoi punti GPS da SQLite, cancella la connessione attiva e lo stato in memoria e conferma l'eliminazione al client.                                                                                       |
| [`handlers/handle_login.rs`](server/src/handlers/handle_login.rs)                     | Gestisce l'autenticazione (`Login`): verifica le credenziali con Argon2 sul database SQLite, rifiuta il login se l'utente ha già una connessione attiva da un altro dispositivo, altrimenti inserisce il sender del client nella mappa delle connessioni attive, notifica la TUI dell'avvenuto login e risponde con `AuthResult`. |
| [`handlers/handle_new_track_point.rs`](server/src/handlers/handle_new_track_point.rs) | Elabora gli aggiornamenti GPS (`PositionUpdate`): aggiorna la macchina a stati di mobilità (`InMovimento`, `Fermo` dopo 180s o recovery da `Problema`), persiste il punto nel database e aggiorna i tempi di permanenza.                                                                                                          |
| [`handlers/handle_registration.rs`](server/src/handlers/handle_registration.rs)       | Gestisce la registrazione di un nuovo utente (`Register`): genera l'hash Argon2 della password, inserisce l'utente nel DB SQLite verificando che l'username non sia duplicato, autentica la sessione e risponde con `AuthResult`.                                                                                                 |
| [`logging/cpu_logger.rs`](server/src/logging/cpu_logger.rs)                           | Monitora le prestazioni della CPU in un thread di background separato: campiona ogni 120s l'uso CPU del processo (normalizzato sui core) e globale tramite `sysinfo`, salvando i dati nel file `cpu_metrics.log`.                                                                                                                 |
| [`ui/main_ui.rs`](server/src/ui/main_ui.rs)                                           | Coordina l'event loop della TUI con `tokio::select!`: sincronizza lo stato locale da `AppState`, ridisegna l'interfaccia e gestisce concorrentemente eventi tastiera, chat in arrivo dai client, errori di rete e variazioni delle connessioni.                                                                                   |
| [`ui/main_ui/draw.rs`](server/src/ui/main_ui/draw.rs)                                 | Implementa il rendering grafico della TUI con Ratatui: disegna il layout a più pannelli (utenti, chat diretta, broadcast, statistiche di movimento, log errori e guida tasti), evidenziando il focus attivo e gestendo scrolling e wrapping del testo.                                                                            |
| [`ui/main_ui/input.rs`](server/src/ui/main_ui/input.rs)                               | Interpreta gli eventi da tastiera (`handle_key`): gestisce lo spostamento del focus tra i pannelli (Tab/BackTab), lo scorrimento dei log e converte i comandi utente nell'enum `Outbound` (`SendChat`, `SendBroadcast`, `QueryStats`, `Quit`).                                                                                    |
| [`ui/main_ui/state.rs`](server/src/ui/main_ui/state.rs)                               | Definisce i modelli e lo stato interno della TUI (`App`): memorizza i log di chat e broadcast, la cronologia errori, i buffer di input testo, il pannello correntemente selezionato (`Panel`) e lo stato del wizard delle statistiche (`StatsStep`).                                                                              |
| [`ui/size_control.rs`](server/src/ui/size_control.rs)                                 | Definisce le dimensioni minime della finestra del terminale (80 colonne x 29 righe) e ne verifica il rispetto, mostrando un messaggio di avviso se lo spazio è insufficiente per visualizzare correttamente la TUI.                                                                                                               |
| [`ui/terminal_guard.rs`](server/src/ui/terminal_guard.rs)                             | Implementa il pattern RAII per il terminale: assicura tramite il tratto `Drop` che, in caso di uscita normale, errore o panic, venga sempre ripristinato lo stato originale del terminale (`ratatui::restore()`) evitando corruzioni della shell.                                                                                 |

## 3. Client

### 3.1 Introduzione

Il client GeoRuggine simula un dispositivo installato su un veicolo che si vuole monitorare: si autentica presso il server, invia periodicamente la propria posizione e permette all'utente di comunicare con l'amministratore e di gestire il proprio account, tutto tramite un'interfaccia testuale (TUI).

La comunicazione con il server avviene su una connessione `TCP` verso l'indirizzo letto dalla variabile d'ambiente `GEORUGGINE_SERVER_ADDR`, oppure `127.0.0.1:8080` se la variabile non è impostata, scambiando messaggi JSON — le stesse `ClientMessage`/`ServerMessage` definite in `common::protocol` (1) — così che client e server restino sempre sincronizzati sullo stesso "vocabolario". Per non bloccare mai l'interfaccia mentre si aspetta la rete, né viceversa bloccare la rete mentre si aspetta un tasto, il client si appoggia alla programmazione asincrona di `Tokio`, distribuendo lettura del socket, scrittura del socket, simulazione del movimento e interfaccia utente su task separati che comunicano tra loro tramite canali, invece che con stato condiviso e lock. L'interfaccia stessa è realizzata con `Ratatui`, che si occupa di disegnare i riquadri e di consegnare gli eventi da tastiera in modo asincrono, nascondendo la complessità della gestione diretta del terminale.

> Per "movimento" si intende qui una simulazione: il client non legge da un vero sensore GPS, ma da un file CSV di coordinate già pronto, e le invia al server esattamente come farebbe un dispositivo reale. In questo caso i file CSV disponibili sono 3 e vengono scelti a rotazione ogni volta che un nuovo client si collega.

### 3.2 Sequenza di avvio (main.rs)

All'avvio, in [main.rs](client/src/main.rs), il client attraversa questi passaggi. Solo i passi 2-9 sono racchiusi in un `loop` che riparte da capo (dal punto 2) se l'utente elimina il proprio account, invece di terminare il processo; il passo 1 viene invece eseguito una sola volta, prima del loop:

1. **Lettura dei dati di movimento**, prima ancora di aprire la rete: [`tools::movement_file_picker::next_movement_file`](client/src/tools/movement_file_picker.rs) sceglie a rotazione uno dei tre CSV disponibili in `movement_data/`, leggendo e riscrivendo un indice su `client/.movement_index` così che ogni esecuzione proponga un percorso diverso; [`tools::read_movement_data::read_movement_data`](client/src/tools/read_movement_data.rs) lo apre e produce le coordinate (senza timestamp, che verrà assegnato più avanti, punto per punto, dalla simulazione). Un errore qui è fatale: il client si ferma prima ancora di provare a collegarsi al server.
2. **Connessione TCP** (`TcpStream::connect`) all'indirizzo determinato leggendo `GEORUGGINE_SERVER_ADDR` (o `127.0.0.1:8080` se non impostata, si veda 3.1). Se il server non è raggiungibile, il client stampa l'errore e termina subito: non è stato ancora creato nessun canale né task, quindi non c'è nulla da ripulire.
3. **Divisione dello stream** (`stream.into_split()`) in una metà di lettura e una di scrittura, così che i due task descritti nei punti successivi possano lavorare in parallelo sullo stesso socket senza mai contendersi l'accesso né richiedere un lock.
4. **Creazione dei canali**: due `mpsc::channel` con capacità 100, uno per i messaggi in uscita verso il server (`ClientMessage`) e uno per quelli in arrivo (`ServerMessage`).
5. **Avvio del task `writer`**: legge dal canale in uscita e, per ogni messaggio, chiama [`messaging::send_message`](client/src/messaging.rs), che lo serializza in JSON e lo scrive sul socket.
6. **Avvio del task `listener`**: esegue [`listener::listen`](client/src/listener.rs), che legge il socket riga per riga, deserializza ogni riga in un `ServerMessage` e lo inoltra sul canale in arrivo; termina quando la connessione si chiude, e con esso il canale — è così che chi aspetta un messaggio (prima l'autenticazione, poi la UI principale) si accorge che la connessione è caduta.
7. **Autenticazione**: [`auth::authenticate`](client/src/auth.rs) delega alla schermata di login/registrazione ([`ui::auth_ui::run`](client/src/ui/auth_ui.rs)), che scambia `Login`/`Register`/`AuthResult` con il server finché l'utente non ottiene l'accesso o annulla con Esc. Se annulla (o la connessione cade prima di autenticarsi), il client si ferma qui: chiude i task già avviati e termina, senza mai arrivare ad avviare la simulazione o la schermata principale.
8. **Avvio della simulazione e della schermata principale**: viene creato un canale `watch` per lo stato del movimento (conserva solo l'ultimo valore pubblicato, adatto a uno stato letto a intervalli), spawnato il task [`movement_sim`](client/src/movement_sim.rs) (che invia una `PositionUpdate` ogni ~30 secondi sullo stesso canale usato dal `writer`), e infine avviata la schermata principale ([`ui::main_ui::run`](client/src/ui/main_ui.rs), si veda 3.3).
9. **Chiusura o ripetizione**: quando la schermata principale termina, restituisce il motivo (`ExitReason`: `UserQuit`, `ConnectionLost` o `AccountDeleted`). In ogni caso il client ferma la simulazione e il `listener`, e aspetta che il `writer` finisca di svuotarsi. Se il motivo è `AccountDeleted`, il `loop` riparte dal punto 2 (nuova connessione, nuova autenticazione); altrimenti il processo termina, stampando un messaggio coerente con il motivo.

### 3.3 Ciclo principale (ui/main_ui.rs)

Una volta autenticato, il cuore dell'applicazione è il loop di [`ui::main_ui::run`](client/src/ui/main_ui.rs): ad ogni iterazione ridisegna la schermata, poi si mette in attesa — tramite `tokio::select!`, che permette di reagire al primo evento pronto tra più sorgenti asincrone senza bisogno di un ciclo di polling attivo — di uno tra:

- un **evento da tastiera** (`EventStream` di `crossterm`);
- un **messaggio dal server** (`server_msg_rx`, alimentato dal task `listener`);
- un **aggiornamento dello stato del movimento** (`movement_status_rx`, alimentato dal task `movement_sim`);
- un **errore locale** (`client_error_rx`, dai task `writer` e `movement_sim`).

Un evento da tastiera viene tradotto da `handle_key` (in [`input.rs`](client/src/ui/main_ui/input.rs)) in un `Outbound`, che indica cosa fare: `SendChat` (manda un `ChatMessage`), `DeleteAccount` (manda un `ClientMessage::DeleteAccount`), `Quit` (esce), oppure `None` (l'evento ha già aggiornato lo stato interno e non richide altro — es. lo spostamento tra riquadri con Tab). Un messaggio dal server aggiorna invece direttamente lo stato dell'`App`: `DirectMessage`/`BroadcastMessage` finiscono nei rispettivi log, `AccountDeleted` fa terminare il loop (in caso di successo) o mostra l'errore nel riquadro "Elimina account" (in caso di fallimento), `Error` finisce nel log Errori con un'etichetta di sistema.

Gestito uno di questi rami, il loop riparte da capo: ridisegna e torna in attesa.

### 3.4 Topologia: task e canali

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

Questi flussi convergono tutti nel `tokio::select!` di `ui::main_ui::run` descritto in 3.3. Avere un solo task dedicato alla scrittura e uno alla lettura del socket evita accessi concorrenti allo stream TCP; il disaccoppiamento tramite canali permette a UI, autenticazione e simulazione di produrre/consumare messaggi senza conoscersi direttamente né bloccarsi a vicenda, e senza bisogno di stato condiviso protetto da lock.

### 3.5 Mappa dei file

Indice di riferimento rapido ai file del client.

| File                                                                        | Ruolo                                                                                                                                                                                                                                                                          |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| [`main.rs`](client/src/main.rs)                                             | Punto di ingresso: legge il CSV, poi ripete in un `loop` l'intera sequenza connessione/canali/`writer`+`listener`+`movement_sim`/autenticazione/UI principale (3.2); il `loop` riparte automaticamente se l'utente elimina il proprio account, altrimenti il processo termina. |
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
| [`ui/main_ui.rs`](client/src/ui/main_ui.rs)                                 | Loop della schermata principale: `tokio::select!` tra eventi tastiera, `server_msg_rx`, `movement_status_rx` e `client_error_rx` (si veda 3.3). Ritorna un `ExitReason` (`UserQuit` / `ConnectionLost` / `AccountDeleted`) invece di `()`.                                     |
| [`ui/main_ui/state.rs`](client/src/ui/main_ui/state.rs)                     | Stato della schermata principale (`App`): riquadro attivo (`Panel`, incluso `DeleteAccount`), log di chat/broadcast/errori, stato movimento corrente, stato del flusso di eliminazione account (`DeleteAccountStep` e campi correlati).                                        |
| [`ui/main_ui/input.rs`](client/src/ui/main_ui/input.rs)                     | Traduce gli eventi tastiera in azioni (`Outbound::SendChat`/`DeleteAccount`/`Quit`/`None`) aggiornando lo stato dei riquadri; gestisce anche il flusso a più passi (password + conferma) del riquadro Elimina account, intercettando `Esc` per annullare senza chiudere l'app. |
| [`ui/main_ui/draw.rs`](client/src/ui/main_ui/draw.rs)                       | Rendering `ratatui` di tutti i riquadri della schermata principale a partire dallo stato `App`.                                                                                                                                                                                |

## 4. Test

Il progetto include una suite di **193 test**, eseguibile con `cargo test --workspace`, distribuiti tra `client` e `server`:

| Crate    | Unit test | Integration test | E2E test | Totale |
| -------- | --------- | ---------------- | -------- | ------ |
| `client` | 81        | 13               | 13       | 107    |
| `server` | 74        | 12               | —        | 86     |

- I **test unitari** si trovano accanto al codice che testano (`#[cfg(test)]` nei singoli moduli di `src/`) e coprono in isolamento la logica delle funzioni più a basso livello.
- I **test di integrazione** (cartella `tests/`, file senza prefisso `e2e_`) verificano un solo lato della comunicazione client/server alla volta. Per fare ciò, sul client, [`mock_connection`](client/tests/support/mod.rs) apre una semplice coppia TCP locale senza alcuna logica reale di server, per testare solo la serializzazione/deserializzazione dei messaggi; sul server, gli helper in [`support/mod.rs`](server/tests/support/mod.rs) avviano un server vero (`spawn_test_server`) ma lo interrogano senza passare dal codice reale del client.
- I **test e2e** (file `e2e_*.rs`, presenti solo nel crate `client`) avviano invece un server realmente funzionante e un client che replica fedelmente la topologia di `main.rs` ([`HeadlessClient`](client/tests/support/mod.rs)), verificando scenari end-to-end tra le due parti: autenticazione, scambio di messaggi diretti e broadcast, eliminazione account, disconnessione, spegnimento del server, più client connessi simultaneamente.

L'assenza di una suite e2e analoga nel crate `server` non è una lacuna, ma una conseguenza naturale di come sono organizzati i test: per avviare un client reale nei propri test, un crate deve includerlo come dipendenza. Il crate `client` lo fa (dipende da `server` per i suoi test e2e), mentre il crate `server` non dipende da `client`. Rifare la stessa suite e2e anche dentro `server` significherebbe solo ripetere gli stessi scenari già verificati dall'altra parte, senza testare nulla di nuovo.

## 5. Log delle prestazioni (CPU)

Come descritto in 2.2, il server scrive ogni 2 minuti una riga in `cpu_metrics.log` con il consumo di CPU del processo e quello globale del sistema.

Un'esecuzione di prova di circa 5 minuti, senza client connessi, ha prodotto:

```
[2026-09-11 12:46:12] App CPU: 0.00% (Normalized: 0.00%) | Global System CPU: 5.51%
[2026-09-11 12:48:12] App CPU: 0.01% (Normalized: 0.00%) | Global System CPU: 4.43%
[2026-09-11 12:50:12] App CPU: 0.00% (Normalized: 0.00%) | Global System CPU: 3.21%
```

Un'esecuzione di prova di circa 5 minuti, con 4 client connessi e un utilizzo intensivo dell'applicativo, ha prodotto:

```
[2026-09-11 12:37:11] App CPU: 0.00% (Normalized: 0.00%) | Global System CPU: 15.64%
[2026-09-11 12:39:11] App CPU: 1.40% (Normalized: 0.12%) | Global System CPU: 9.04%
[2026-09-11 12:41:11] App CPU: 6.19% (Normalized: 0.52%) | Global System CPU: 12.53%
```

Osservazioni:

- **App CPU cresce con il carico**: da 0,00% con nessun client a un picco di 6,19% (0,52% normalizzato) con 4 client connessi e traffico intenso.
- **Il carico resta comunque contenuto**: anche nello scenario più intenso testato, il valore normalizzato non supera lo 0,52% — segno che l'architettura asincrona basata su `Tokio` scala bene con più connessioni simultanee.

## 6. Dimensione applicativo

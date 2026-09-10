# Server

## Introduzione

Il server Georuggine è stato progettato per fornire un servizio di messaggistica **unicast bidirezionale** e **broadcast** verso i client e ricevere da questi ultimi dati GPS per il monitoraggio della _sosta_, del _movimento_, della _disconnessione_ e della _velocità_.

Tutto questo avviene tramite connessione `TCP` gestita dal server attraverso un socket di ascolto (`TcpListener`).
Più client possono connettersi simultaneamente grazie all'adozione della programmazione asincrona e al runtime `Tokio`, che gestisce le operazioni di I/O di rete in modo non bloccante e concorrente.

L'utilizzo della libreria `Ratatui` permette di avere un semplice interfaccia da terminale (TUI - Text User Interface) che nasconde la complessità della gestione degli eventi da tastiera e la suddivisione, nel medesimo terminale, in aree di visualizzazione differenti.

> Per client o utente si intende un qualsiasi dispositivo o software installato sul veicolo che si vuole monitorare. In questo progetto il movimento è stato effettuato leggendo un file con coordinate GPS.

### Sequenza di avvio (main.rs)

La prima parte di avvio del server è una sequenza di inizializzazione che permette di configurare il server:

1. Avvio di un logger (`start_cpu_logger.rs`) per la registrazione del consumo di CPU del nostro applicativo.
   La funzione di log inizia subito facendo uno spawn di un thread separato dal main. Utilizzando la libreria `sysinfo` vengono raccolte le informazioni del sistema ad intervalli di campionamento di 2 minuti.
   I risultato sono scritti nel file `cpu_metrics.log` che contiene le informazioni quali:
   - _timestamp_
   - _utilizzo CPU totale dell'app_
   - _utilizzo CPU normalizzato_
   - _consumo globale della CPU del server_

   > NOTA: Sysinfo riporta il consumo della CPU per processo senza tenere conto di quanti core sono presenti nel sistema. Ad esempio con 8 core si potrebbero verificare valori di CPU fino a 800%, per questo motivo è stato normalizzato questo valore dividendo per il numero di core presenti nel sistema.

2. Connessione al database `sqlite`. La scelta di utilizzare sqlite è stata dettata dalla semplicità di utilizzo, dalla leggerezza del database stesso e anche dalla semplicità dei dati che vengono salvati nel database. La scelta di altri database come ad esempio quelli documentali, come MongoDB, è stata scartata in quanto uno schema flessibile dei dati non è necessario per il nostro caso d'uso.

3. Avvio di diversi canali di comunicazione:
   - Un canale `shutdown_tx` Utilizzato per inviare a tutti i client connessi un avviso che il server sta per chiudere la connessione. Utile per dare tempo al client di predisporre eventuali azioni prima della chiusura della connessione.
   - Un canale `connections_notify` che notifica quando le connessioni al server cambiano. Questo serve per poter far in modo che il server possa aggiornare le informazioni di stato.
   - Un canale unbounded per la gestione, in particolare della TUI, dei messaggi provenienti dalle chat dei vari client connessi.
   - Un canale unbounded per la gestione degli errori provenienti da `handle_connection`

4. Inizializzazione dello stato dell'applicazione con tutti gli utenti presenti nel database.
5. Crezione di un canale oneshot utilizzato internamente per comunicare quando il servizio TCP di ascolto è pronto per ricevere connessioni.
6. Spawn di thread tokio separato per la gestione delle connessioni ed ascolto di eventuali errori provenienti da `handle_connection`.
7. Una volta che la connessione TCP è pronta, e dunque il server in ascolto, viene avviata la TUI

### Sequenza di avvio (main_ui.rs)

Questo file è il cuore della TUI.
All'inizio il flusso si presenta sequenziale per via delle varie inizializzazioni:

1. Si crea una istanzia di `App` (ui/main_ui/state.rs) che contiene lo stato dell'applicazione. A questa viene passato lo stato creato precedentemente in `main.rs` per permettere di inizializzare con i nomi degli utenti presenti nel database.
2. Inizializzazione di ratatui.
3. Gestione tramite il tratto Drop della chiusura pulita del terminale. Viene invocata automaticamente da Rust ed evita che a terminale rimanga l'interfaccia TUI.
4. La libreria `crossterm` permette di ricevere eventi asincroni da tastiera grazie alla creazione di un `EventStream`.
5. Sottoscrizione al canale creato precedentemente in `main.rs` per la ricezione di nuove connessioni e disconnessioni dei client, permettendo una iterazione del loop e, dunque, la visualizzazione dei nuovi cambiamenti.

#### Loop principale

Come prima cosa vengono aggiornati i dati prima della visualizzazione su schermo:

1. Aggiornamento del'indice della selezione delle statistiche da visualizzare.
2. Aggiornamento della lista degli utenti presenti nel database con il proprio stato (**Sconnesso**, **Fermo**, **InMovimento** **Problema**) utilizzato per la visualizzazione lato server.
3. Vengono letti i nomi degli utenti connessi
4. Caricamento delle vecchie chat degli utenti connessi al database
5. Visualizzazione a schermo della TUI.
6. Gestione degli eventi da tastiera grazie l'utilizzo di `tokio::select!` che permette la gestione di eventi asincroni senza consumare cicli di CPU.
   - La funzione `handle_key` riceva un KeyEvent in input e ritorna un enum `Outbound` che puo indicare _Quit_, _SendChat_, _SendBroadcast_ oppure _None_. Questa funzione utilizza il focus per permettere la _selezione_, la _scrittura_ e l'_invio_ di messaggi verso i client connessi:
     - Esc: Ritorna sun Outbound::Quit
     - Tab: Gestisce, attraverso una macchina a stati finiti, dove si trova il focus così da permettere a `draw.rs` di disegnare a schermo il bordo colorato della selezione attiva.
     - BackTab: Uguale a Tab ma gestisce il verso inverso della selezione.

7. Una volta capito quale evento da tasiera è stato prodotto, questo viene gestito tarmite un match:
   - **Outbound::Quit**: Vengono inviati a tutti i client messaggi broadcast di chiusura connessione. Si invia anche un messaggio vuoto di shoutdown e si prevedono 300ms per la chiusura della connessione.
   - **Outbound::SendChat**: Invia al client selezionato il messaggio scritto nella casella di testo.
   - **Outbound::SendBroadcast**: Invia a tutti i client connessi il messaggio scritto nella casella broadcast di testo.
   - **Outbound::QueryStats**: Vengono aggiornate le statistiche dello specifico user selezionato.

> Se non fosse un evento da tastiera, come ad esempio la ricezione di un messaggio da parte di un client, il canale chat_rx verrebbe notificato e verrebbe selezionato l'utente corretto, pushato il nuovo messaggio nella chat ed aggiornato il numero di eventuali notifiche che l'amministratore non ha ancora letto.
>
> Eventuali errori provenienti dal canale `error_rx` vengono stampati direttamente nella TUI, nell'apposito box di errori.

Una volta intrapreso uno di questi 4 rami:

- Tasto premuto (maybe_event = term_events.next())
- Messaggio chat in arrivo (chat_rx.recv())
- Notifica di errore (error_rx.recv())
- Variazione dei client connessi (connections_rx.changed())

il loop riparte da capo: aggiorna i dati, ridisegna la TUI e si mette di nuovo in attesa di eventi.

#### Mappa dei file

| File                                            | Ruolo                                                                                                                                                                                                                                                                             |
| :---------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `server/src/main.rs`                            | Entry point del server: inizializza il logger CPU, apre il database SQLite, istanzia lo stato condiviso `AppState` con i relativi canali (broadcast, watch, mpsc), avvia il listener TCP in background e avvia l'interfaccia terminale TUI.                                       |
| `server/src/network.rs`                         | Gestisce l'ascolto di rete TCP: si lega all'indirizzo configurato, notifica l'avvio tramite canale oneshot, accetta le connessioni in ingresso dai client in un loop asincrono e genera per ciascuna un task Tokio dedicato a `handle_connection`.                                |
| `server/src/state.rs`                           | Definisce lo stato globale condiviso `AppState` (pool SQLite, mappa delle connessioni attive protetta da `RwLock`, stato utenti in tempo reale e canali di notifica/shutdown) e le strutture dati correlate (`UserStatus`, `Info`, `IncomingChat`, `IncomingError`).              |
| `server/src/messaging.rs`                       | Fornisce le primitive I/O a basso livello sul socket TCP: serializza e invia messaggi `ServerMessage` in formato JSON newline-delimited (`send_message`) e legge e deserializza a righe i `ClientMessage` ricevuti (`receive_message`).                                           |
| `server/src/auth.rs`                            | Modulo per la sicurezza e l'hashing delle credenziali: implementa `hash_password` e `verify_password` basate su algoritmo crittografico Argon2 e sale casuale (`OsRng`), garantendo che le password non siano mai memorizzate in chiaro.                                          |
| `server/src/db.rs`                              | Layer di accesso ai dati su SQLite tramite SQLx: gestisce l'inizializzazione dello schema (`init_schema`), le operazioni CRUD sugli utenti (`insert_user`, `delete_user`, `get_all_users`) e il salvataggio/recupero della cronologia delle posizioni GPS (`insert_track_point`). |
| `server/src/stats.rs`                           | Calcola le metriche di movimento dalla cronologia dei punti GPS: calcola distanze con la formula di Haversine, velocità media, tempo in movimento e sosta (escludendo disconnessioni >35s) per rispondere alle richieste di statistiche dalla TUI.                                |
| `server/src/user_status.rs`                     | Gestisce la mappa in memoria dello stato operativo degli utenti (`Sconnesso`, `Fermo`, `InMovimento`, `Problema`): aggiorna gli stati correnti, incrementa i contatori di permanenza temporale e notifica in tempo reale la TUI delle variazioni.                                 |
| `server/src/bin/script_hashing.rs`              | Utility CLI per generare offline gli hash Argon2 delle password, utilizzata per creare le credenziali degli utenti predefiniti da inserire nel database iniziale.                                                                                                                 |
| `server/src/handlers/handle_connection.rs`      | Gestisce il ciclo di vita di una singola connessione client: riceve i pacchetti TCP, li smista all'handler specifico, gestisce un watchdog timer (35s) per rilevare l'inattività (stato `Problema`) e pulisce connessioni e stati alla disconnessione o allo shutdown.            |
| `server/src/handlers/handle_delete_account.rs`  | Gestisce la cancellazione dell'account (`DeleteAccount`): verifica la password tramite hash su DB, rimuove l'utente e i suoi punti GPS da SQLite, cancella la connessione attiva e lo stato in memoria e conferma l'eliminazione al client.                                       |
| `server/src/handlers/handle_login.rs`           | Gestisce l'autenticazione (`Login`): verifica le credenziali con Argon2 sul database SQLite, inserisce il sender del client nella mappa delle connessioni attive, notifica la TUI dell'avvenuto login e risponde con `AuthResult`.                                                |
| `server/src/handlers/handle_new_track_point.rs` | Elabora gli aggiornamenti GPS (`PositionUpdate`): aggiorna la macchina a stati di mobilità (`InMovimento`, `Fermo` dopo 180s o recovery da `Problema`), persiste il punto nel database e aggiorna i tempi di permanenza.                                                          |
| `server/src/handlers/handle_registration.rs`    | Gestisce la registrazione di un nuovo utente (`Register`): genera l'hash Argon2 della password, inserisce l'utente nel DB SQLite verificando che l'username non sia duplicato, autentica la sessione e risponde con `AuthResult`.                                                 |
| `server/src/logging/cpu_logger.rs`              | Monitora le prestazioni della CPU in un thread di background separato: campiona ogni 120s l'uso CPU del processo (normalizzato sui core) e globale tramite `sysinfo`, salvando i dati nel file `cpu_metrics.log`.                                                                 |
| `server/src/ui/main_ui.rs`                      | Coordina l'event loop della TUI con `tokio::select!`: sincronizza lo stato locale da `AppState`, ridisegna l'interfaccia e gestisce concorrentemente eventi tastiera, chat in arrivo dai client, errori di rete e variazioni delle connessioni.                                   |
| `server/src/ui/main_ui/draw.rs`                 | Implementa il rendering grafico della TUI con Ratatui: disegna il layout a più pannelli (utenti, chat diretta, broadcast, statistiche di movimento, log errori e guida tasti), evidenziando il focus attivo e gestendo scrolling e wrapping del testo.                            |
| `server/src/ui/main_ui/input.rs`                | Interpreta gli eventi da tastiera (`handle_key`): gestisce lo spostamento del focus tra i pannelli (Tab/BackTab), lo scorrimento dei log e converte i comandi utente nell'enum `Outbound` (`SendChat`, `SendBroadcast`, `QueryStats`, `Quit`).                                    |
| `server/src/ui/main_ui/state.rs`                | Definisce i modelli e lo stato interno della TUI (`App`): memorizza i log di chat e broadcast, la cronologia errori, i buffer di input testo, il pannello correntemente selezionato (`Panel`) e lo stato del wizard delle statistiche (`StatsStep`).                              |
| `server/src/ui/size_control.rs`                 | Definisce le dimensioni minime della finestra del terminale (80 colonne x 29 righe) e ne verifica il rispetto, mostrando un messaggio di avviso se lo spazio è insufficiente per visualizzare correttamente la TUI.                                                               |
| `server/src/ui/terminal_guard.rs`               | Implementa il pattern RAII per il terminale: assicura tramite il tratto `Drop` che, in caso di uscita normale, errore o panic, venga sempre ripristinato lo stato originale del terminale (`ratatui::restore()`) evitando corruzioni della shell.                                 |

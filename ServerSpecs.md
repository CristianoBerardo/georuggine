# Server

## Introduzione

Il server Georuggine è stato progettato per fornire un servizio di messaggistica unicast bidirezionale e broadcast verso i client e ricevere da questi ultimi dati GPS per il monitoraggio della sosta, del movimento, della disconnessione e della velocità.

Tutto questo avviene attraverso connessione `TCP` a molti utenti simultaneamente grazie all'utilizzo delle programmazione asincrona massa a disposizione da una delle più comuni librerie utilizzate per questo scopo: `Tokio`.

L'utilizzo della libreria `Ratatui` permette di avere un semplice interfaccia da terminale TUI (Text User Interface) che nasconde la complessità della gestione degli eventi da tastiera e la suddivisione, nel medesimo terminale, in aree di visualizzazione differenti.

> Per client o utente si intende un qualsiasi dispositivo o software installato sul veicolo che si vuole monitorare. In questo progetto il movimento è stato effettuato leggendo un file con coordinate GPS.

### Sequenza di avvio (main.rs)

La prima parte di avvio del server è una sequenza di inizializzazione che permette di configurare il server:

1. Avvio di un logger (`start_cpu_logger.rs`) per la registrazione del consumo di CPU del nostro applicativo.
   La funzione di log inizia subito facendo uno spawn di un thread separato dal main. Utilizzando la libreria `sysinfo` vengono raccolte le informazioni del sistema ad intervalli di campionamento di 2 minuti.
   I risultato sono scritti nel file `cpu_metrics.log` e contiene le informazioni quali: timestamp, utilizzo CPU totale dell'app, utilizzo CPU normalizzato e il consumo globale della CPU del server.

   > NOTA: Sysinfo riporta il consumo della CPU per processo senza tenere conto di quanti core sono presenti nel sistema. Ad esempio con 8 core si potrebbero verificare valori di CPU fino a 800%, per questo motivo è stato normalizzato questo valore dividendo per il numero di core presenti nel sistema.

2. Connessione al database `sqlite`. La scelta di utilizzare sqlite è stata dettata dalla semplicità di utilizzo, dalla leggerezza del database stesso e anche dalla semplicità dei dati che vengono salvati nel database. La scelta di altri database come ad esempio quelli documentali, come MongoDB, è stata scartata in quanto uno schema flessibile dei dati non è necessario per il nostro caso d'uso.

3. Avvio di diversi canali di comunicazione:
   - Un canale `shutdown_tx` utilizzato per la gestione della chiusura del server. Utilizzato per inviare a tutti i client connessi un avviso che il server sta per chiudere la connessione. Utile per dare tempo al client di predisporre eventuali azioni prima della chiusura della connessione.
   - Un canale `connections_notify` che notifica quando le connessioni al server cambiano per poter far in modo che il server possa aggiornare le informazioni di stato del server stesso.
   - Un canale unbounded per la gestione da parte del server, in particolare della TUI, dei messaggi provenienti dalle chat dei vari client connessi.
   - Un canale unbounded per la gestione degli errori provenienti da `handle_connection`

4. Inizializzazione dello stato dell'applicazione con tutti gli utenti presenti nel database.
5. Crezione di un canale oneshot utilizzato internamente per comunicare quando il servizio TCP di ascolto è pronto per ricevere connessioni.
6. Spawn di thread tokio separato per la gestione delle connessioni ed ascolto di eventuali errori provenienti da `handle_connection`.
7. Una volta che la connessione TCP è pronta, e dunque il server in ascolto, viene avviata la TUI

### Sequenza di avvio (main_ui.rs)

Questo file è il cuore della TUI.
All'inizio il flusso si presenta sequenziale per via delle varie inizializzazioni:

1. Si crea una istanzia di `App` (ui/main_ui/state.rs) che contiene lo stato dell'applicazione. A questa viene passato lo stato creato precedentemente in `main.rs` per permettere di inizializzare con i nomi degli utenti presenti nel database.
2. Inizializzazione di ratatui
3. Gestione tramite il tratto Drop della chiusura pulita del terminale, viene invocata automaticamente da Rust. La funzione `ratatui::restore()` al verificarsi un panic o quando viene restituito il valore Ok() alla pressione del tasto esc. Questo evita che a terminale rimanga l'interfaccia TUI.
4. La libreria `crossterm` mette a disposizione la gestione degli eventi grazie alla creazione di un `EventStream` che permette di ricevere eventi asincroni da tastiera.
5. Sottoscrizione al canale creato precedentemente in `main.rs` per la ricezione di nuove connessioni e disconnessioni dei client, permettendo un iterazione del loop e la nuova visualizzazione dei cambiamenti.

#### Loop principale

Come prima cosa vengono aggiornati i dati prima della visualizzazione su schermo:

// ! DA RIGUARDARE NON HO BEN CAPITO COSA FA

1. Aggiornamento del'indice della selezione delle statistiche da visualizzare.
2. Aggiornamento della lista degli utenti presenti nel database con il proprio stato (Sconnesso, Fermo, InMovimento Problema) utilizzato per la visualizzazione lato server.
3. NON HO CAPITO app.stats_user_index = stats_selected_username
   .and_then(|name| app.users.iter().position(|u| u.username == name));

4. Vengono letti i nomi degli utenti connessi
5. Caricamento delle vecchie chat degli utenti connessi al database
6. Visualizzazione a schermo della TUI.
7. Gestione degli eventi da tastiera grazie l'utilizzo di `tokio::select!` che permette la gestione di eventi asincroni senza consumare creare un ciclo busy wait con polling continuo.

   Vengono gestiti i seguenti eventi da tastiera:
   - Evento da tastiera tramite la funzione `handle_key` la quale riceva un KeyEvent in input e ritorna un enum `Outbound` che puo indicare Quit, SendChat, SendBroadcast oppure None. Questa funzione utilizza il focus per permettere la selezione, la scrittura e l'invio di messaggi verso i client connessi:
     - Esc: Ritorna un Outbound::Quit
     - Tab: Gestisce, attraverso una macchina a stati finiti, dove si trova il focus così da permettere a `draw.rs` di disegnare a schermo il bordo colorato della selezione attiva.
     - BackTab: Uguale a Tab ma gestisce il verso inverso della selezione.

8. Una volta capito di quale evento da tasiera di parla viene gestito in un match per i vari casi:
   - Outbound::Quit: il più semplice: Vengono inviati a tutti i client messaggi broadcast di chiusura connessione, i quali possono eventualmente prevedere politiche di chiusura connessione. Si invia anche un messaggio vuoto di shoutdown e si prevedono 300ms per la chiusura delle connessioni.
   - Outbound::SendChat: Invia al client selezionato il messaggio scritto nella casella di testo.
   - Outbound::SendBroadcast: Invia a tutti i client connessi il messaggio scritto nella casella broadcast di testo.
   - Outbound::QueryStats: E' DA INSERIRE O NON PIù??

Se non fosse un evento da tastiera, come ad esempio la ricezione di un messaggio da parte di un client, il canale chat_rx verrebbe notificato e verrebbe selezionato l'utente corretto, pushato il nuovo messaggio nella chat ed aggiornato il numero di eventuali notifiche che l'amministratore non ha ancora letto.

Eventuali errori provenienti dal canale `error_rx` vengono stampati direttamente nella TUI, nell'apposito box di errori.

Una volta intrapreso uno di questi 4 rami:

- Tasto premuto (maybe_event = term_events.next())
- Messaggio chat in arrivo (chat_rx.recv())
- Notifica di errore (error_rx.recv())
- Variazione dei client connessi (connections_rx.changed())

il loop riparte da capo aggiorna i dati, ridisegna la TUI e si mette di nuovo in attesa di eventi.

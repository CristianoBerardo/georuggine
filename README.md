# GeoRuggine

GeoRuggine è un'applicazione client-server che simula il monitoraggio della posizione e degli spostamenti di un veicolo. Il server tiene traccia dei punti di tracciamento (`TrackPoint`) inviati dai client autenticati e ne calcola statistiche di movimento (distanza percorsa, velocità media, tempo in movimento/in sosta) su richiesta.

## Architettura

Il progetto è organizzato come workspace Cargo con tre crate:

- **`common`**: tipi condivisi tra client e server (modelli dati e protocollo di comunicazione), così da garantire che entrambe le parti parlino "la stessa lingua".
- **`server`**: applicazione server che accetta connessioni TCP, gestisce autenticazione, persistenza su database e calcolo delle statistiche.
- **`client`**: applicazione CLI che si connette al server, effettua login/registrazione e permette di interagire tramite un menu testuale.

## Come funziona

### Comunicazione client-server

Client e server comunicano su una connessione TCP (`127.0.0.1:8080` di default) scambiandosi messaggi JSON delimitati da `\n` (un messaggio per riga). I messaggi sono definiti in `common::protocol`:

- `ClientMessage`: `Register`, `Login`, `PositionUpdate`, `ChatMessage`, `QueryStats`.
- `ServerMessage`: `AuthResult`, `BroadcastMessage`, `DirectMessage`, `StatsResult`, `Error`.

Ogni connessione client viene gestita dal server in un task `tokio` dedicato. All'interno di questo task, un `tokio::select!` permette di gestire contemporaneamente:

1. i messaggi in arrivo dal client (letti dal socket);
2. i messaggi asincroni da inviare al client (broadcast/unicast avviati dall'operatore del server), accodati su un canale `mpsc` associato allo username autenticato.

### Autenticazione

All'avvio, il client chiede all'utente se vuole effettuare il **login** o la **registrazione**:

- In fase di **registrazione**, la password viene richiesta due volte (per conferma) e mai trasmessa o salvata in chiaro: il server la sottopone a hashing con **Argon2** prima di salvarla nel database.
- In fase di **login**, il server recupera l'utente dal database e verifica la password confrontandola con l'hash salvato.

In entrambi i casi il server risponde con un `AuthResult` (successo/fallimento). Se l'autenticazione va a buon fine, la connessione dell'utente viene registrata nella mappa condivisa `connections` (chiave: username, valore: canale di invio), usata per l'invio di messaggi broadcast/unicast, e il client avvia un task `listener` che resta in ascolto di messaggi asincroni provenienti dal server (messaggi diretti, broadcast, risultati di statistiche) mostrandoli a video.

### Menu del client

Una volta autenticato, il client mostra un menu con tre opzioni:

1. **Invia messaggio al server**: invia un `ChatMessage` libero.
2. **Stampa statistiche**: chiede il periodo di interesse (oggi / questa settimana / questo mese) e invia una `QueryStats`; la risposta (`StatsResult`) viene mostrata dal listener asincrono.
3. **Disconnetti**: chiude la connessione con il server.

### Menu del server

Il server, in parallelo all'accettazione di nuove connessioni, mostra un menu che permette all'operatore di:

1. **Inviare un messaggio broadcast** a tutti gli utenti attualmente connessi.
2. **Inviare un messaggio unicast** a un singolo utente connesso, selezionato per username.

### Statistiche di movimento

Le statistiche (`MovementStats`) sono calcolate a partire dallo storico dei `TrackPoint` di un utente nel periodo richiesto:

- la **distanza percorsa** viene calcolata sommando, tra coppie di punti consecutivi in stato `InMovimento`, la distanza geografica ottenuta con la **formula di Haversine**;
- il **tempo in movimento** e il **tempo di sosta** vengono accumulati in base allo stato (`InMovimento` / `Fermo`) registrato tra un punto e il successivo;
- coppie di punti con un intervallo temporale superiore a 30 minuti vengono ignorate, poiché indicano una disconnessione del tracker e non un reale periodo di marcia o sosta continua;
- la **velocità media** è calcolata come distanza totale diviso tempo totale in movimento.

### Modello dati

- `User`: utente registrato (username, hash della password).
- `Position` / `TrackPoint`: coordinate geografiche (`lat`, `lon`) con timestamp e stato del veicolo (`VehicleState`: `Sconnesso`, `Fermo`, `InMovimento`).
- `MovementStats`: risultato aggregato di un'interrogazione sulle statistiche (distanza, velocità media, durata movimento/sosta).

I dati sono persistiti in un database SQLite (`server/data/georuggine.db`) tramite `sqlx`.

# Avviare l'applicazione

1. **Avviare il server**:

   ```powershell
   cargo run -p server
   ```

   Output atteso:

   ```text
   Pool fatto
   Stato fatto
   Server in ascolto su 127.0.0.1:8080
   ```

2. **Avviare il client** (in un nuovo terminale):

   ```powershell
   cargo run -p client
   ```

   Output atteso:

   ```text
   Connessione a 127.0.0.1:8080 in corso...
   Connessione stabilita con successo!
   [AUTH] Autenticazione riuscita!
   ```

## Utenti

L'applicazione propone un set di 2 utenti preimpostati:

| Utente | Password     |
| ------ | ------------ |
| mario  | supersegreta |
| anna   | password     |

- **mario**: possiede già dei dati di tracciamento relativi all'ultimo mese, quindi interrogando le statistiche (periodo "oggi", "questa settimana" o "questo mese") restituisce risultati non nulli.
- **anna**: possiede solo l'account, senza alcun dato di tracciamento associato.

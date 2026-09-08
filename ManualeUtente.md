# Manuale Utente

## GeoRuggine Server

TO DO

## GeoRuggine Client

Questo manuale si pone come obiettivo di spiegare come utilizzare GeoRuggine, l'applicazione da terminale che simula il movimento di un mezzo e permette di comunicare con l'amministratore del sistema.

### 1. Avvio dell'applicazione

Prima di avviare il client, assicurati che il server sia già in esecuzione, altrimenti il collegamento fallirà e il programma si chiuderà con un messaggio di errore.
Il client si avvia da un'istanza del terminale con il seguente comando:
`cargo run -p client` e si collega automaticamente al server.

Per una visualizzazione corretta è necessario un terminale di almeno 80 colonne per 29 righe: se lo spazio disponibile è inferiore, il client mostra un avviso al posto della schermata normale, finché non ingrandisci la finestra.

All'avvio, il client stampa a schermo l'esito del tentativo di connessione; se va a buon fine, si entra nella schermata di accesso.

### 2. Accesso: login e registrazione

#### 2.1 Scelta tra Login e Registrazione

La prima schermata mostra due opzioni: **Login** e **Registrazione**.

- **↑ / ↓**: sposta la selezione tra le due opzioni (evidenziata in giallo).
- **Invio**: conferma la scelta ed entra nella schermata corrispondente.
- **Esc**: chiude l'applicazione.

#### 2.2 Login

Il modulo di login ha due campi: **Username** e **Password**.

- **Tab**: passa da un campo all'altro (il campo attivo, cioè quello con cui stai interagendo in quel momento, ha il bordo giallo).
- Digita normalmente per inserire il testo nel campo attivo; **Backspace** cancella l'ultimo carattere.
- La password viene mostrata mascherata con asterischi (`*`).
- **Invio**: conferma e invia le credenziali al server. Se uno dei due campi è vuoto, compare un messaggio d'errore e non viene inviato nulla.
- Mentre si attende la risposta del server, il messaggio "In attesa di risposta dal server..." è visibile in basso e tutti i tasti sono ignorati tranne **Esc**.
- Se il login fallisce (credenziali errate), compare un messaggio d'errore in rosso e puoi correggere e riprovare senza dover uscire.
- **Esc**: se non stai aspettando una risposta dal server, torna alla schermata di scelta Login/Registrazione. Se invece lo premi proprio mentre è visibile "In attesa di risposta dal server...", **chiude l'intera applicazione** invece di tornare indietro.

#### 2.3 Registrazione

Il modulo di registrazione ha tre campi: **Username**, **Password** e **Conferma Password**. Stessa navigazione del login (Tab per spostarsi tra i campi, Invio per confermare).

- Prima dell'invio viene controllato che nessun campo sia vuoto e che password e conferma della password coincidano; in caso contrario compare un messaggio d'errore.
- Mentre si attende la risposta del server, tutti i tasti sono ignorati tranne **Esc**.
- Se la registrazione va a buon fine, l'applicazione torna automaticamente alla schermata di Login, pronta per effettuare l'accesso con le credenziali appena create o con un altro set di credenziali.
- **Esc**: se non stai aspettando una risposta dal server, torna alla schermata di scelta Login/Registrazione. Se invece lo premi proprio mentre è visibile "In attesa di risposta dal server...", **chiude l'intera applicazione** invece di tornare indietro.

#### 2.4 Utenti preimpostati

L'applicazione mette a disposizione un set di 2 utenti preimpostati:

| Utente | Password     |
| ------ | ------------ |
| mario  | supersegreta |
| anna   | password     |

- **mario**: possiede già dei dati di tracciamento relativi all'ultimo mese.
- **anna**: possiede solo l'account, senza alcun dato di tracciamento associato.

### 3. Schermata principale

Dopo l'accesso, la schermata si divide in due colonne e due barre infondo:

```
┌─────────────┬────────────────────┐
│ Utente      │ Chat               │
├─────────────┤                    │
│ Elimina     │                    │
│ account     │                    │
├─────────────┤                    │
│ Movimento   │                    │
├─────────────├────────────────────┤
│ Broadcast   │ Scrivi messaggio   │
├─────────────┴────────────────────┤
│ Errori                           │
├──────────────────────────────────┤
│ Barra di aiuto                   │
└──────────────────────────────────┘
```

Il riquadro attivo — cioè quello con cui stai interagendo in quel momento — ha sempre il bordo giallo, per farti capire subito dove ti trovi. Nei riquadri scorrevoli (Chat, Scrivi messaggio, Broadcast, Errori), quando il contenuto supera lo spazio visibile il titolo mostra anche `↑` e/o `↓`, per farti capire che c'è altro testo sopra o sotto rispetto a quanto vedi in quel momento. Puoi spostarti tra i riquadri così:

- **Tab**: passa al riquadro successivo, nell'ordine Utente → Elimina account → Movimento → Broadcast → Chat → Scrivi messaggio → Errori (dopo l'ultimo si torna al primo).
- **Shift+Tab**: passa al riquadro precedente, nello stesso ordine ma al contrario.
- **Esc**: chiude l'applicazione (e con essa la connessione al server), da qualunque riquadro ti trovi — **tranne** durante la procedura di eliminazione account già avviata (3.2), dove invece annulla solo il passo corrente senza chiudere il programma.

#### 3.1 Utente

Mostra semplicemente il nome utente con cui hai effettuato l'accesso. È un riquadro di sola lettura.

#### 3.2 Elimina account

Permette di cancellare **definitivamente** il proprio account e tutti i dati di tracciamento ad esso associati. La procedura è a più passi, per evitare eliminazioni accidentali:

1. **Invio** sul riquadro: avvia la procedura e mostra il campo password.
2. Digita la password (mostrata mascherata con `*`). In questo passo:
   - **Backspace**: cancella l'ultimo carattere digitato.
   - **Invio** (con il campo non vuoto): passa alla conferma.
   - **Esc**: annulla e torna al riquadro a riposo.
3. Compare la richiesta "Eliminare DAVVERO l'account? Azione irreversibile. (y/n)":
   - **y**: conferma l'eliminazione.
   - **n** oppure **Esc**: annulla e torna al riquadro a riposo.
4. Dopo la conferma, mentre si attende la risposta del server compare "Eliminazione in corso..." e l'input viene ignorato.

Importante: durante l'intera procedura (dal passo 2 in poi) **Esc** annulla soltanto il passo corrente e **non chiude l'applicazione**, a differenza del comportamento di Esc su tutti gli altri riquadri.

Esiti possibili:

- **Password errata**: il server rifiuta la richiesta, il messaggio di errore compare sotto il campo password e puoi correggere e ritentare (si torna al passo 2 con il campo password vuoto).
- **Eliminazione riuscita**: la sessione si chiude e il client **torna automaticamente alla schermata di scelta Login/Registrazione**, come se fosse stato appena avviato — l'account e tutti i dati di tracciamento ad esso associati sono stati rimossi in modo permanente dal server, e potrai effettuare un nuovo accesso o una nuova registrazione senza dover riavviare manualmente il programma.

#### 3.3 Stato movimento

Il client simula automaticamente, in background, il movimento di un mezzo lungo un percorso, inviando la posizione al server ogni 30 secondi circa. Questo riquadro mostra:

- **Stato**:
  - "In movimento" mentre gli invii proseguono;
  - "Fermo" se il mezzo è fermo nella stessa posizione da più di 3 minuti;
  - "Problema" (in rosso) se il mezzo ha smesso di inviare posizioni.
- **Primo invio** e **Ultimo invio**: orario del primo e dell'ultimo aggiornamento di posizione ricevuto.
- **Posizione**: le coordinate dell'ultimo punto inviato.

Questo riquadro si aggiorna in automatico con il movimento del mezzo e non richiede alcun tipo di interazione.

#### 3.4 Broadcast

Mostra gli annunci che l'amministratore invia a **tutti** gli utenti connessi, con orario di ricezione. È un riquadro di sola lettura, distinto dalla chat diretta.

- **↑ / ↓** (quando questo riquadro è quello attivo): scorre lo storico dei messaggi, utile quando sono troppi per stare tutti a schermo.

Quando l'amministratore spegne il server, in questo riquadro compare un vero e proprio conto alla rovescia, in sequenza: "Il server si sta arrestando, verrai disconnesso in 3 secondi...", poi "...in 2 secondi...", poi "...in 1 secondi..." — subito dopo quest'ultimo la connessione si chiude e vedrai il messaggio di perdita di connessione (3.9).

#### 3.5 Chat

Mostra la conversazione diretta tra te e l'amministratore:

- `>` indica un messaggio che hai inviato tu.
- `<` indica un messaggio ricevuto dall'amministratore.
- Ogni riga riporta l'orario del messaggio.
- **↑ / ↓** (quando questo riquadro è quello attivo): scorre lo storico, come per il Broadcast.

#### 3.6 Scrivi messaggio

Il campo dove componi un messaggio da inviare all'amministratore:

- Digita normalmente; **Backspace** cancella l'ultimo carattere.
- **Invio**: invia il messaggio (se non è vuoto) — comparirà subito nel riquadro "Chat" preceduto da `>`.
- **↑ / ↓**: permette di scorrere il messaggio nel caso in cui sia più lungo di 2 righe.

#### 3.7 Errori

Riquadro a tutta larghezza, sotto le due colonne: mostra tutti gli errori che possono verificarsi durante l'uso, in rosso e con l'orario in cui si sono verificati. È un riquadro di sola lettura: non contiene messaggi dell'amministratore, ma segnalazioni di errore, di due tipi diversi (riconoscibili dal prefisso):

- **`[WRITER]`** o **`[MOVEMENT_SIM]`**: problemi tecnici locali del client (ad es. un aggiornamento di posizione o un messaggio che non è stato possibile inviare al server).
- **`[SERVER]`**: errori di sessione segnalati dal server (ad es. un aggiornamento di posizione rifiutato perché non sei autenticato).

- **↑ / ↓** (quando questo riquadro è quello attivo): scorre lo storico degli errori, come per Chat e Broadcast.
- Se non si è mai verificato nessun problema, il riquadro resta vuoto.

#### 3.8 Barra di aiuto

In fondo allo schermo, mostra un promemoria dei tasti disponibili, che cambia in base al riquadro attivo in quel momento (es. ricorda come scrivere e inviare un messaggio quando sei su "Scrivi messaggio", oppure ricorda la procedura password+conferma quando sei su "Elimina account").

#### 3.9 Perdita di connessione

Se durante l'uso della schermata principale la connessione con il server cade — sia in modo improvviso (rete o server bloccati), sia al termine della sequenza di spegnimento descritta al 3.4 — il client mostra il messaggio "Connessione al server persa. Uscita dall'applicazione." e si chiude. Se succede, verifica che il server sia attivo e riavvia il client per accedere di nuovo.

### 4. Riepilogo dei tasti

| Tasto         | Effetto                                                                                                                         |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| **Tab**       | Passa al riquadro successivo                                                                                                    |
| **↑ / ↓**     | Cambia selezione (login/registrazione) o scorre lo storico (chat/broadcast/errori)                                              |
| **Invio**     | Conferma la scelta, invia il modulo o il messaggio, oppure avanza di un passo nella procedura "Elimina account"                 |
| **y / n**     | Nel passo di conferma di "Elimina account": conferma (y) o annulla (n) l'eliminazione                                           |
| **Backspace** | Cancella l'ultimo carattere digitato                                                                                            |
| **Esc**       | Torna indietro (dai moduli di login/registrazione), annulla il passo corrente ("Elimina account"), oppure chiude l'applicazione |

### 5. Uscire dall'applicazione

Premi **Esc** da qualunque punto della schermata principale (o dalla schermata di scelta Login/Registrazione) per chiudere il client in modo pulito: la connessione con il server viene chiusa e il terminale torna al suo stato normale mostrando il messaggio "Uscita dall'applicazione. Disconnessione dal server...".

C'è un secondo modo, distinto dall'uscita, in cui la sessione principale termina: **eliminando il proprio account** (3.2). In quel caso il client **non si chiude**, ma torna alla schermata di scelta Login/Registrazione, pronto per un nuovo accesso.

### 6. Problemi comuni

| Messaggio                                                                                                                                                                                          | Quando compare                           | Cosa significa                                                                                                                                             |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| "Impossibile connettersi al server: ..."                                                                                                                                                           | All'avvio                                | Il server non è raggiungibile: verifica che sia in esecuzione                                                                                              |
| "Username e password non possono essere vuoti."                                                                                                                                                    | Login/registrazione                      | Hai lasciato un campo vuoto                                                                                                                                |
| "Le password non corrispondono."                                                                                                                                                                   | Registrazione                            | Password e conferma sono diverse                                                                                                                           |
| "Credenziali non valide"                                                                                                                                                                           | Login                                    | Username o password errati (per sicurezza il server non specifica quale dei due)                                                                           |
| "Username già in uso."                                                                                                                                                                             | Registrazione                            | L'username scelto è già stato preso da un altro utente: scegline un altro                                                                                  |
| "Errore durante la registrazione: ..."                                                                                                                                                             | Registrazione                            | Si è verificato un problema imprevisto lato server (diverso da un username già in uso)                                                                     |
| Conto alla rovescia "Il server si sta arrestando..." (vedi 3.4)                                                                                                                                    | Broadcast, durante l'uso                 | L'amministratore sta spegnendo il server: la connessione cadrà a breve e il client si chiuderà                                                             |
| Messaggi nel riquadro "Errori" col prefisso "[WRITER]" o "[MOVEMENT_SIM]" (es. "[WRITER] Errore nell'invio del messaggio: ..." o "[MOVEMENT_SIM] Impossibile inviare la posizione: canale chiuso") | Durante l'uso, riquadro Errori           | Un problema temporaneo locale (invio di un messaggio o di un aggiornamento di posizione non riuscito); se persiste, la connessione potrebbe cadere a breve |
| Messaggi nel riquadro "Errori" col prefisso "[SERVER]" (es. "[SERVER] Devi essere autenticato per inviare aggiornamenti di posizione.")                                                            | Durante l'uso, riquadro Errori           | Un problema di sessione segnalato dal server, distinto da un errore locale del client                                                                      |
| "Connessione al server persa. Uscita dall'applicazione."                                                                                                                                           | Durante l'uso della schermata principale | Il server si è arrestato o la connessione di rete è caduta: il client si chiude, riavvialo quando il server è di nuovo raggiungibile                       |
| "Password errata."                                                                                                                                                                                 | Riquadro "Elimina account"               | La password inserita non corrisponde a quella dell'account: puoi correggerla e ritentare, oppure premere Esc per annullare                                 |

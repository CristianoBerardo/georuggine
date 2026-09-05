# Manuale Utente

## GeoRuggine Server

TO DO

## GeoRuggine Client

Questo manuale si pone come obiettivo di spiegare come utilizzare GeoRuggine, l'applicazione da terminale che simula il movimento di un mezzo, permette di comunicare con l'amministratore del sistema e consultare le proprie statistiche di percorso.

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

- **Tab**: sposta il focus tra i campi (evidenziato con un bordo giallo).
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

- **mario**: possiede già dei dati di tracciamento relativi all'ultimo mese, quindi interrogando le statistiche (periodo "oggi", "questa settimana" o "questo mese") restituisce risultati non nulli.
- **anna**: possiede solo l'account, senza alcun dato di tracciamento associato.

### 3. Schermata principale

Dopo l'accesso, la schermata si divide in due colonne e due barre infondo:

```
┌─────────────┬────────────────────┐
│ Utente      │ Broadcast          │
├─────────────┤                    │
│ Movimento   ├────────────────────┤
├─────────────┤ Chat               │
│ Periodo     │                    │
├─────────────┤                    │
│ Statistiche ├────────────────────┤
├─────────────┤ Scrivi messaggio   │
│ Elimina     │                    │
│ account     │                    │
├─────────────┴────────────────────┤
│ Errori                           │
├──────────────────────────────────┤
│ Barra di aiuto                   │
└──────────────────────────────────┘
```

Il riquadro con il focus attivo ha sempre il bordo giallo. Si passa da un riquadro all'altro con **Tab** (per avanzare, nell'ordine Utente → Movimento → Periodo statistiche → Statistiche → Elimina account → Broadcast → Chat → Scrivi messaggio → Errori) o **Shift+Tab** (per indietreggiare nello stesso ordine), che li scorre in sequenza tornando al primo dopo l'ultimo. **Esc**, da qualunque riquadro, chiude l'applicazione e, quindi, anche la connessione al server — **tranne** durante la procedura di eliminazione account già avviata (§3.5), dove annulla il passo corrente senza chiudere il programma.

#### 3.1 Utente

Mostra semplicemente il nome utente con cui hai effettuato l'accesso. È un riquadro di sola lettura.

#### 3.2 Stato movimento

Il client simula automaticamente, in background, il movimento di un mezzo lungo un percorso, inviando la posizione al server ogni 30 secondi circa. Questo riquadro mostra:

- **Stato**:
  - "In movimento" mentre gli invii proseguono;
  - "Fermo" se il mezzo è fermo nella stessa posizione da più di 3 minuti;
  - "Problema" (in rosso) se il mezzo ha smesso di inviare posizioni.
- **Primo invio** e **Ultimo invio**: orario del primo e dell'ultimo aggiornamento di posizione ricevuto.
- **Posizione**: le coordinate dell'ultimo punto inviato.

Questo riquadro si aggiorna in automatico con il movimento del mezzo e non richiede alcun tipo di interazione.

#### 3.3 Periodo statistiche

Permette di scegliere il periodo su cui interrogare le statistiche di percorso:

- **↑ / ↓**: cambia il periodo selezionato tra **Oggi**, **Questa settimana**, **Questo mese** (evidenziato in giallo).
- **Invio**: invia la richiesta al server per il periodo selezionato.

#### 3.4 Statistiche

Mostra il risultato dell'ultima richiesta fatta dal riquadro "Periodo statistiche":

- **"Nessuna richiesta ancora"**: non hai ancora interrogato le statistiche in questa sessione.
- **"In attesa della risposta dal server..."**: la richiesta è stata inviata, si aspetta la risposta.
  - In caso di successo: distanza totale percorsa, velocità media, tempo totale in movimento e tempo totale fermo, con l'orario a cui il dato è stato calcolato.
  - In caso di errore (es. problema nel recupero dei dati): il messaggio d'errore, con l'orario in cui si è verificato.

#### 3.5 Elimina account

Permette di cancellare **definitivamente** il proprio account e tutti i dati di tracciamento ad esso associati. La procedura è a più passi, per evitare eliminazioni accidentali:

1. **Invio** sul riquadro a riposo (mostra il testo "Invio per eliminare l'account"): avvia la procedura e mostra il campo password.
2. Digita la password (mostrata mascherata con `*`); **Backspace** cancella l'ultimo carattere. **Invio** con il campo non vuoto passa alla conferma; **Esc** annulla e torna al riquadro a riposo.
3. Compare la richiesta "Eliminare DAVVERO l'account? Azione irreversibile. (y/n)": premi **y** per confermare, oppure **n** o **Esc** per annullare e tornare al riquadro a riposo.
4. Dopo la conferma, mentre si attende la risposta del server compare "Eliminazione in corso..." e l'input viene ignorato.

Importante: durante l'intera procedura (dal passo 2 in poi) **Esc** annulla soltanto il passo corrente e **non chiude l'applicazione**, a differenza del comportamento di Esc su tutti gli altri riquadri.

Esiti possibili:

- **Password errata**: il server rifiuta la richiesta, il messaggio di errore compare sotto il campo password e puoi correggere e ritentare (si torna al passo 2 con il campo password vuoto).
- **Eliminazione riuscita**: la sessione si chiude e il client **torna automaticamente alla schermata di scelta Login/Registrazione**, come se fosse stato appena avviato — l'account e tutti i dati di tracciamento ad esso associati sono stati rimossi in modo permanente dal server, e potrai effettuare un nuovo accesso o una nuova registrazione senza dover riavviare manualmente il programma.

#### 3.6 Broadcast

Mostra gli annunci che l'amministratore invia a **tutti** gli utenti connessi, con orario di ricezione. È un riquadro di sola lettura, distinto dalla chat diretta.

- **↑ / ↓** (col focus su questo riquadro): scorre lo storico dei messaggi, utile quando sono troppi per stare tutti a schermo.

#### 3.7 Chat

Mostra la conversazione diretta tra te e l'amministratore:

- `>` indica un messaggio che hai inviato tu.
- `<` indica un messaggio ricevuto dall'amministratore.
- I messaggi di sistema (es. errori generali segnalati dal server) compaiono con il prefisso `[sistema]`, in rosso.
- Ogni riga riporta l'orario del messaggio.
- **↑ / ↓** (col focus su questo riquadro): scorre lo storico, come per il Broadcast.

#### 3.8 Scrivi messaggio

Il campo dove componi un messaggio da inviare all'amministratore:

- Digita normalmente; **Backspace** cancella l'ultimo carattere.
- **Invio**: invia il messaggio (se non è vuoto) — comparirà subito nel riquadro "Chat" preceduto da `>`.
- **↑ / ↓**: permette di scorrere il messaggio nel caso in cui sia più lungo di 2 righe.

#### 3.9 Errori

Riquadro a tutta larghezza, sotto le due colonne: mostra eventuali problemi tecnici locali del client (ad es. un aggiornamento di posizione o un messaggio che non è stato possibile inviare al server), in rosso e con l'orario in cui si sono verificati. È un riquadro di sola lettura, distinto sia dalla Chat sia dal Broadcast: non riguarda i messaggi dell'amministratore, ma il funzionamento interno del client stesso.

- **↑ / ↓** (col focus su questo riquadro): scorre lo storico degli errori, come per Chat e Broadcast.
- Se non si è mai verificato nessun problema, il riquadro resta vuoto.

#### 3.10 Barra di aiuto

In fondo allo schermo, mostra un promemoria dei tasti disponibili, che cambia in base al riquadro con il focus attivo (es. suggerisce le frecce e Invio quando sei su "Periodo statistiche", oppure ricorda come scrivere e inviare un messaggio quando sei su "Scrivi messaggio", oppure ricorda la procedura password+conferma quando sei su "Elimina account").

#### 3.11 Perdita di connessione

Se durante l'uso della schermata principale il server si arresta o la connessione di rete cade, il client mostra il messaggio "Connessione al server persa. Uscita dall'applicazione." e si chiude. Se succede, verifica che il server sia attivo e riavvia il client per accedere di nuovo.

### 4. Riepilogo dei tasti

| Tasto         | Effetto                                                                                                          |
| ------------- | ----------------------------------------------------------------------------------------------------------------- |
| **Tab**       | Sposta il focus al riquadro successivo                                                                          |
| **↑ / ↓**     | Cambia selezione (login/registrazione, periodo) o scorre lo storico (chat/broadcast/errori)                    |
| **Invio**     | Conferma la scelta, invia il modulo o il messaggio, oppure avanza di un passo nella procedura "Elimina account" |
| **y / n**     | Nel passo di conferma di "Elimina account": conferma (y) o annulla (n) l'eliminazione                           |
| **Backspace** | Cancella l'ultimo carattere digitato                                                                             |
| **Esc**       | Torna indietro (dai moduli di login/registrazione), annulla il passo corrente ("Elimina account"), oppure chiude l'applicazione |

### 5. Uscire dall'applicazione

Premi **Esc** da qualunque punto della schermata principale (o dalla schermata di scelta Login/Registrazione) per chiudere il client in modo pulito: la connessione con il server viene chiusa e il terminale torna al suo stato normale mostrando il messaggio "Uscita dall'applicazione. Disconnessione dal server...".

C'è un secondo modo, distinto dall'uscita, in cui la sessione principale termina: **eliminando il proprio account** (§3.5). In quel caso il client **non si chiude**, ma torna alla schermata di scelta Login/Registrazione, pronto per un nuovo accesso.

### 6. Problemi comuni

| Messaggio                                                                   | Quando compare                           | Cosa significa                                                                                                                                             |
| --------------------------------------------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| "Impossibile connettersi al server: ..."                                    | All'avvio                                | Il server non è raggiungibile: verifica che sia in esecuzione                                                                                              |
| "Username e password non possono essere vuoti."                             | Login/registrazione                      | Hai lasciato un campo vuoto                                                                                                                                |
| "Le password non corrispondono."                                            | Registrazione                            | Password e conferma sono diverse                                                                                                                           |
| "Credenziali non valide" / "Utente non trovato"                             | Login                                    | Username o password errati                                                                                                                                 |
| "Errore durante la registrazione: ..."                                      | Registrazione                            | L'username scelto potrebbe già esistere, o si è verificato un altro problema                                                                               |
| "[sistema] Il server si sta arrestando, verrai disconnesso."                | Chat, durante l'uso                      | L'amministratore ha fermato il server: la sessione terminerà a breve                                                                                       |
| Messaggi nel riquadro "Errori" (es. "Errore nell'invio del messaggio: ...") | Durante l'uso, riquadro Errori           | Un problema temporaneo locale (invio di un messaggio o di un aggiornamento di posizione non riuscito); se persiste, la connessione potrebbe cadere a breve |
| "Connessione al server persa. Uscita dall'applicazione."                    | Durante l'uso della schermata principale | Il server si è arrestato o la connessione di rete è caduta: il client si chiude, riavvialo quando il server è di nuovo raggiungibile                       |
| "Password errata."                                                          | Riquadro "Elimina account"               | La password inserita non corrisponde a quella dell'account: puoi correggerla e ritentare, oppure premere Esc per annullare                                 |

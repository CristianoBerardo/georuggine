# Manuale Utente

## GeoRuggine Server

Questo manuale spiega come utilizzare la console dell'operatore di GeoRuggine: l'interfaccia da terminale con cui si tengono sotto controllo gli utenti registrati, si osservano quelli attualmente collegati e si comunica con loro (in privato o in broadcast a tutti).

### 1. Avvio dell'applicazione

Il server si avvia da un'istanza del terminale con il seguente comando:
`cargo run -p server`

All'avvio, prima ancora che compaia l'interfaccia, il server stampa a schermo alcuni messaggi di inizializzazione ("Pool fatto", "Stato fatto", "Server in ascolto su 127.0.0.1:8080"): indicano che la connessione al database è stata stabilita e che il server è pronto ad accettare connessioni dai client. Subito dopo si entra direttamente nella schermata principale — a differenza del client, l'operatore non deve effettuare login.

Per una visualizzazione corretta è necessario un terminale di almeno 80 colonne per 29 righe: se lo spazio disponibile è inferiore, il server mostra un avviso al posto della schermata normale, finché non ingrandisci la finestra.

### 2. Schermata principale

La schermata si divide in due colonne, un riquadro a tutta larghezza e una barra in fondo:

```
┌────────────────────┬────────────────────────────┐
│ Utenti registrati  │ Selezione utenti collegati │
├────────────────────┼────────────────────────────┤
│ Broadcast chat     │ Chat                       │
│                    │                            │
├────────────────────┼────────────────────────────┤
│ Scrivi messaggio   │ Scrivi messaggio           │
│ Broadcast          │                            │
├────────────────────┴────────────────────────────┤
│ Errori del server                               │
├─────────────────────────────────────────────────┤
│ Barra di aiuto                                  │
└─────────────────────────────────────────────────┘
```

La colonna di sinistra riguarda la comunicazione broadcast (a tutti gli utenti connessi), quella di destra la comunicazione diretta con un singolo utente selezionato. Il riquadro con il focus attivo ha sempre il bordo giallo. Si passa da un riquadro all'altro con **Tab** (per avanzare) o **Shift+Tab** (per indietreggiare), che li scorre in sequenza tornando al primo dopo l'ultimo.

#### 2.1 Utenti registrati

Elenca **tutti** gli utenti registrati sul sistema, ciascuno con il proprio stato tra parentesi quadre: `[Sconnesso]`, `[Fermo]`, `[In movimento]`, `[Problema]` — lo stesso significato descritto nel manuale del client per il riquadro "Stato movimento". È un riquadro di sola lettura.

- **↑ / ↓** (col focus su questo riquadro): scorre l'elenco se supera lo spazio disponibile.

#### 2.2 Broadcast chat

Mostra la cronologia dei messaggi broadcast che hai inviato, con `>` e orario di invio. È un riquadro di sola lettura.

- **↑ / ↓** (col focus su questo riquadro): scorre lo storico.

#### 2.3 Scrivi messaggio Broadcast

Il campo dove componi un messaggio da inviare a **tutti** gli utenti attualmente connessi:

- Digita normalmente; **Backspace** cancella l'ultimo carattere.
- **Invio**: invia il messaggio a tutti i connessi (se non è vuoto) e lo aggiunge a "Broadcast chat".
- **↑ / ↓**: permette di scorrere il testo se è più lungo dello spazio visibile.

#### 2.4 Selezione utenti collegati

Elenca solo gli utenti **attualmente connessi**, cioè con un client GeoRuggine aperto in questo momento:

- **↑ / ↓** (col focus su questo riquadro): sposta la selezione tra gli utenti connessi (evidenziata in giallo), scorrendo in sequenza e tornando al primo dopo l'ultimo. La selezione determina a chi vengono inviati i messaggi scritti in "Scrivi messaggio" e di chi si vede la conversazione in "Chat".
- Se un utente non selezionato ti scrive un messaggio, accanto al suo nome compare il numero di messaggi non letti tra parentesi, in rosso (es. `mario (2)`); selezionandolo, i messaggi risultano letti e il contatore si azzera.
- La lista scorre automaticamente per tenere sempre visibile l'utente selezionato.

#### 2.5 Chat

Mostra la conversazione diretta con l'utente selezionato in "Selezione utenti collegati". Il titolo del riquadro riporta con chi stai parlando ("Chat con: nome") oppure "Nessun utente collegato selezionato" se non hai ancora scelto nessuno:

- `>` indica un messaggio che hai inviato tu.
- `<` indica un messaggio ricevuto dall'utente.
- Ogni riga riporta l'orario del messaggio.
- **↑ / ↓** (col focus su questo riquadro): scorre lo storico.

#### 2.6 Scrivi messaggio

Il campo dove componi un messaggio diretto per l'utente selezionato:

- Finché non hai selezionato nessun utente in "Selezione utenti collegati", il riquadro mostra "Seleziona un utente collegato..." e non puoi scrivere.
- Digita normalmente; **Backspace** cancella l'ultimo carattere.
- **Invio**: invia il messaggio all'utente selezionato (se non è vuoto) e compare subito in "Chat" preceduto da `>`.
- **↑ / ↓**: permette di scorrere il testo se è più lungo dello spazio visibile.

#### 2.7 Errori del server

Riquadro a tutta larghezza, sotto le due colonne: mostra eventuali problemi tecnici del server (ad es. un messaggio che non è stato possibile recapitare a un client, un errore del database durante un login, un problema di rete), in rosso e con l'orario in cui si sono verificati. È un riquadro di sola lettura.

- **↑ / ↓** (col focus su questo riquadro): scorre lo storico degli errori.
- Se non si è mai verificato nessun problema, il riquadro resta vuoto.

#### 2.8 Barra di aiuto

In fondo allo schermo, mostra un promemoria dei tasti disponibili, che cambia in base al riquadro con il focus attivo.

### 3. Riepilogo dei tasti

| Tasto         | Effetto                                                                 |
| ------------- | ----------------------------------------------------------------------- |
| **Tab**       | Sposta il focus al riquadro successivo (Shift+Tab per tornare indietro) |
| **↑ / ↓**     | Cambia la selezione (utenti collegati) o scorre lo storico/l'elenco     |
| **Invio**     | Invia il messaggio scritto (broadcast o diretto)                        |
| **Backspace** | Cancella l'ultimo carattere digitato                                    |
| **Esc**       | Avvia l'arresto del server (vedi sezione successiva)                    |

### 4. Arresto del server

Premi **Esc** da qualunque riquadro per avviare la chiusura ordinata del server:

1. A tutti gli utenti connessi vengono inviati tre messaggi broadcast in sequenza ("Il server si sta arrestando, verrai disconnesso in 3/2/1 secondi..."), a un secondo di distanza l'uno dall'altro, seguiti da un messaggio di saluto ("Alla prossima!").
2. Ogni connessione viene poi chiusa in modo ordinato: il client riceve un ultimo avviso e si disconnette da solo.
3. Il processo del server termina.

L'intera procedura richiede circa 4 secondi: durante questo intervallo la console dell'operatore resta bloccata e non risponde ad altri tasti.

### 5. Problemi comuni

| Messaggio (nel riquadro "Errori del server")                             | Quando compare                               | Cosa significa                                                                                                                                               |
| ------------------------------------------------------------------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| "Errore nella gestione della connessione: ..."                           | Durante l'uso, un client si disconnette male | La connessione con un client si è interrotta in modo anomalo                                                                                                 |
| "Errore DB durante login di ...: ..."                                    | Un client tenta il login                     | Problema nell'accesso al database durante l'autenticazione di quell'utente                                                                                   |
| "Errore durante l'invio del messaggio a ...: ..." / "... broadcast: ..." | Invio di un messaggio diretto o broadcast    | Il messaggio non è stato recapitato a quel client (probabilmente si è già disconnesso)                                                                       |
| "Errore nel server di rete: ..."                                         | Raramente, durante l'uso                     | Il server TCP ha smesso di accettare nuove connessioni: i client già collegati restano attivi, ma nessun altro potrà collegarsi finché non riavvii il server |
| "Utente ... non trovato nella mappa degli stati."                        | Aggiornamento di stato di un utente          | Incoerenza interna tra il database e la mappa di stato in memoria; non richiede un'azione immediata da parte tua                                             |

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

   Operazione completata. Disconnessione.
   ```

## Utenti

| Utente | Password     |
| ------ | ------------ |
| mario  | supersegreta |
| anna   | password     |

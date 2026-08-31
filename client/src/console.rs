use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

// Coordina le stampe a schermo con la lettura da tastiera: mentre l'utente
// sta scrivendo una riga, i messaggi che arrivano in modo asincrono
// vengono accodati invece di interrompere la riga in corso, e
// stampati tutti insieme non appena l'input corrente termina.
struct Console {
    typing: AtomicBool,               // Indica se l'utente sta scrivendo una riga
    pending: Mutex<VecDeque<String>>, // Coda di messagi
}

// Stato globale del console manager
static CONSOLE: OnceLock<Console> = OnceLock::new();

// Inizializzazione dello stato globale del console manager
fn console() -> &'static Console {
    CONSOLE.get_or_init(|| Console {
        typing: AtomicBool::new(false),
        pending: Mutex::new(VecDeque::new()),
    })
}

// Stampa subito `text`, oppure lo accoda se l'utente sta scrivendo.
// Ritorna `true` se è stato stampato subito (non accodato).
pub fn print_or_queue(text: String) -> bool {
    let c = console();
    if c.typing.load(Ordering::Acquire) {
        c.pending.lock().unwrap().push_back(text);
        false
    } else {
        println!("{}", text);
        true
    }
}

// Da chiamare subito prima di iniziare a leggere una riga da stdin.
// Cambia il valore del flag typing a true.
pub(crate) fn begin_typing() {
    console().typing.store(true, Ordering::Release);
}

// Da chiamare subito dopo che l'input corrente è terminato: stampa tutto
// ciò che nel frattempo è stato accodato.
// Cambia il valore del flag typing a false.
pub(crate) fn end_typing() {
    let c = console();
    c.typing.store(false, Ordering::Release);
    // drain() svuota la coda e ne prende il possesso, altrimenti non si potrebbe
    // iterare su di essa poich è protetta da un Mutex
    let queued: Vec<String> = c.pending.lock().unwrap().drain(..).collect();
    for text in queued {
        println!("{}", text);
    }
}

// Come `end_typing`, ma stampa prima `text` e solo dopo il resto della coda:
// da usare quando l'evento che chiude la protezione (es. la risposta a una
// richiesta esplicita) deve avere la precedenza su eventuali messaggi accodati
// nel frattempo.
pub(crate) fn end_typing_with(text: String) {
    let c = console();
    c.typing.store(false, Ordering::Release);
    println!("{}", text);
    let queued: Vec<String> = c.pending.lock().unwrap().drain(..).collect();
    for queued_text in queued {
        println!("{}", queued_text);
    }
}

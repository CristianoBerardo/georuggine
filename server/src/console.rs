use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

// Coordina le stampe a schermo con la lettura da tastiera: mentre l'operatore
// sta scrivendo una riga, i messaggi che arrivano in modo asincrono vengono
// accodati invece di interrompere la riga in corso, e stampati tutti insieme
// non appena l'input corrente termina.
struct Console {
    typing: AtomicBool,
    pending: Mutex<VecDeque<String>>,
}

static CONSOLE: OnceLock<Console> = OnceLock::new();

fn console() -> &'static Console {
    CONSOLE.get_or_init(|| Console {
        typing: AtomicBool::new(false),
        pending: Mutex::new(VecDeque::new()),
    })
}

// Stampa subito text, oppure lo accoda se l'operatore sta scrivendo.
// Ritorna true se è stato stampato subito (non accodato).
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
pub(crate) fn begin_typing() {
    console().typing.store(true, Ordering::Release);
}

// Da chiamare subito dopo che l'input corrente è terminato: stampa tutto
// ciò che nel frattempo è stato accodato.
pub(crate) fn end_typing() {
    let c = console();
    c.typing.store(false, Ordering::Release);
    let queued: Vec<String> = c.pending.lock().unwrap().drain(..).collect();
    for text in queued {
        println!("{}", text);
    }
}

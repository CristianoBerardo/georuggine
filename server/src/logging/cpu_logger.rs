use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

use chrono::Local;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, get_current_pid};
pub fn start_cpu_logger<P: AsRef<Path> + Send + 'static>(log_file_path: P) {
    thread::spawn(move || {
        let mut sys = System::new();
        let pid = get_current_pid().expect("Errore nel recupero PID");

        sys.refresh_cpu_usage();
        sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().with_cpu(),
        );

        let interval = Duration::from_secs(10);

        loop {
            thread::sleep(interval);

            sys.refresh_cpu_usage();

            sys.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[pid]),
                true,
                ProcessRefreshKind::nothing().with_cpu(),
            );

            let global_cpu = sys.global_cpu_usage();
            let process_cpu = sys.process(pid).map(|p| p.cpu_usage()).unwrap_or(0.0);
            let num_cores = sys.cpus().len().max(1) as f32;

            // sysinfo riporta 100% per cose, se si hanno 8 core sarebbero 800%
            // lo normalizzo
            let normalized_process_cpu = process_cpu / num_cores;

            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
            let log_entry = format!(
                "[{timestamp}] App CPU: {process_cpu:.2}% (Normalized: {normalized_process_cpu:.2}%) | Global System CPU: {global_cpu:.2}%\n"
            );

            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file_path)
            {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(log_entry.as_bytes()) {
                        eprintln!("Errore nella scrittura del log: {e}");
                    }
                    // Flush a ogni singola iterazione
                    if let Err(e) = file.flush() {
                        eprintln!("Errore nel flush del file: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("Errore nell'apertura del file durante il log: {e}");
                }
            }
        }
    });
}

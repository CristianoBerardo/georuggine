# Georuggine

Georuggine is a distributed client-server fleet management and tracking application written in Rust. The system tracks simulated vehicle movements, computes telemetry metrics such as distance and average speed, and provides bidirectional text communication between vehicle drivers and the central operator.

Built on top of the asynchronous **Tokio** runtime, Georuggine delivers high concurrency, low resource overhead, and terminal-based interfaces.

---

## Features

- **Fleet Tracking**: Clients stream simulated geographic coordinates to the server every 30 seconds.
- **State Detection**: Automatically switches user status between **Stopped** and **Moving** (transitions to stopped after 3 minutes without coordinate changes).
- **Movement Analytics**: Computes total distance, average speed, movement duration, and pause time over selectable periods (daily, weekly, or monthly).
- **Direct & Broadcast Chat**: Operators can message individual vehicles or broadcast announcements to the entire fleet; drivers can send messages directly to the server.
- **Terminal User Interface (TUI)**: Multi-panel terminal dashboards for both the operator and the client.
- **Resource Logging**: The server records CPU usage to a log file every 2 minutes to monitor performance.

---

## Architecture & Technologies

The workspace is organized into three distinct crates:

- **`server`**: Manages concurrent TCP connections, authentication, data persistence, and the operator dashboard.
- **`client`**: Reads route data to simulate movement, handles the TCP connection, and provides the driver interface.
- **`common`**: Contains shared data structures and network protocol models used by both server and client.

### Core Libraries

- **Rust & Tokio**: Non-blocking asynchronous runtime and channel-based task communication.
- **Ratatui & Crossterm**: Multi-panel terminal UI and keyboard event handling.
- **SQLite & SQLx**: Embedded relational database and asynchronous query execution.
- **Argon2**: Secure password hashing for account registration and authentication.

---

## Getting Started

### Prerequisites

- Rust (latest stable toolchain recommended)
- Cargo package manager

### Running the Application

Open two separate terminal windows from the repository root directory.

#### 1. Start the Server

```bash
cargo run -p server
```

The server initializes the SQLite database connection, launches the CPU logging task, binds the TCP port, and starts the operator dashboard.

#### 2. Start the Client

```bash
cargo run -p client
```

The client loads the route coordinates from a CSV file, connects to the server over TCP, and displays the authentication screen. Once logged in, the client begins transmitting telemetry data and enables the chat interface.

#### Simulation Details

Client vehicles simulate travel by reading a sequence of coordinates and timestamps from a CSV file. The client sends updates to the server at fixed 30-second intervals to emulate live GPS telemetry.

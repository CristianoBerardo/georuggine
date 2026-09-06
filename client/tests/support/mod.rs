use common::protocol::{ClientMessage, ServerMessage};
use std::future::Future;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

//Avvia connessione fittizzia e ritorna indirizzo e hanlde
pub async fn spawn_mock_server<F, Fut>(handler: F) -> (SocketAddr, JoinHandle<()>)
where
    F: FnOnce(BufReader<OwnedReadHalf>, OwnedWriteHalf) -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (reader, writer) = stream.into_split();
        handler(BufReader::new(reader), writer).await;
    });

    (addr, handle)
}

// Apre una connessione verso il server mock e ritorna un reader e writer per comunicare
pub async fn connect(addr: SocketAddr) -> (BufReader<OwnedReadHalf>, OwnedWriteHalf) {
    let stream = TcpStream::connect(addr).await.unwrap();
    let (reader, writer) = stream.into_split();
    (BufReader::new(reader), writer)
}

// Invia un `ServerMessage`
pub async fn send_server_msg(writer: &mut OwnedWriteHalf, msg: &ServerMessage) {
    let mut payload = serde_json::to_string(msg).unwrap();
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await.unwrap();
    writer.flush().await.unwrap();
}

// Legge una riga dal socket e la decodifica come `ServerMessage`.
pub async fn recv(reader: &mut BufReader<OwnedReadHalf>) -> ServerMessage {
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    serde_json::from_str(line.trim()).unwrap()
}

// Legge una riga dal socket e la decodifica come `ClientMessage`
pub async fn recv_client_msg(reader: &mut BufReader<OwnedReadHalf>) -> ClientMessage {
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    serde_json::from_str(line.trim()).unwrap()
}

// Inviamo messaggi malformati
pub async fn send_raw(writer: &mut OwnedWriteHalf, raw: &str) {
    let mut payload = raw.to_string();
    payload.push('\n');
    writer.write_all(payload.as_bytes()).await.unwrap();
    writer.flush().await.unwrap();
}

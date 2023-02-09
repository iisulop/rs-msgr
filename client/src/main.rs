use std::io;
use std::net::SocketAddr;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::spawn;
use tracing::trace_span;
use tracing::Level;
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::Subscriber;
use types::{deserialize_message, serialize_message, MsgrError};

#[tokio::main()]
async fn main() -> Result<(), MsgrError> {
    let subscriber = Subscriber::builder()
        .with_span_events(FmtSpan::ACTIVE)
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not initialize tracing");

    client().await?;

    Ok(())
}

async fn write_message(write: &mut OwnedWriteHalf, contents: String) -> Result<(), MsgrError> {
    debug!("Sending message \"{}\"", &contents);
    let buf = serialize_message(contents)?;
    Ok(write.write_all(&buf).await?)
}

async fn client() -> Result<(), MsgrError> {
    let addr = "127.0.0.1:34254".parse::<SocketAddr>().unwrap();
    let stream = TcpStream::connect(&addr).await.unwrap();
    let (read, mut write) = stream.into_split();
    spawn(async move { connection_listener(read).await });

    loop {
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        write_message(&mut write, buf).await?
    }
}

async fn connection_listener(mut read_socket: OwnedReadHalf) {
    let peer = read_socket.peer_addr();
    let connection_span = trace_span!("listening on connection", ?peer);
    let _enter = connection_span.enter();
    let mut buf = vec![0; 1024];
    loop {
        #[allow(unused_must_use)]
        match read_incoming_bytes(&mut read_socket, &mut buf).await {
            Ok(0) => {
                info!("Socket {:?} closed", read_socket.peer_addr());
                break;
            }
            Ok(n) => {
                trace!("Read {n} bytes from socket {:?}", read_socket.peer_addr());
                if let Ok(message) = deserialize_message(&buf[0..n]) {
                    println!("Got message:\n{:?}", message);
                } else {
                    error!("Could not deserialize message");
                }
                println!("{:?}", String::from_utf8_lossy(&buf[0..n]))
            }
            Err(e) => {
                warn! { %e, "Error processing socket {:?}", read_socket.peer_addr()}
                break;
            }
        }
    }
}

async fn read_incoming_bytes(
    read_socket: &mut OwnedReadHalf,
    buf: &mut [u8],
) -> Result<usize, MsgrError> {
    trace!("Processing incoming from {:?}", read_socket.peer_addr());
    let n = read_socket.read(buf).await?;
    Ok(n)
}

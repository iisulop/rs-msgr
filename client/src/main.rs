use std::io;
use std::net::SocketAddr;

use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedReadHalf;
use tokio::spawn;
use tracing::Level;
use tracing::trace_span;
use tracing::{info, warn, trace};
use tracing_subscriber::fmt::Subscriber;
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Error, Debug)]
enum ClientError {
    #[error("Cannot read from socket")]
    SocketReadError(#[from] std::io::Error),
}

#[tokio::main()]
async fn main() -> Result<(), ClientError> {
    let subscriber = Subscriber::builder()
        .with_span_events(FmtSpan::ACTIVE)
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not initialize tracing");

    client().await?;

    Ok(())
}

async fn client() -> Result<(), ClientError> {
    let addr = "127.0.0.1:34254".parse::<SocketAddr>().unwrap();
    let stream = TcpStream::connect(&addr).await.unwrap();
    let (read, mut write) = stream.into_split();
    spawn(async move {
        connection_listener(read).await
    });

    loop {
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        write.write_all(buf.as_bytes()).await?;
    }
}

async fn connection_listener(mut read_socket: OwnedReadHalf) {
    let peer = read_socket.peer_addr();
    let connection_span = trace_span!("listening on connection", ?peer);
    let _enter = connection_span.enter();
    let mut buf = vec![0; 1024];
    loop {
        #[allow(unused_must_use)]
        match process_incoming(&mut read_socket, &mut buf).await {
            Ok(0) => {
                info!("Socket {:?} closed", read_socket.peer_addr());
                break;
            }
            Ok(n) => {
                trace!("Read {n} bytes from socket {:?}", read_socket.peer_addr());
                println!("{:?}", String::from_utf8_lossy(&buf[0..n]))
            }
            Err(e) => {
                warn! { %e, "Error processing socket {:?}", read_socket.peer_addr()}
                break;
            }
        }
    }
}

async fn process_incoming(read_socket: &mut OwnedReadHalf, buf: &mut[u8]) -> Result<usize, ClientError> {
    trace!("Processing incoming from {:?}", read_socket.peer_addr());
    let n = read_socket.read(buf).await?;
    Ok(n)
}

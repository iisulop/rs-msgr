use std::net::SocketAddr;

use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::spawn;
use tracing::Level;
use tracing::trace_span;
use tracing::{debug, info, warn, trace};
use tracing_subscriber::fmt::Subscriber;
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Error, Debug)]
enum ServerError {
    #[error("Cannot read from socket")]
    SocketReadError(#[from] std::io::Error),
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ServerError> {
    let subscriber = Subscriber::builder()
        .with_span_events(FmtSpan::ACTIVE)
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Could not initialize tracing");
    server().await;

    Ok(())
}

async fn server() {
    let addr = "127.0.0.1:34254".parse::<SocketAddr>().unwrap();
    debug!("Starting listener on address {addr:#?}");
    let listener = TcpListener::bind(addr).await.unwrap();
    info!("Started listener on address {addr:#?}");
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        spawn(
            async move {
                connection_listener(socket).await
            }
        );
    }
}

async fn connection_listener(mut socket: TcpStream) {
    let peer = socket.peer_addr();
    let connection_span = trace_span!("listening on connection", ?peer);
    let _enter = connection_span.enter();
    let mut buf = vec![0; 1024];
    loop {
        #[allow(unused_must_use)]
        match process_incoming(&mut socket, &mut buf).await {
            Ok(0) => {
                info!("Socket {:?} closed", socket.peer_addr());
                break;
            }
            Ok(n) => {
                trace!("Read {n} bytes from socket {:?}", socket.peer_addr());
                println!("{:?}", String::from_utf8_lossy(&buf[0..n]))
            }
            Err(e) => {
                warn! { %e, "Error processing socket {:?}", socket.peer_addr()}
                break;
            }
        }
    }
}

async fn process_incoming(socket: &mut TcpStream, buf: &mut[u8]) -> Result<usize, ServerError> {
    trace!("Processing incoming from {:?}", socket.peer_addr());
    let n = socket.read(buf).await?;
    socket.write_all(&buf[0..n]).await?;
    Ok(n)
}

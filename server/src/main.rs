use std::net::SocketAddr;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::spawn;
use tracing::trace_span;
use tracing::Level;
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::Subscriber;
use types::{deserialize_message, serialize_message, MsgrError};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), MsgrError> {
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
        spawn(async move { connection_listener(socket).await });
    }
}

async fn write_message(socket: &mut TcpStream, contents: String) -> Result<(), MsgrError> {
    debug!("Sending message \"{}\"", &contents);
    let buf = serialize_message(contents)?;
    Ok(socket.write_all(&buf).await?)
}

async fn connection_listener(mut socket: TcpStream) {
    let peer = socket.peer_addr();
    let connection_span = trace_span!("listening on connection", ?peer);
    let _enter = connection_span.enter();
    let mut buf = vec![0; 1024];
    loop {
        #[allow(unused_must_use)]
        match read_incoming_bytes(&mut socket, &mut buf).await {
            Ok(0) => {
                info!("Socket {:?} closed", socket.peer_addr());
                break;
            }
            Ok(n) => {
                trace!("Read {n} bytes from socket {:?}", socket.peer_addr());
                match deserialize_message(&buf[0..n]) {
                    Ok(message) => {
                        println!("Got message:\n{:?}", message);
                        if let Err(err) = write_message(
                            &mut socket,
                            message
                                .content
                                .map(|c| c.contents)
                                .unwrap_or_else(|| String::from("")),
                        )
                        .await
                        {
                            error!("Could not send message: {:#?}", err);
                        }
                    }
                    Err(err) => error!("Could not deserialize message: {:#?}", err),
                };
            }
            Err(e) => {
                warn! { %e, "Error processing socket {:?}", socket.peer_addr()}
                break;
            }
        }
    }
}

async fn read_incoming_bytes(socket: &mut TcpStream, buf: &mut [u8]) -> Result<usize, MsgrError> {
    trace!("Processing incoming from {:?}", socket.peer_addr());
    let n = socket.read(buf).await?;
    Ok(n)
}

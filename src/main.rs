use std::env::args;
use std::error::Error;
use std::net::ToSocketAddrs;

use futures::{SinkExt, StreamExt};

use tokio::net::TcpStream;

use tokio_util::codec::Decoder;

use tokio_native_tls::native_tls::TlsConnector;

use mumble_protocol::control::msgs;
use mumble_protocol::control::ClientControlCodec;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let addr = args()
        .collect::<Vec<String>>()
        .get(1)
        .ok_or("give me host:port")?
        .to_socket_addrs()?
        .next()
        .ok_or("failed to resolve")?;

    let cx = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()?;
    let cx = tokio_native_tls::TlsConnector::from(cx);

    let socket = TcpStream::connect(&addr).await?;
    socket.set_nodelay(true)?;
    let socket = cx.connect("fake.com", socket).await?;

    let (mut sink, mut stream) = ClientControlCodec::new().framed(socket).split();

    sink.send({
        let mut msg = msgs::Authenticate::new();
        msg.set_username("test".into());
        msg.set_opus(true);
        msg.into()
    })
    .await?;

    while let Some(packet) = stream.next().await {
        let packet = packet.unwrap();
        eprintln!("{:?}", packet);
    }

    Ok(())
}

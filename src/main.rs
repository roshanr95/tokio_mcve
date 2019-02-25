use futures::future::lazy;
use futures::stream::Stream;
use futures::try_ready;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio;
use tokio::net::UdpSocket;
use tokio::prelude::future::poll_fn;
use tokio::prelude::Future;

extern crate env_logger;
extern crate log;

fn main() {
    env_logger::init();

    let mut sock = UdpSocket::bind(&SocketAddr::from_str("127.0.0.1:8000").expect("Parse error"))
        .expect("Bind error");

    let (mut tx, mut rx) = tokio::sync::mpsc::channel::<(Vec<u8>, SocketAddr)>(2000);

    tokio::run(lazy(move || {
        //----------------- This future works ----------------//
        // tokio::spawn(
        //     poll_fn(move || {
        //         let packet = vec![70; 10];
        //         let to = SocketAddr::from_str("127.0.0.1:8001").expect("Parse error");
        //         try_ready!(sock.poll_send_to(packet.as_slice(), &to));
        //         Ok(futures::Async::Ready(()))
        //     })
        //     .map_err(|e: tokio::io::Error| println!("Error: {:?}", e)),
        // );

        //----------------- This future doesn't ----------------//
        tokio::spawn(
            poll_fn(move || {
                match try_ready!(rx
                    .poll()
                    .map_err(|_e| tokio::io::Error::new(tokio::io::ErrorKind::Other, "Poll error")))
                {
                    Some((packet, to)) => {
                        println!(
                            "Rx: Received {} bytes for {}: {:?}",
                            packet.len(),
                            to,
                            packet.as_slice(),
                        );
                        try_ready!(sock.poll_send_to(packet.as_slice(), &to));
                        println!("Sent");
                    }
                    None => println!("Rx end"),
                }
                Ok(futures::Async::Ready(()))
            })
            .map_err(|e: tokio::io::Error| println!("Error: {:?}", e)),
        );

        //----------------- This future queues a packet ----------------//
        tokio::spawn(
            poll_fn(move || {
                try_ready!(tx.poll_ready());
                tx.try_send((
                    vec![70; 10],
                    SocketAddr::from_str("127.0.0.1:8001").expect("Parse error"),
                ))
                .expect("Send error");
                // Wait permanently so message channel doesn't get disconnected
                // Achieved differently in production
                Ok(futures::Async::NotReady)
            })
            .map_err(|e: tokio::sync::mpsc::error::SendError| println!("Error: {:?}", e)),
        );

        Ok(())
    }));
}

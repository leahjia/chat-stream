use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    // result type is impl feature, hence need await
    let listener = TcpListener::bind("localhost:8080").await.unwrap();

    // broadcast for all
    let (tx, rx) = broadcast::channel(10);

    // have multiple clients
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        let (tx, mut rx) = (tx.clone(), tx.subscribe());

        // move one client to its individual task
        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                // use case: multiple channels with shared state
                tokio::select! {
                    // in this case there are two tasks executing
                    res = reader.read_line(&mut line) => {
                        if res.unwrap() == 0 {
                            break;
                        }
                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    res = rx.recv() => {
                        let (msg, other_addr) = res.unwrap();
                        if addr != other_addr {
                            writer.write_all(&msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}

use com::{client::Client, proto::Command};
use tokio::spawn;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    let (client_handle, mut client_worker) = Client::connect("127.0.0.1:5000").await.unwrap();

    let cancellation_token = CancellationToken::new();

    let a= spawn(async move {
        client_worker.run(cancellation_token).await.unwrap();
    });

    client_handle.write_command_closure(Command::new(53), vec![1, 1, 1, 1], |value| {
        println!("Received reply: {:?}", value);
    }).await.unwrap();

    a.await.unwrap();
}

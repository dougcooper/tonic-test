pub mod pb {
    tonic::include_proto!("greet");
}

use std::time::Duration;
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use pb::greeter_client::GreeterClient;

use crate::pb::HelloRequest;

async fn streaming_hello(client: &mut GreeterClient<Channel>, num: usize)->Result<(), Box<dyn std::error::Error>> {
    let stream = client
        .say_hello(HelloRequest {
            name: "foo".into(),
        })
        .await?
        .into_inner();
    
    println!(" we have a stream");

    // stream is infinite - take just 5 elements and then disconnect
    let mut stream = stream.take(num);
    while let Some(item) = stream.next().await {
        println!("\treceived: {}", item.unwrap().message);
    }

    Ok(())
    // stream is droped here and the disconnect info is send to server
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:5001").await.unwrap();

    println!("Streaming echo:");
    streaming_hello(&mut client, 5).await?;
    tokio::time::sleep(Duration::from_secs(1)).await; //do not mess server println functions

    Ok(())
}

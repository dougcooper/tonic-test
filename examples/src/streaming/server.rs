pub mod pb {
    tonic::include_proto!("greet");
}

use futures::Stream;
use std::{net::ToSocketAddrs, pin::Pin, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tonic::{transport::Server, Request, Response, Status};
use tokio::time::{sleep};

use crate::pb::{HelloRequest,HelloReply};
use crate::pb::greeter_server::GreeterServer;

type HelloResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<HelloReply, Status>> + Send>>;

#[derive(Debug)]
pub struct HelloServer {}

#[tonic::async_trait]
impl pb::greeter_server::Greeter for HelloServer {

    type SayHelloStream = ResponseStream;

    async fn say_hello(
        &self,
        req: Request<HelloRequest>,
    ) -> HelloResult<Self::SayHelloStream> {
        println!("HelloServer::say_hello");
        println!("\tclient connected from: {:?}", req.remote_addr());

        // creating infinite stream with requested message
        let repeat = std::iter::repeat(HelloReply{
            message: req.into_inner().name,
        });
        let mut stream = Box::pin(tokio_stream::iter(repeat).throttle(Duration::from_millis(200)));

        // spawn and channel are required if you want handle "disconnect" functionality
        // the `out_stream` will not be polled after client disconnect
        let (tx, rx) = mpsc::channel(128);
        tokio::spawn(async move {
            sleep(Duration::from_secs(3600)).await;
            while let Some(item) = stream.next().await {
                match tx.send(Result::<_, Status>::Ok(item)).await {
                    Ok(_) => {
                        // item (server response) was queued to be send to client
                    }
                    Err(_item) => {
                        // output_stream was build from rx and both are dropped
                        break;
                    }
                }
            }
            println!("\tclient disconnected");
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::SayHelloStream
        ))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = HelloServer {};
    Server::builder()
        .add_service(GreeterServer::new(server))
        .serve("[::1]:5001".to_socket_addrs().unwrap().next().unwrap())
        .await
        .unwrap();

    Ok(())
}

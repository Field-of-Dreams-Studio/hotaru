//! Simple gRPC client to test the Hotaru gRPC server
//!
//! This client uses tonic to connect to our Hotaru gRPC server.

use tonic::Request;

// Generated protobuf types  
include!(concat!(env!("OUT_DIR"), "/helloworld.rs"));

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = greeter_client::GreeterClient::connect("http://127.0.0.1:50051").await?;

    let request = Request::new(HelloRequest {
        name: "Hotaru".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
use amqprs::{
    BasicProperties, Deliver,
    channel::{BasicAckArguments, BasicConsumeArguments},
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
};
use async_trait::async_trait;

struct MyConsumer;

#[async_trait]
impl AsyncConsumer for MyConsumer {
    async fn consume(
        &mut self,
        channel: &amqprs::channel::Channel,
        deliver: Deliver,
        _properties: BasicProperties,
        content: Vec<u8>,
    ) {
        println!("Received: {}", String::from_utf8_lossy(&content));

        // acknowledge message
        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .unwrap();
    }
}

#[tokio::main]
async fn main() {
    // connect
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "guest",
        "guest",
    ))
    .await
    .unwrap();

    // open channel
    let channel = connection.open_channel(None).await.unwrap();

    // consume from your existing queue
    channel
        .basic_consume(
            MyConsumer,
            BasicConsumeArguments::new("block_txs_queue", "consumer_tag"),
        )
        .await
        .unwrap();

    println!("Waiting for messages...");

    // keep alive
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

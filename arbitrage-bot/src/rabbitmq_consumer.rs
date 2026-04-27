use amqprs::{
    BasicProperties, Deliver,
    channel::{BasicAckArguments, BasicConsumeArguments, Channel},
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
};
use async_trait::async_trait;
use tokio::sync::mpsc;

pub struct ArbitrageConsumer {
    trigger_sender: mpsc::UnboundedSender<()>,
}

impl ArbitrageConsumer {
    pub fn new(trigger_sender: mpsc::UnboundedSender<()>) -> Self {
        Self { trigger_sender }
    }
}

#[async_trait]
impl AsyncConsumer for ArbitrageConsumer {
    async fn consume(
        &mut self,
        channel: &Channel,
        deliver: Deliver,
        _properties: BasicProperties,
        content: Vec<u8>,
    ) {
        println!("Received block: {}", String::from_utf8_lossy(&content));

        let _ = self.trigger_sender.send(());

        channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
            .unwrap();
    }
}

pub async fn setup_consumer(
    trigger_sender: mpsc::UnboundedSender<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "guest",
        "guest",
    ))
    .await?;

    let channel = connection.open_channel(None).await?;

    channel
        .basic_consume(
            ArbitrageConsumer::new(trigger_sender),
            BasicConsumeArguments::new("block_txs_queue", "arbitrage-bot"),
        )
        .await?;

    println!("Arbitrage bot listening for blocks...");
    Ok(())
}

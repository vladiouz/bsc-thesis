use amqprs::{
    BasicProperties, Deliver,
    callbacks::DefaultConnectionCallback,
    channel::{BasicAckArguments, BasicConsumeArguments, Channel},
    connection::{Connection, OpenConnectionArguments},
    consumer::AsyncConsumer,
};
use async_trait::async_trait;
use tokio::sync::mpsc;

pub struct ConsumerRuntime {
    _connection: Connection,
    _channel: Channel,
}

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

        if let Err(e) = self.trigger_sender.send(()) {
            eprintln!("Failed to enqueue arbitrage trigger: {}", e);
        }

        if let Err(e) = channel
            .basic_ack(BasicAckArguments::new(deliver.delivery_tag(), false))
            .await
        {
            eprintln!("Failed to ack RabbitMQ message: {}", e);
        }
    }
}

pub async fn setup_consumer(
    trigger_sender: mpsc::UnboundedSender<()>,
) -> Result<ConsumerRuntime, Box<dyn std::error::Error>> {
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "guest",
        "guest",
    ))
    .await?;
    connection
        .register_callback(DefaultConnectionCallback)
        .await?;

    let channel = connection.open_channel(None).await?;

    channel
        .basic_consume(
            ArbitrageConsumer::new(trigger_sender),
            BasicConsumeArguments::new("block_txs_queue", "arbitrage-bot"),
        )
        .await?;

    println!("Arbitrage bot listening for blocks...");
    Ok(ConsumerRuntime {
        _connection: connection,
        _channel: channel,
    })
}

# stop the gateway
cd mx-chain-observing-squad/mainnet
docker compose down

# stop the events notifier
cd ../../mx-chain-notifier-go
docker compose down

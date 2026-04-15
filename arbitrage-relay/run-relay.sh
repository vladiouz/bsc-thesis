# start the gateway
cd mx-chain-observing-squad/mainnet
docker-compose --env-file .env up -d

# start the events notifier
cd ../../mx-chain-notifier-go
docker-compose --env-file .env up -d

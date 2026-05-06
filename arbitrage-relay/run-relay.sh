#!/usr/bin/env bash

NETWORK="$1"

case "$NETWORK" in
  --devnet)
    export SHARD_1=1
    export DISPLAY_NAME_1="MyObservingSquad-1"
    export SQUAD_DIRECTORY="$HOME/MyObservingSquad-1"
    export OBSERVER_IMAGE="chain-observer-devnet:D1.11.0.6"
    ;;
    
  --mainnet)
    export SHARD_1=1
    export DISPLAY_NAME_1="MyObservingSquad-2"
    export SQUAD_DIRECTORY="$HOME/MyObservingSquad-2"
    export OBSERVER_IMAGE="chain-observer-mainnet:1.11.0.0"
    ;;

  *)
    echo "Usage: $0 --devnet | --mainnet"
    exit 1
    ;;
esac

STACK_FOLDER=${SQUAD_DIRECTORY}
KEYS_FOLDER=${STACK_FOLDER}/keys
BOOTSTRAP=false

if [[ "$2" == "--bootstrap" ]]; then
  BOOTSTRAP=true
fi

if [[ "${BOOTSTRAP}" == "true" ]]; then
  mkdir -p "${STACK_FOLDER}"/{proxy,node-1}/{config,logs}
  mkdir -p "${STACK_FOLDER}"/node-1/db
  mkdir -p "${KEYS_FOLDER}"

  docker run --rm \
    --mount type=bind,source="${KEYS_FOLDER}",destination=/keys \
    --workdir /keys \
    multiversx/chain-keygenerator:latest

  sudo chown "$(whoami)" "${KEYS_FOLDER}/validatorKey.pem"
  mv "${KEYS_FOLDER}/validatorKey.pem" "${STACK_FOLDER}/node-1/config/observerKey_1.pem"
fi

docker-compose up -d

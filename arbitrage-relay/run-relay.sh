#!/usr/bin/env bash

STACK_FOLDER=~/MyObservingSquad-1
KEYS_FOLDER=${STACK_FOLDER}/keys
BOOTSTRAP=false

if [[ "${1:-}" == "--bootstrap" ]]; then
  BOOTSTRAP=true
elif [[ $# -gt 0 ]]; then
  echo "Usage: $0 [--bootstrap]"
  exit 1
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

docker-compose --env-file .env up -d

#!/bin/bash
set -m

echo "[Snake-Online] starting SpacetimeDB instance"
spacetime start --non-interactive &

echo "waiting for startup to complete..."
until $(curl --output /dev/null --silent --head --fail http://localhost:3000/v1/identity/public-key); do
  sleep 1
done
echo "[Snake-Online] done!"

echo "[Snake-Online] publishing module to database"
spacetime publish --server local --anonymous --bin-path /app/snake_online_backend.wasm snake-online

echo "[Snake-Online] waiting for publish to complete..."
until $(curl --output /dev/null --silent --head --fail http://localhost:3000/v1/database/snake-online); do
  sleep 1
done
echo "[Snake-Online] done!"

echo "[Snake-Online] Resume SpacetimeDB instance"
fg

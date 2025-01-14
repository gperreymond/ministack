#!/bin/bash

set -e

cargo run -- --config tests/test-001.yaml
docker compose --file /tmp/ministack/cluster.yaml config
 
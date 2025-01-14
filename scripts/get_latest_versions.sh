#!/bin/bash

set -e

# Function to get the latest version of a project on GitHub
get_latest_version() {
    local repo=$1
    curl -s "https://api.github.com/repos/$repo/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

# Get the latest version of Consul
consul_version=$(get_latest_version "hashicorp/consul")
echo "[INFO] the latest version of Consul is: $consul_version"

# Get the latest version of Nomad
nomad_version=$(get_latest_version "hashicorp/nomad")
echo "[INFO] the latest version of Nomad is: $nomad_version"

# Get the latest version of Vault
vault_version=$(get_latest_version "hashicorp/vault")
echo "[INFO] the latest version of Vault is: $vault_version"

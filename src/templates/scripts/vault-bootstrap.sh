{%- set network_subnet = "10.1.0.0" -%}
{%- if network.subnet -%}{%- set network_subnet = network.subnet -%}{%- endif -%}
{%- set network_prefix = network_subnet | split(pat=".") | slice(start=0, end=3) | join(sep=".") -%}
{%- set vault_mode = "dev" -%}
{%- if services.vault.mode -%}{%- set vault_mode = services.vault.mode -%}{%- endif -%}
{%- set nomad_scheme = "http" -%}
{%- if services.nomad.config.tls -%}{%- set nomad_scheme = "https" -%}{%- endif -%}
{%- set root_token = "root" -%}
{%- if services.vault.config.root_token -%}{%- set root_token = services.vault.config.root_token -%}{%- endif -%}
{%- set raft = vault_mode == "raft" -%}
{%- set with_nomad = services.nomad.enabled and services.nomad.servers -%}
{%- if services.vault.enabled and (raft or with_nomad) -%}
#!/bin/bash

# runs at every start, every step is idempotent
{%- if raft %}
# - raft mode: init vault (first start), unseal it, create the '{{ root_token }}' token, enable kv v2 on secret/
{%- endif %}
{%- if with_nomad %}
# - configure vault for the nomad workload identities (jwt auth method "jwt-nomad",
#   role "nomad-workloads", policy on the kv "secret" mount)
{%- endif %}

set -euo pipefail

export VAULT_ADDR='http://{{ network_prefix }}.30:8200'
export VAULT_TOKEN='{{ root_token }}'
{%- if with_nomad %}
export NOMAD_ADDR='{{ nomad_scheme }}://{{ network_prefix }}.11:4646'
{%- if services.nomad.config.tls %}
export NOMAD_SKIP_VERIFY='true'
{%- endif %}
{%- endif %}

# call the vault api: vault_api <method> <path> [json body] [token]
vault_api() {
  local method=$1 path=$2 body=${3:-} token=${4:-$VAULT_TOKEN}
  if [ -n "$body" ]; then
    curl -sf -X "$method" -H "X-Vault-Token: ${token}" -d "$body" "${VAULT_ADDR}/v1/${path}"
  else
    curl -sf -X "$method" -H "X-Vault-Token: ${token}" "${VAULT_ADDR}/v1/${path}"
  fi
}
{%- if raft %}

KEYS_FILE=/ministack/keys/init.json

echo "[INFO] waiting for the vault api..."
until curl -s -o /dev/null "${VAULT_ADDR}/v1/sys/seal-status"; do sleep 1; done

# first start: initialize with a single unseal key, saved in the vault_keys volume
if [ "$(vault_api GET sys/init | jq -r .initialized)" != "true" ]; then
  echo "[INFO] initializing vault..."
  vault_api PUT sys/init '{"secret_shares": 1, "secret_threshold": 1}' > "${KEYS_FILE}.tmp"
  mv "${KEYS_FILE}.tmp" "$KEYS_FILE"
  chmod 600 "$KEYS_FILE"
fi

if [ ! -f "$KEYS_FILE" ]; then
  echo "[ERROR] vault is initialized but ${KEYS_FILE} is missing, the data can not be unsealed"
  echo "[ERROR] run 'ministack --purge' to start from scratch"
  exit 1
fi

if [ "$(vault_api GET sys/seal-status | jq -r .sealed)" == "true" ]; then
  echo "[INFO] unsealing vault..."
  vault_api PUT sys/unseal "$(jq -c '{key: .keys_base64[0]}' "$KEYS_FILE")" > /dev/null
fi

echo "[INFO] waiting for vault to be active..."
until curl -sf -o /dev/null "${VAULT_ADDR}/v1/sys/health"; do sleep 1; done

# a non expiring token with a known id, like in dev mode
if ! vault_api GET auth/token/lookup-self > /dev/null 2>&1; then
  echo "[INFO] creating the '${VAULT_TOKEN}' root token..."
  initial_root_token=$(jq -r .root_token "$KEYS_FILE")
  vault_api POST auth/token/create-orphan \
    "{\"id\": \"${VAULT_TOKEN}\", \"policies\": [\"root\"], \"display_name\": \"ministack\"}" \
    "$initial_root_token" > /dev/null
fi

if ! vault_api GET sys/mounts | jq -e '.data["secret/"]' > /dev/null; then
  echo "[INFO] enabling the kv v2 secrets engine on secret/..."
  vault_api POST sys/mounts/secret '{"type": "kv", "options": {"version": "2"}}' > /dev/null
fi
{%- endif %}
{%- if with_nomad %}

if vault_api GET sys/auth | jq -e '.data["jwt-nomad/"]' > /dev/null; then
  echo "[INFO] vault is already configured for nomad"
else
  echo "[INFO] waiting for a nomad leader..."
  until curl -sfk "${NOMAD_ADDR}/v1/status/leader" | grep -q ':'; do sleep 2; done

  echo "[INFO] configuring vault for nomad workload identities..."
  nomad setup vault -y -jwks-url "${NOMAD_ADDR}/.well-known/jwks.json"
fi
{%- endif %}
echo "[INFO] vault is ready"
{%- endif %}

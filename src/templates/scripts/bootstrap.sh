{%- set network_subnet = "10.1.0.0" -%}
{%- if network.subnet -%}{%- set network_subnet = network.subnet -%}{%- endif -%}
{%- set network_prefix = network_subnet | split(pat=".") | slice(start=0, end=3) | join(sep=".") -%}
{%- set nomad_scheme = "http" -%}
{%- if services.nomad.config.tls -%}{%- set nomad_scheme = "https" -%}{%- endif -%}
{%- set nomad_acl = false -%}
{%- if services.nomad.config.acl.enabled -%}{%- set nomad_acl = true -%}{%- endif -%}
{%- set nomad_token = "11111111-1111-1111-1111-111111111111" -%}
{%- if services.nomad.config.acl.token -%}{%- set nomad_token = services.nomad.config.acl.token -%}{%- endif -%}
{%- set consul_acl = false -%}
{%- if services.consul.enabled -%}{%- if services.consul.config.acl.enabled -%}{%- set consul_acl = true -%}{%- endif -%}{%- endif -%}
{%- set consul_token = "22222222-2222-2222-2222-222222222222" -%}
{%- if services.consul.config.acl.token -%}{%- set consul_token = services.consul.config.acl.token -%}{%- endif -%}
{%- set root_token = "root" -%}
{%- if services.vault.config.root_token -%}{%- set root_token = services.vault.config.root_token -%}{%- endif -%}
{%- set vault_mode = "dev" -%}
{%- if services.vault.mode -%}{%- set vault_mode = services.vault.mode -%}{%- endif -%}
{%- set with_nomad = false -%}
{%- if services.nomad.enabled -%}{%- if services.nomad.servers -%}{%- set with_nomad = true -%}{%- endif -%}{%- endif -%}
{%- set vault_raft = false -%}
{%- if services.vault.enabled -%}{%- if vault_mode == "raft" -%}{%- set vault_raft = true -%}{%- endif -%}{%- endif -%}
{%- set needs_bootstrap = vault_raft -%}
{%- if with_nomad -%}{%- if nomad_acl or consul_acl or services.vault.enabled -%}{%- set needs_bootstrap = true -%}{%- endif -%}{%- endif -%}
{%- if needs_bootstrap -%}
#!/bin/bash

# runs at every start, every step is idempotent

set -euo pipefail
{%- if with_nomad %}

export NOMAD_ADDR='{{ nomad_scheme }}://{{ network_prefix }}.11:4646'
{%- if services.nomad.config.tls %}
export NOMAD_SKIP_VERIFY='true'
{%- endif %}
{%- if nomad_acl %}
export NOMAD_TOKEN='{{ nomad_token }}'
{%- endif %}
{%- endif %}
{%- if services.consul.enabled and services.consul.servers %}
export CONSUL_HTTP_ADDR='http://{{ network_prefix }}.21:8500'
{%- if consul_acl %}
export CONSUL_HTTP_TOKEN='{{ consul_token }}'
{%- endif %}
{%- endif %}
{%- if services.vault.enabled %}
export VAULT_ADDR='http://{{ network_prefix }}.30:8200'
export VAULT_TOKEN='{{ root_token }}'

# call the vault api: vault_api <method> <path> [json body] [token]
vault_api() {
  local method=$1 path=$2 body=${3:-} token=${4:-$VAULT_TOKEN}
  if [ -n "$body" ]; then
    curl -sf -X "$method" -H "X-Vault-Token: ${token}" -d "$body" "${VAULT_ADDR}/v1/${path}"
  else
    curl -sf -X "$method" -H "X-Vault-Token: ${token}" "${VAULT_ADDR}/v1/${path}"
  fi
}
{%- endif %}
{%- if with_nomad %}

echo "[INFO] waiting for a nomad leader..."
until curl -sfk "${NOMAD_ADDR}/v1/status/leader" | grep -q ':'; do sleep 2; done
{%- endif %}
{%- if with_nomad and nomad_acl %}

# ---------------------------------------------------------------- nomad acl
if nomad acl token self > /dev/null 2>&1; then
  echo "[INFO] nomad acl already bootstrapped"
else
  echo "[INFO] bootstrapping nomad acl with the management token..."
  echo "${NOMAD_TOKEN}" | nomad acl bootstrap - > /dev/null
fi
{%- endif %}
{%- if with_nomad and consul_acl %}

# ---------------------------------------------------------------- consul for nomad
if consul acl auth-method list -format=json | jq -e '.[] | select(.Name == "nomad-workloads")' > /dev/null; then
  echo "[INFO] consul is already configured for nomad"
else
  echo "[INFO] configuring consul for nomad workload identities..."
  nomad setup consul -y -jwks-url "${NOMAD_ADDR}/.well-known/jwks.json"
fi

# local stack: allow all service mesh traffic, like without acl
echo "[INFO] allowing all service mesh traffic (intentions * -> *)..."
consul config write - > /dev/null <<'EOF'
Kind = "service-intentions"
Name = "*"
Sources = [
  {
    Name   = "*"
    Action = "allow"
  }
]
EOF
{%- endif %}
{%- if vault_raft %}

# ---------------------------------------------------------------- vault raft
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
{%- if with_nomad and services.vault.enabled %}

# ---------------------------------------------------------------- vault for nomad
if vault_api GET sys/auth | jq -e '.data["jwt-nomad/"]' > /dev/null; then
  echo "[INFO] vault is already configured for nomad"
else
  echo "[INFO] configuring vault for nomad workload identities..."
  nomad setup vault -y -jwks-url "${NOMAD_ADDR}/.well-known/jwks.json"
fi
{%- endif %}

echo "[INFO] cluster is bootstrapped"
{%- endif %}

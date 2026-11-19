{%- set network_subnet = "10.1.0.0" -%}
{%- if network.subnet -%}{%- set network_subnet = network.subnet -%}{%- endif -%}
{%- set network_prefix = network_subnet | split(pat=".") | slice(start=0, end=3) | join(sep=".") -%}
{%- set nomad_scheme = "http" -%}
{%- if services.nomad.config.tls -%}{%- set nomad_scheme = "https" -%}{%- endif -%}
{%- set root_token = "root" -%}
{%- if services.vault.config.root_token -%}{%- set root_token = services.vault.config.root_token -%}{%- endif -%}
{%- if services.vault.enabled and services.nomad.enabled and services.nomad.servers -%}
#!/bin/bash

# configure vault for the nomad workload identities (jwt auth method "jwt-nomad",
# role "nomad-workloads", policy on the kv "secret" mount), vault is in dev mode
# so this runs at every start

set -euo pipefail

export NOMAD_ADDR='{{ nomad_scheme }}://{{ network_prefix }}.11:4646'
export VAULT_ADDR='http://{{ network_prefix }}.30:8200'
export VAULT_TOKEN='{{ root_token }}'
{%- if services.nomad.config.tls %}
export NOMAD_SKIP_VERIFY='true'
{%- endif %}

echo "[INFO] waiting for a nomad leader..."
until curl -sfk "${NOMAD_ADDR}/v1/status/leader" | grep -q ':'; do sleep 2; done

echo "[INFO] configuring vault for nomad workload identities..."
nomad setup vault -y -jwks-url "${NOMAD_ADDR}/.well-known/jwks.json"
echo "[INFO] vault is ready for nomad"
{%- endif %}

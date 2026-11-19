{%- set nomad_tls = false -%}
{%- if services.nomad.enabled -%}{%- if services.nomad.config.tls -%}{%- set nomad_tls = true -%}{%- endif -%}{%- endif -%}
{%- set consul_tls = false -%}
{%- if services.consul.enabled -%}{%- if services.consul.config.tls -%}{%- set consul_tls = true -%}{%- endif -%}{%- endif -%}
{%- set vault_tls = false -%}
{%- if services.vault.enabled -%}{%- if services.vault.config.tls -%}{%- set vault_tls = true -%}{%- endif -%}{%- endif -%}
{%- set any_tls = nomad_tls or consul_tls or vault_tls -%}
{%- set nomad_scheme = "http" -%}{%- if nomad_tls -%}{%- set nomad_scheme = "https" -%}{%- endif -%}
{%- set consul_scheme = "http" -%}{%- set consul_port = 8500 -%}
{%- if consul_tls -%}{%- set consul_scheme = "https" -%}{%- set consul_port = 8501 -%}{%- endif -%}
{%- set vault_scheme = "http" -%}{%- if vault_tls -%}{%- set vault_scheme = "https" -%}{%- endif -%}
{%- set network_subnet = "10.1.0.0" -%}
{%- if network.subnet -%}{%- set network_subnet = network.subnet -%}{%- endif -%}
{%- set network_prefix = network_subnet | split(pat=".") | slice(start=0, end=3) | join(sep=".") -%}
{%- set vault_mode = "dev" -%}
{%- if services.vault.mode -%}{%- set vault_mode = services.vault.mode -%}{%- endif -%}
{%- set raft = vault_mode == "raft" -%}
{%- if services.vault.enabled and (raft or vault_tls) -%}
{%- if raft %}
# vault with persistent raft storage, initialized and unsealed by the bootstrap service
ui            = true
disable_mlock = true

storage "raft" {
  path    = "/vault/file"
  node_id = "{{ datacenter }}-vault"
}

api_addr     = "{{ vault_scheme }}://{{ network_prefix }}.30:8200"
cluster_addr = "https://{{ network_prefix }}.30:8201"
{%- endif %}

listener "tcp" {
  address = "0.0.0.0:8200"
  {%- if vault_tls %}
  tls_cert_file = "/ministack/tls/vault.pem"
  tls_key_file  = "/ministack/tls/vault-key.pem"
  {%- else %}
  tls_disable   = true
  {%- endif %}
}
{%- endif %}

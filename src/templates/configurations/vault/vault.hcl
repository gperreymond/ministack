{%- set network_subnet = "10.1.0.0" -%}
{%- if network.subnet -%}{%- set network_subnet = network.subnet -%}{%- endif -%}
{%- set network_prefix = network_subnet | split(pat=".") | slice(start=0, end=3) | join(sep=".") -%}
{%- set vault_mode = "dev" -%}
{%- if services.vault.mode -%}{%- set vault_mode = services.vault.mode -%}{%- endif -%}
{%- if services.vault.enabled and vault_mode == "raft" -%}
# vault with persistent raft storage, initialized and unsealed by the bootstrap service
ui            = true
disable_mlock = true

storage "raft" {
  path    = "/vault/file"
  node_id = "{{ datacenter }}-vault"
}

listener "tcp" {
  address     = "0.0.0.0:8200"
  tls_disable = true
}

api_addr     = "http://{{ network_prefix }}.30:8200"
cluster_addr = "http://{{ network_prefix }}.30:8201"
{%- endif %}

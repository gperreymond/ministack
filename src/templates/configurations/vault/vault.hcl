{%- if services.vault.enabled and (vault_raft or vault_tls) -%}
{%- if vault_raft %}
# vault with persistent raft storage, initialized and unsealed by the bootstrap service
ui            = true
disable_mlock = true

storage "raft" {
  path    = "/vault/file"
  node_id = "{{ datacenter }}-vault"
}

api_addr     = "{{ vault_scheme }}://{{ vault_ip }}:8200"
cluster_addr = "https://{{ vault_ip }}:8201"
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

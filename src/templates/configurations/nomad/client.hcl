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
{%- if 
  services.nomad.enabled and 
  services.nomad.clients
%}
{%- set retry_join = "nomad" -%}
{%- if  services.nomad.config.client.retry_join -%}{%- set retry_join = services.nomad.config.client.retry_join -%}{%- endif -%}

server {
  enabled = false
}

client {
  enabled = true
  server_join {
    retry_max = 10
    retry_interval = "30s"
    {%- if retry_join == "nomad" %}
    retry_join = [
      {%- for item in services.nomad.servers %}
      "{{ datacenter }}-{{ item.name }}",
      {%- endfor %}
    ]
    {%- else %}
    retry_join = [
      {%- for item in retry_join %}
      "{{ item }}",
      {%- endfor %}
    ]
    {%- endif %}
  }
  template {
    disable_file_sandbox = true
  }
}

{%- if services.vault.enabled %}
vault {
  enabled = true
  address = "{{ vault_scheme }}://{{ network_prefix }}.30:8200"
  {%- if vault_tls %}
  ca_file = "/ministack/tls/ca.pem"
  {%- endif %}
}

{% endif -%}
{%- if nomad_tls %}

tls {
  http = true
  rpc  = true

  ca_file   = "/ministack/tls/ca.pem"
  cert_file = "/ministack/tls/nomad-client.pem"
  key_file  = "/ministack/tls/nomad-client-key.pem"

  verify_server_hostname = true
  verify_https_client    = false
}
{%- endif %}

plugin "docker" {
  config {
    allow_privileged = true
    allow_caps = ["all"]
    volumes {
      enabled = true
    }
    extra_labels = ["job_name", "task_group_name", "task_name", "namespace", "node_name"]
  }
}
{%- endif %}
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
  address = "{{ vault_scheme }}://{{ vault_ip }}:8200"
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
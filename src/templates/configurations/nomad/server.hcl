{%- if 
  services.nomad.enabled and 
  services.nomad.servers
%}

{%- set bootstrap_expect = 1 -%}
{%- if  services.nomad.config.server.bootstrap_expect -%}{%- set bootstrap_expect = services.nomad.config.server.bootstrap_expect -%}{%- endif -%}
{%- set retry_join = "nomad" -%}
{%- if  services.nomad.config.server.retry_join -%}{%- set retry_join = services.nomad.config.server.retry_join -%}{%- endif -%}

server {
  enabled = true
  bootstrap_expect = {{ bootstrap_expect }}
  default_scheduler_config {
    memory_oversubscription_enabled = true
  }
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
}

client {
  enabled = false
}

autopilot {
  cleanup_dead_servers = true
}
{%- if nomad_tls %}

tls {
  http = true
  rpc  = true

  ca_file   = "/ministack/tls/ca.pem"
  cert_file = "/ministack/tls/nomad-server.pem"
  key_file  = "/ministack/tls/nomad-server-key.pem"

  verify_server_hostname = true
  verify_https_client    = false
}
{%- endif %}
{%- if services.vault.enabled %}

# vault, through the nomad workload identities (configured by the bootstrap service)
vault {
  enabled = true
  address = "{{ vault_scheme }}://{{ vault_ip }}:8200"
  {%- if vault_tls %}
  ca_file = "/ministack/tls/ca.pem"
  {%- endif %}
  default_identity {
    aud = ["vault.io"]
    ttl = "1h"
  }
}
{%- endif %}
{%- if consul_acl %}

# consul, through the nomad workload identities (configured by the bootstrap service)
consul {
  service_identity {
    aud = ["consul.io"]
    ttl = "1h"
  }
  task_identity {
    aud = ["consul.io"]
    ttl = "1h"
  }
}
{%- endif %}
{%- endif %}

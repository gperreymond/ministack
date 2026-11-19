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
{%- set nomad_acl = false -%}
{%- if services.nomad.config.acl.enabled -%}{%- set nomad_acl = true -%}{%- endif -%}
{%- set nomad_token = "11111111-1111-1111-1111-111111111111" -%}
{%- if services.nomad.config.acl.token -%}{%- set nomad_token = services.nomad.config.acl.token -%}{%- endif -%}
{%- set consul_acl = false -%}
{%- if services.consul.enabled -%}{%- if services.consul.config.acl.enabled -%}{%- set consul_acl = true -%}{%- endif -%}{%- endif -%}
{%- set consul_token = "22222222-2222-2222-2222-222222222222" -%}
{%- if services.consul.config.acl.token -%}{%- set consul_token = services.consul.config.acl.token -%}{%- endif -%}
{%- set network_subnet = "10.1.0.0" -%}
{%- if network.subnet -%}{%- set network_subnet = network.subnet -%}{%- endif -%}
{%- set network_prefix = network_subnet | split(pat=".") | slice(start=0, end=3) | join(sep=".") -%}
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
  address = "{{ vault_scheme }}://{{ network_prefix }}.30:8200"
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

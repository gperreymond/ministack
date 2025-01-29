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
    retry_max = 3
    retry_interval = "15s"
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
{%- endif %}

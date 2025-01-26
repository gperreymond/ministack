{%- if services.nomad.enabled %}
{%- set replicas = 1 -%}
{%- if services.nomad.bootstrap_expect -%}
{%- set replicas = services.nomad.bootstrap_expect -%}
{%- endif -%}
{%- if services.consul.enabled %}
consul {}
{%- endif %}

server {
  enabled = true
  bootstrap_expect = {{ replicas }}
  default_scheduler_config {
    memory_oversubscription_enabled = true
  }
  {%- if not services.consul.enabled %}
  server_join {
    retry_max = 3
    retry_interval = "15s"
    {% if services.nomad.customized.retry_join %}
    retry_join = [
      {%- for item in services.nomad.customized.retry_join %}
      "{{ item }}",
      {%- endfor %}
    ]
    {%- else %}
    retry_join = [
      {%- for i in range(start=1, end=replicas+1) %}
      "nomad-server-{{ i }}",
      {%- endfor %}
    ]
    {%- endif %}
  }
  {%- endif %}
}

client {
  enabled = false
}

autopilot {
  cleanup_dead_servers = true
}
{%- endif %}

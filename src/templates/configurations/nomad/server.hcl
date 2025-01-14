{%- if services.nomad.enabled %}
{%- set replicas = 1 -%}
{%- if services.nomad.replicas -%}
{%- set replicas = services.nomad.replicas -%}
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
    retry_join = [
      {%- for i in range(start=1, end=replicas+1) %}
      "nomad-server-{{ i }}",
      {%- endfor %}
    ]
  }
  {%- endif %}
}

client {
  enabled = false
}

autopilot {
  cleanup_dead_servers = true
}

telemetry {
  publish_allocation_metrics = true
  publish_node_metrics       = true
}
{%- endif %}

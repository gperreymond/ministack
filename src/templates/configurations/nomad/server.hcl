{%- if services.nomad.enabled %}
{%- if services.consul.enabled %}
consul {}
{%- endif %}

server {
  enabled = true
  bootstrap_expect = {{ services.nomad.replicas }}
  {%- if not services.consul.enabled %}
  server_join {
    retry_max = 3
    retry_interval = "15s"
    retry_join = [
      {%- for i in range(start=1, end=services.nomad.replicas+1) %}
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

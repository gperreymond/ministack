{%- if services.nomad.enabled %}
{%- if services.consul.enabled %}
consul {
  address = "consul-server-1:8500"
}
{%- else %}
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

server {
  enabled = true
  bootstrap_expect = {{ services.nomad.replicas }}
}

client {
  enabled = false
}

autopilot {
  cleanup_dead_servers = true
}
{%- endif %}

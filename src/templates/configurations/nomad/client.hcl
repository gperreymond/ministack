{%- if services.nomad.enabled %}
server {
  enabled = false
}

{%- if services.consul.enabled %}
consul {
  address = "consul-server-1:8500"
  grpc_address = "consul-server-1:8502"
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

client {
  enabled = true
  template {
    disable_file_sandbox = true
  }
}

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
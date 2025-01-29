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
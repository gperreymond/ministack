{%- if services.nomad.enabled %}
{%- set replicas = 1 -%}
{%- if services.nomad.bootstrap_expect -%}
{%- set replicas = services.nomad.bootstrap_expect -%}
{%- endif -%}
server {
  enabled = false
}

{%- if services.consul.enabled %}
consul {}
{%- endif %}

client {
  enabled = true
  {%- if plugins.netbird.enabled and services.nomad.netbird.extra_envs %}
  network_interface = "{{ `{{ GetPrivateInterfaces | include \"network\" \"100.64.0.0/10\" | attr \"name\" }}` }}"
  {%- else %}
  network_interface = "{{ `{{ GetPrivateInterfaces | include \"network\" \"10.1.0.0/24\" | attr \"name\" }}` }}"
  {%- endif %}
  {%- if not services.consul.enabled %}
  servers = [
    {%- for i in range(start=1, end=replicas+1) %}
    "nomad-server-{{ i }}:4647",
    {%- endfor %}
  ]
  {%- endif %}
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
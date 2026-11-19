{%- if services.consul.enabled -%}
# consul client agent, running next to every nomad agent
server = false
{%- endif %}

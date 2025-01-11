{%- if services.consul.enabled %}
server = false

connect {
  enabled = true
}
{%- endif %}

{%- if services.consul.enabled %}
datacenter = "{{ datacenter }}"
data_dir = "/consul/data"
log_level = "{%- if log_level -%}{{ log_level }}{%- else -%}INFO{%- endif -%}"
log_json = true

ports {
  grpc     = 8502
  grpc_tls = -1
}

tls {
  grpc {
    use_auto_cert = false
  }
}

connect {
  enabled = true
}

dns_config {
  allow_stale   = true
  node_ttl      = "5s"
  use_cache     = true
  cache_max_age = "5s"
}

retry_join = [
  {%- for i in range(start=1, end=services.consul.replicas+1) %}
  "consul-server-{{ i }}",
  {%- endfor %}
]

bind_addr   = "{{ `{{ GetPrivateInterfaces | include \"network\" \"10.1.0.0/24\" | attr \"address\" }}` }}"
client_addr = "{{ `{{ GetPrivateInterfaces | exclude \"name\" \"docker.*\" | join \"address\" \" \" }} {{ GetAllInterfaces | include \"flags\" \"loopback\" | join \"address\" \" \" }}` }}"
{%- endif %}
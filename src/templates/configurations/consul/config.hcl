{%- if services.consul.enabled %}
{%- set replicas = 1 -%}
{%- if services.consul.bootstrap_expect -%}
{%- set replicas = services.consul.bootstrap_expect -%}
{%- endif -%}
{%- set log_level = "info" -%}
{%- if services.consul.log_level -%}
{%- set log_level = services.consul.log_level -%}
{%- endif -%}
datacenter = "{{ datacenter }}"
data_dir = "/consul/data"
log_level = "{{ log_level }}"
log_json = true

ports {
  https = 8501 
  grpc = 8502
  grpc_tls = -1
}

tls {
  defaults {
    verify_incoming = false
    verify_outgoing = false
    verify_server_hostname = false
  }
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
  {%- for i in range(start=1, end=replicas+1) %}
  "consul-server-{{ i }}",
  {%- endfor %}
]

bind_addr   = "{{ `{{ GetPrivateInterfaces | include \"network\" \"10.1.0.0/24\" | attr \"address\" }}` }}"
client_addr = "{{ `{{ GetPrivateInterfaces | exclude \"name\" \"docker.*\" | join \"address\" \" \" }} {{ GetAllInterfaces | include \"flags\" \"loopback\" | join \"address\" \" \" }}` }}"
{%- endif %}
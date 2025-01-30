job "prometheus-reloader" {
  datacenters = ["europe-paris"]
  type        = "batch"

  periodic {
    crons            = ["* * * * *"]
    prohibit_overlap = true
  }

  group "prometheus-reloader" {
    task "prometheus-reloader" {
      driver = "docker"

      config {
        image = "alpine/curl:8.11.1"
        args  = ["sh", "-c", "curl -s -X POST \"$PROMETHEUS_RELOAD_URL\""]
      }

      template {
        data        = <<-EOF
{{- range nomadService "prometheus" }}
PROMETHEUS_RELOAD_URL="http://{{ .Address }}:{{ .Port }}/-/reload"
{{- end }}
EOF
        env         = true
        destination = "local/.env"
      }

      resources {
        cpu    = 50
        memory = 64
      }

      logs {
        max_files     = 1
        max_file_size = 5
      }
    }
  }
}

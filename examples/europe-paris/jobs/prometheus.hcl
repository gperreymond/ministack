job "prometheus" {
  datacenters = ["europe-paris"]
  type        = "service"

  constraint {
    attribute = "${attr.unique.hostname}"
    value     = "europe-paris-worker-monitoring"
  }

  group "prometheus" {
    count = 1

    network {
      mode = "bridge"
      port "http" {
        to = 9090
      }
    }

    task "prometheus" {
      driver = "docker"
      user   = "root"

      config {
        image      = "prom/prometheus:v3.1.0"
        privileged = true
        ports      = ["http"]
        volumes = [
          "local/prometheus.yml:/etc/prometheus/prometheus.yml",
          "/mnt/prometheus_data:/prometheus"
        ]
      }

      template {
        data        = <<-EOF
global:
  evaluation_interval: 1m
  scrape_interval: 1m
  scrape_timeout: 10s
EOF
        destination = "local/prometheus.yml"
      }

      resources {
        cpu    = 500
        memory = 512
      }

      service {
        provider = "nomad"
        name     = "prometheus"
        port     = "http"
        tags     = ["metrics", "monitoring"]

        check {
          type     = "http"
          path     = "/-/ready"
          interval = "10s"
          timeout  = "2s"
        }
      }
    }
  }
}

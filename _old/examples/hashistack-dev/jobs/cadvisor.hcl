job "cadvisor" {
  datacenters = ["*"]
  type        = "system"
  update {
    stagger      = "30s"
    max_parallel = 1
  }
  group "cadvisor" {
    count = 1
    network {
      mode = "bridge"
      port "http" { to = 8080 }
    }
    service {
      provider = "consul"
      name     = "cadvisor"
      port     = "http"
      tags = [
        "metrics=cadvisor",
      ]
      check {
        type     = "http"
        path     = "/metrics/"
        interval = "10s"
        timeout  = "2s"
      }
    }
    task "cadvisor" {
      driver = "docker"
      config {
        image      = "gcr.io/cadvisor/cadvisor:v0.49.2"
        privileged = true
        volumes = [
          "/:/rootfs:ro",
          "/var/run:/var/run:rw",
          "/sys:/sys:ro",
          "/var/lib/docker/:/var/lib/docker:ro",
          "/cgroup:/cgroup:ro"
        ]
        ports = ["http"]
        logging {
          type = "journald"
          config {
            tag = "CADVISOR"
          }
        }
      }
      resources {
        cpu    = 50
        memory = 100
      }
    }
  }
}

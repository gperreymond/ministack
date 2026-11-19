# a web service in bridge mode, registered in the nomad service discovery
# usage:
#   nomad job run examples/nomad/minimum/jobs/whoami.hcl
#   nomad service info whoami
job "whoami" {
  # no datacenters: all of them (nomad >= 1.5), works on every example

  group "whoami" {
    network {
      mode = "bridge"

      port "http" {
        to = 80
      }
    }

    service {
      provider = "nomad"
      name     = "whoami"
      port     = "http"

      check {
        type     = "http"
        path     = "/"
        interval = "5s"
        timeout  = "2s"
      }
    }

    task "whoami" {
      driver = "docker"

      config {
        image = "traefik/whoami:v1.10.3"
        ports = ["http"]
      }

      resources {
        cpu    = 50
        memory = 32
      }
    }
  }
}

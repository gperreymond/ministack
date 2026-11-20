# a tcp service, exposed by the tcp ingress (examples/hashistack: 'ingress.tcp.redis: 6379')
# usage:
#   nomad job run examples/nomad/minimum/jobs/redis.hcl
#   redis-cli -p 6379 ping
job "redis" {
  # no datacenters: all of them (nomad >= 1.5), works on every example

  group "redis" {
    network {
      mode = "bridge"

      port "redis" {
        to = 6379
      }
    }

    service {
      provider = "nomad"
      name     = "redis"
      port     = "redis"

      # traefik only routes tcp when asked: the router listens on the entrypoint 'redis'
      # of the ingress, and takes every connection (no tls, so no server name to match)
      tags = [
        "traefik.tcp.routers.redis.entrypoints=redis",
        "traefik.tcp.routers.redis.rule=HostSNI(`*`)",
      ]

      check {
        type     = "tcp"
        interval = "5s"
        timeout  = "2s"
      }
    }

    task "redis" {
      driver = "docker"

      config {
        image = "redis:7.4-alpine"
        ports = ["redis"]
      }

      resources {
        cpu    = 50
        memory = 64
      }
    }
  }
}

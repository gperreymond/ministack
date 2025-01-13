job "hello-world" {
  datacenters = ["*"]
  constraint {
    attribute = "${attr.unique.hostname}"
    value     = "worker-ronflex"
  }
  group "server" {
    count = 2
    network {
      mode = "bridge"
      port "envoy_metrics" { to = "9102" }
    }
    service {
      provider = "consul"
      name     = "hello-world-port-http"
      port     = "6000"
      meta {
        envoy_metrics_port = "${NOMAD_HOST_PORT_envoy_metrics}"
      }
      connect {
        sidecar_service {
          proxy {
            config {
              envoy_prometheus_bind_addr = "0.0.0.0:9102"
            }
          }
        }
        sidecar_task {
          config {
            auth_soft_fail = true
          }
          resources {
            cpu = 100
            memory = 32
            memory_max = 128
          }
        }
      }
      check {
        expose   = true
        name     = "hello-world-health"
        type = "http"
        path = "/"
        interval = "10s"
        timeout  = "2s"
      }
      tags = [
        "traefik.enable=true",
        "traefik.http.routers.hello-world.rule=Host(`hello-world.docker.localhost`)",
        "traefik.http.routers.hello-world.entrypoints=web",
        "traefik.http.services.hello-world.loadbalancer.passhostheader=true"
      ]
    }
    task "web" {
      driver = "docker"
      config {
        image      = "busybox:1"
        privileged = true
        command    = "httpd"
        args       = ["-v", "-f", "-p", "6000", "-h", "/local"]
      }
      template {
        data        = <<-EOF
<h1>Hello, Nomad!</h1>
<ul>
    <li>Task: {{env "NOMAD_TASK_NAME"}}</li>
    <li>Group: {{env "NOMAD_GROUP_NAME"}}</li>
    <li>Job: {{env "NOMAD_JOB_NAME"}}</li>
</ul>
EOF
        destination = "local/index.html"
      }
      resources {
        cpu = 50
        memory = 64
        memory_max = 128
      }
    }
  }
}

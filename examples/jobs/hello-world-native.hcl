job "hello-world-native" {
  datacenters = ["*"]
  constraint {
    attribute = "${attr.unique.hostname}"
    value     = "worker-pikachu"
  }
  group "server" {
    count = 1
    network {
      mode = "bridge"
      port "http" { to = 6000 }
    }
    service {
      provider = "consul"
      name     = "hello-world-native-port-http"
      port     = "http"
      connect {
        native = true
      }
      check {
        name     = "hello-world-native-health"
        type = "http"
        path = "/"
        interval = "10s"
        timeout  = "2s"
      }
      tags = [
        "traefik.enable=true",
        "traefik.http.routers.hello-world-native.rule=Host(`hello-world-native.docker.localhost`)",
        "traefik.http.routers.hello-world-native.entrypoints=web",
        "traefik.http.services.hello-world-native.loadbalancer.passhostheader=true",
        "traefik.http.services.hello-world-native.loadbalancer.server.scheme=http",
      ]
    }
    task "web" {
      driver = "docker"
      config {
        image      = "busybox:1"
        privileged = true
        command    = "httpd"
        args       = ["-v", "-f", "-p", "6000", "-h", "/local"]
        ports = ["http"]
      }
      template {
        data        = <<-EOF
<h1>Hello, Nomad!</h1>
<h3>service provider: consul (connect native)</h3>
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

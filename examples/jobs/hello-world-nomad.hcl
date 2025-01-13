job "hello-world-nomad" {
  datacenters = ["*"]
  group "server" {
    count = 1
    network {
      mode = "bridge"
      port "http" {
        to = 6000
      }
    }
    service {
      provider = "nomad"
      name     = "hello-world-nomad-port-http"
      port     = "http"
      check {
        name     = "hello-world-nomad-health"
        type = "http"
        path = "/"
        interval = "10s"
        timeout  = "2s"
      }
      tags = [
        "traefik.enable=true",
        "traefik.http.routers.hello-world-nomad.rule=Host(`hello-world-nomad.docker.localhost`)",
        "traefik.http.routers.hello-world-nomad.entrypoints=web",
        "traefik.http.services.hello-world-nomad.loadbalancer.passhostheader=true",
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
<h3>service provider: nomad</h3>
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

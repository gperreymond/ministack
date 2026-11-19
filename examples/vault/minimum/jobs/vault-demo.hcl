# read a vault secret from a nomad task, with the nomad workload identity
# usage:
#   vault kv put secret/default/vault-demo password=ministack
#   nomad job run examples/vault/minimum/jobs/vault-demo.hcl
#   nomad alloc logs -job vault-demo
job "vault-demo" {
  # no datacenters: all of them (nomad >= 1.5), works on every example

  group "demo" {
    task "demo" {
      driver = "docker"

      config {
        image   = "busybox:1.37"
        command = "sh"
        args    = ["-c", "echo \"password from vault: $PASSWORD\"; sleep 3600"]
      }

      # default workload identity, vault role "nomad-workloads"
      vault {}

      # the default policy allows secret/data/<namespace>/<job_id>
      template {
        data        = <<-EOF
          {{ with secret "secret/data/default/vault-demo" }}PASSWORD={{ .Data.data.password }}{{ end }}
        EOF
        destination = "secrets/vault.env"
        env         = true
      }

      resources {
        cpu    = 50
        memory = 32
      }
    }
  }
}

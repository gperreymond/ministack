# Ministack

![Logo of Ministack](images/logo-256.png)

Ministack is a lightweight, local environment tool designed to mimic a Minikube-like experience for HashiCorp's suite of tools: **Nomad**, **Consul**, and **Vault**. It allows you to easily deploy and manage local clusters, simplifying development and testing workflows.

---

## Features

- **Local Cluster Deployment**: Quickly launch and manage local clusters for Nomad, Consul, and Vault.
- **Configuration Flexibility**: Define and manage multiple cluster configurations using YAML files.
- **Simplified CLI Commands**: Start and stop clusters effortlessly with command-line tools.

---

## Installation

To install Ministack, follow this step:

```sh
$ curl -fsSL https://raw.githubusercontent.com/gperreymond/ministack/main/install | bash
```

---

## Cluster configuration details

All default versions, are the minimum versions working with the automatic config files.

```yaml
# mandatory
name: 'my-cluster-name'
# mandatory
datacenter: 'datacenter name used for nomad/consul'

hashibase:
  repository: 'custom' # default = "ghcr.io/gperreymond/hashibase"
  tag: 'custom' # default = "base-1.1.0"

services:
  consul:
    enabled: true
    version: 'x.x.x' # default = "1.20.2"
    log_level: 'trace|debug|info|warn|error' # default = "info"
    # self-elect, should be 3 or 5 for production
    bootstrap_expect: 1 # default = 1
  nomad:
    enabled: true
    log_level: 'trace|debug|info|warn|error' # default = "info"
    version: 'x.x.x' # default = "1.9.5"
    customized:
      extra_configs: false # default = false (see "customize nomad" below in the documentation )
      extra_volumes: [] # default = [] (see "customize nomad" below in the documentation )
      tls: false # default = false (see "customize nomad" below in the documentation )
      bind_addr: '....' # default = null (see "nomad bind_add" documentation)
      retry_join: [] # default = [] (see "customize nomad" below in the documentation )
    # self-elect, should be 3 or 5 for production.
    # could be 0, i you want only clients, but you need to work on custom retry_join
    bootstrap_expect: 1 # default = 1
    clients: # default = []
      - name: 'worker-pikachu'
      - name: 'worker-ronflex'

plugins:
  traefik:
    enabled: true
    log_level: 'RACE|DEBUG|INFO|WARN|ERROR|FATAL|PANIC' # default = "INFO"
    version: 'x.x.x' # default = "3.3.1"
  prometheus:
    enabled: true
    log_level: 'info' # default = "info"
    version: 'x.x.x'  # default = "3.1.0"
    customized: false # default = false (see "customize prometheus" below in the documentation )
```

---

## Some useful articles

* https://developer.hashicorp.com/nomad/docs/configuration
* https://github.com/GuyBarros/nomad_jobs
* https://romanzipp.com/blog/get-started-with-hashi-nomad-consul
* https://mrkaran.dev/posts/nomad-networking-explained
* https://last9.io/blog/mastering-prometheus-relabeling-a-comprehensive-guide
* https://samber.github.io/awesome-prometheus-alerts/rules.html

---

## Contributing

Contributions are welcome! Feel free to fork the repository and submit a pull request.

---

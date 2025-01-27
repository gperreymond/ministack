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
name: 'europe-paris'
datacenter: 'europe-paris'

hashibase:
  repository: 'ghcr.io/gperreymond/hashibase'
  tag: 'base-1.2.0'

network:
  subnet: '10.1.0.0/24'

secrets:
  - 'nomad.env'

services:
  nomad:
    enabled: true
    config:
      bind_addr: '{{ GetInterfaceIP \"eth0\" }}'
      log_level: 'info'
      server:
        bootstrap_expect: 3
        labels:
          - 'traefik.enable=true'
          - 'traefik.http.routers.nomad.rule=Host(`nomad.docker.localhost`)'
          - 'traefik.http.routers.nomad.entrypoints=web'
          - 'traefik.http.services.nomad.loadbalancer.server.port=4646'
        # retry_join:
        #   - 'provider=aws tag_key=... tag_value=...'
        #   - 'provider=azure tag_name=... tag_value=... tenant_id=... client_id=... subscription_id=... secret_access_key=...'
    servers:
      - name: 'nomad-server-1a'
        labels:
          - 'server=1a'
      - name: 'nomad-server-1b'
        labels:
          - 'server=1b'
      - name: 'nomad-server-1c'
        labels:
          - 'server=1c'
    clients:
      - name: 'worker-system'
      - name: 'worker-monitoring'
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

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

## Usages

Automatic usage:
```sh
$ ministack --config [path_to_your_config_yaml_file] --start
$ ministack --config [path_to_your_config_yaml_file] --stop
```

But you can also, only generate the docker compose files, and manage manually:
```sh
$ ministack --config examples/europe-paris/cluster.yaml
$ docker compose --file examples/europe-paris/.ministack/cluster.yaml up -d
$ docker compose --file examples/europe-paris/.ministack/cluster.yaml down
```

This approach is ideal if you want to add some docker compose services you created, or working on a customization of nomad files.

---

## Cluster configuration details

This is the minimum configuration to have a fully working nomad server/client.  
With this configuration and those command lines, you can start working with nomad.

```sh
$ ministack --config examples/nomad/minimum/cluster.yaml --start
$ ministack --config examples/nomad/minimum/cluster.yaml --stop
```

```yaml
name: 'nomad-minimum'
datacenter: 'localhost'

services:
  nomad:
    enabled: true
    servers:
      - name: 'server'
    clients:
      - name: 'worker-1'
      - name: 'worker-2'
```

This is a full example if you decide to use all the custom values.  
You can find a full usage into __examples/europe-paris__. This approach let you really customize nomad, and adding __plugins__.  
Plugins are pieces of docker compose services you managed and integrate into the stack.  
You will find traefik as plugin into this example.

```yaml
name: 'europe-paris'
datacenter: 'europe-paris'

network:
  subnet: '10.1.20.0'

plugins:
  - 'plugins/traefik.yaml'
  - 'plugins/prometheus-reloader.yaml'

services:
  nomad:
    enabled: true
    version: '1.9.5'
    config:
      bind_addr: '{{ GetInterfaceIP \"eth0\" }}'
      log_level: 'info'
      server:
        bootstrap_expect: 3
        local_volumes:
          - 'certs:/certs'
          - 'nomad/01-tls.hcl:/etc/nomad.d/config/01-tls.hcl'
        labels:
          - 'traefik.enable=true'
          - 'traefik.http.routers.nomad.rule=Host(`nomad.docker.localhost`)'
          - 'traefik.http.routers.nomad.entrypoints=web'
          - 'traefik.http.services.nomad.loadbalancer.server.port=4646'
          - 'traefik.http.services.nomad.loadbalancer.server.scheme=https'
          - 'traefik.http.services.nomad.loadbalancer.serverstransport=insecure@file'
        # retry_join:
        #   - 'provider=aws tag_key=... tag_value=...'
        #   - 'provider=azure tag_name=... tag_value=... tenant_id=... client_id=... subscription_id=... secret_access_key=...'
      client:
        local_volumes:
          - 'certs:/certs'
          - 'nomad/01-tls.hcl:/etc/nomad.d/config/01-tls.hcl'
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
        local_volumes:
          - 'prometheus/rules:/mnt/prometheus/rules'
          - 'prometheus/scrape_configs:/mnt/prometheus/scrape_configs'
        docker_volumes:
          - 'prometheus_data'
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

# Ministack

![Logo of Ministack](images/logo-256.png)

Ministack is a lightweight, local environment tool designed to mimic a Minikube-like experience for HashiCorp's suite of tools: **Nomad**, **Consul**, and **Vault**. It allows you to easily deploy and manage local clusters, simplifying development and testing workflows.

## Features

- **Local Cluster Deployment**: Quickly launch and manage local clusters for Nomad, Consul, and Vault.
- **Configuration Flexibility**: Define and manage multiple cluster configurations using YAML files.
- **Simplified CLI Commands**: Start and stop clusters effortlessly with command-line tools.

## Installation

To install Ministack, follow these steps:

```bash
# on linux
sudo curl -L https://github.com/gperreymond/ministack/releases/download/v1.0.2/ministack-linux -o /usr/local/bin/ministack
# on macos
sudo curl -L https://github.com/gperreymond/ministack/releases/download/v1.0.2/ministack-macos -o /usr/local/bin/ministack
# then, move the binary
sudo chmod +x /usr/local/bin/ministack
# check if all is ok
ministack --version
```

## Cluster configuration details

```yaml
# mandatory
name: 'the cluster name'
# mandatory
datacenter: 'datacenter name used for nomad/consul'
# not mandatory, default is "info"
log_level: 'trace|debug|info|warn|error'

# mandatory
# for now we have only one docker image
image:
  repository: 'ghcr.io/gperreymond/hashibase'
  tag: 'base-1.0.0'

# nothing mandatory here
services:
  consul:
    enabled: true
    # if enabled is true, version is not mandatory
    # default version will be "1.20.1"
    version: 'x.x.x'
    # if enabled is true, mandatory
    # it will be => bootstrap_expect
    replicas: 1
  nomad:
    enabled: true
    # if enabled is true, version is not mandatory
    # default version will be "1.9.4"
    version: 'x.x.x'
    # if enabled is true, mandatory
    # it will be => bootstrap_expect
    replicas: 1
    # not mandatory
    clients:
      - name: 'worker-pikachu'
      - name: 'worker-ronflex'

# nothing mandatory here
plugins:
  traefik:
    enabled: true
    # not mandatory, default is "INFO"
    log_level: 'RACE|DEBUG|INFO|WARN|ERROR|FATAL|PANIC'
    # if enabled is true, version is mandatory, only 3.x.x
    version: '3.3.1'
```

## Some examples

Warnings:
* stop a cluster before starting another one.
* all data will persist.

Common urls:
* http://traefik.docker.localhost/
* http://nomad.docker.localhost/
* http://consul.docker.localhost/

### Cluster mode "nomad only"

__Configuration__
* consul is disable
* nomad with replicas 3 and one client
* vault is disable

```sh
# start cluster
$ ministack --config examples/nomad-only.yaml --start
# stop cluster
$ ministack --config examples/nomad-only.yaml --stop
```

### Cluster mode dev

__Configuration__
* consul with replicas 1
* nomad with replicas 1 and 2 clients
* vault is disable

```sh
# start cluster
$ ministack --config examples/nomad-with-consul.yaml --start
# stop cluster
$ ministack --config examples/nomad-with-consul.yaml --stop
```

## Customize your own configurations for nomad, consul and/or vault

...

## Some useful articles

* https://romanzipp.com/blog/get-started-with-hashi-nomad-consul
* https://mrkaran.dev/posts/nomad-networking-explained


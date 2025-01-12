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
sudo curl -L https://github.com/gperreymond/ministack/releases/download/v1.0.0/ministack-linux -o /usr/local/bin/ministack
# on macos
sudo curl -L https://github.com/gperreymond/ministack/releases/download/v1.0.0/ministack-macos -o /usr/local/bin/ministack
# then, move the binary
sudo chmod +x /usr/local/bin/ministack
# check if all is ok
ministack --version
```

## Exemples of usages

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

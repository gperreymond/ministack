# MINISTACK

## Concepts

...

## Install binary

```sh
$ sudo curl -L https://github.com/gperreymond/ministack/releases/download/v1.0.0/ministack -o /usr/local/bin/ministack
$ sudo chmod +x /usr/local/bin/ministack
$ ministack --version
```

## Manage your own clusters

Warnings:
* stop a cluster before starting another one.
* all data will persist.

### Cluster mode old versions

__Configuration__
* consul is disable
* nomad with replicas 3, version 1.8.2
* vault is disable

```sh
# start cluster
$ ministack --config clusters/old.yaml --start
# stop cluster
$ ministack --config clusters/old.yaml --stop
```

### Cluster mode dev

__Configuration__
* consul with replicas 1
* nomad with replicas 1
* vault is disable

```sh
# start cluster
$ ministack --config clusters/dev.yaml --start
# stop cluster
$ ministack --config clusters/dev.yaml --stop
```

### Cluster mode all

__Configuration__
* consul with replicas 3
* nomad with replicas 3
* vault with replicas 3

```sh
# start cluster
$ ministack --config clusters/all.yaml --start
# stop cluster
$ ministack --config clusters/all.yaml --stop
```

## Customize your own configurations for nomad, consul and/or vault

...

## Some useful articles

* https://romanzipp.com/blog/get-started-with-hashi-nomad-consul

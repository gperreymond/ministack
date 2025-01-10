# MINISTACK

## Prepare your local environment

```sh
# we us "asdf" for versionning
$ ./install-dependencies.sh
```

## Concepts

...

## Manage your own configurations

...

## Examples

Warnings:
* docker networks will be created for all examples.
* stop a cluster before starting another one.

### Cluster mode old versions

__Configuration__
* consul is disable
* nomad with replicas 1, version 1.8.2
* vault is disable

```sh
# start cluster
$ ./bin/ministack -config clusters/old.yaml start
# stop cluster
$ ./bin/ministack -config clusters/old.yaml stop
```

### Cluster mode dev

__Configuration__
* consul with replicas 1
* nomad with replicas 1
* vault is disable

```sh
# start cluster
$ ./bin/ministack -config clusters/dev.yaml start
# stop cluster
$ ./bin/ministack -config clusters/dev.yaml stop
```

### Cluster mode all

__Configuration__
* consul with replicas 3
* nomad with replicas 3
* vault with replicas 3

```sh
# start cluster
$ ./bin/ministack -config clusters/all.yaml start
# stop cluster
$ ./bin/ministack -config clusters/all.yaml stop
```

## Some useful articles

* https://romanzipp.com/blog/get-started-with-hashi-nomad-consul

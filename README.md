# MINISTACK

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
$ ./bin/ministack -config clusters/old.yaml -command start
# stop cluster
$ ./bin/ministack -config clusters/old.yaml -command stop
```

### Cluster mode dev

__Configuration__
* consul with replicas 1
* nomad with replicas 1
* vault is disable

```sh
# start cluster
$ ./bin/ministack -config clusters/dev.yaml -command start
# stop cluster
$ ./bin/ministack -config clusters/dev.yaml -command stop
```

### Cluster mode all

__Configuration__
* consul with replicas 3
* nomad with replicas 3
* vault with replicas 3

```sh
# start cluster
$ ./bin/ministack -config clusters/all.yaml -command start
# stop cluster
$ ./bin/ministack -config clusters/all.yaml -command stop
```

## Some useful articles

* https://romanzipp.com/blog/get-started-with-hashi-nomad-consul

# MINISTACK

## Concepts

...

## Manage your own configurations

...

## Examples

Warning: Docker networks will be created for all examples, stop a cluster before starting another one.

### Cluster mode dev

__Configuration__
* consul with replicas 1
* nomad with replicas 1
* vault is disable

```sh
# start cluster
$ devbox run start:dev
# stop cluster
$ devbox run stop:dev
```

### Cluster mode all

__Configuration__
* consul with replicas 3
* nomad with replicas 3
* vault with replicas 3

```sh
# start cluster
$ devbox run start:all
# stop cluster
$ devbox run stop:all
```
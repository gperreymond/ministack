#!/bin/bash

set -e

# -----------------------
# PREPARE
# -----------------------

while [[ "$#" -gt 0 ]]; do
    case $1 in
        -prefix) prefix="$2"; shift ;;
        *) echo "[ERROR] param is not authorized"; exit 1;;
    esac
    shift
done

if [[ -z "$prefix" ]]; then
    echo "[ERROR] prefix is mandatory"
    exit 1
fi

# -----------------------
# RESUME
# -----------------------

echo ""
echo ">>> DOCKER VOLUMES DELETE <<<"
echo "================================================================="
echo "[INFO] prefix............................... '${prefix}'"
echo "================================================================="
echo ""

# -----------------------
# EXECUTE COMMAND
# -----------------------

# List Docker volumes that start with "prefix"
volumes_to_delete=$(docker volume ls --format "{{.Name}}" | grep "^${prefix}")

# Check if any matching volumes exist
if [[ -z "$volumes_to_delete" ]]; then
  echo "[WARNING] no Docker volumes found starting with '${prefix}'"
  echo ""
  exit 0
fi

for volume in $volumes_to_delete; do
    echo "[INFO] deleting the following volume: $volume"
    docker volume rm "$volume"
done

echo "[INFO] all matching volumes have been deleted"
echo ""

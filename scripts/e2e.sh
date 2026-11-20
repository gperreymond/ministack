#!/bin/bash

# end to end test of an example, used by the ci and runnable locally:
# start, api checks, demo jobs, stop/start, checks again, purge
#
# usage: scripts/e2e.sh <example dir> [ministack binary]
#   scripts/e2e.sh examples/secure
#   scripts/e2e.sh examples/nomad/minimum ./target/release/ministack
#
# needs: docker, curl. The cluster is purged at the end (success or failure).

set -euo pipefail

EXAMPLE=${1:?usage: scripts/e2e.sh <example dir> [ministack binary]}
BIN=${2:-./target/debug/ministack}
CONFIG="${EXAMPLE}/cluster.yaml"
REPO_DIR=$(cd "$(dirname "$0")/.." && pwd)
JOBS_DIR="${REPO_DIR}/examples"
PROJECT=$(sed -n "s/^name: *['\"]\{0,1\}\([^'\"]*\)['\"]\{0,1\} *$/\1/p" "$CONFIG" | head -n 1)
RAFT=$(grep -q "mode: *'raft'" "$CONFIG" && echo true || echo false)

log() { echo "[E2E] $*"; }
fail() { echo "[E2E][FAIL] $*"; exit 1; }

# retry <seconds> <command...>: until the command succeeds
retry() {
  local timeout=$1; shift
  local end=$((SECONDS + timeout))
  until "$@" > /dev/null 2>&1; do
    [ $SECONDS -ge $end ] && return 1
    sleep 2
  done
}

# first container of the cluster with a ministack label, e.g. node ministack/nomad=server
node() {
  docker ps --filter "label=com.docker.compose.project=${PROJECT}" --filter "label=$1" \
    --format '{{.Names}}' | sort | head -n 1
}

dump_logs() {
  echo "::group::[E2E] containers and logs"
  docker ps -a --filter "label=com.docker.compose.project=${PROJECT}" --format '{{.Names}}\t{{.Status}}' || true
  for c in $(docker ps -a --filter "label=com.docker.compose.project=${PROJECT}" --format '{{.Names}}'); do
    echo "----- ${c}"
    docker logs --tail 40 "$c" 2>&1 || true
    for service in consul nomad; do
      docker exec "$c" journalctl -u "$service" --no-pager -n 40 2> /dev/null || true
    done
  done
  echo "::endgroup::"
}

cleanup() {
  local rc=$?
  if [ $rc -ne 0 ]; then
    dump_logs
  fi
  log "purging ${EXAMPLE}..."
  "$BIN" --config "$CONFIG" --purge > /dev/null 2>&1 || true
  exit $rc
}
trap cleanup EXIT

# http code of an api call from the host
http_code() {
  local ca=$1; shift
  curl -s -o /dev/null -w '%{http_code}' ${ca:+--cacert "$ca"} "$@"
}

check_apis() {
  log "checking the apis from the host..."
  [ "$(http_code "${NOMAD_CACERT:-}" -H "X-Nomad-Token: ${NOMAD_TOKEN:-}" "${NOMAD_ADDR}/v1/jobs")" == "200" ] \
    || fail "nomad api"
  if [ -n "${NOMAD_TOKEN:-}" ]; then
    [ "$(http_code "${NOMAD_CACERT:-}" "${NOMAD_ADDR}/v1/jobs")" == "403" ] || fail "nomad acl not enforced"
  fi
  if [ -n "${CONSUL_HTTP_ADDR:-}" ]; then
    curl -sf ${CONSUL_CACERT:+--cacert "$CONSUL_CACERT"} "${CONSUL_HTTP_ADDR}/v1/status/leader" | grep -q ':' \
      || fail "consul leader"
    if [ -n "${CONSUL_HTTP_TOKEN:-}" ]; then
      [ "$(http_code "${CONSUL_CACERT:-}" "${CONSUL_HTTP_ADDR}/v1/acl/tokens")" == "403" ] || fail "consul acl not enforced"
    fi
  fi
  if [ -n "${VAULT_ADDR:-}" ]; then
    [ "$(http_code "${VAULT_CACERT:-}" "${VAULT_ADDR}/v1/sys/health")" == "200" ] || fail "vault health"
  fi
  if [[ "${NOMAD_ADDR}" == https://* ]]; then
    [ "$(http_code "" "${NOMAD_ADDR/https:/http:}/v1/jobs")" != "200" ] || fail "nomad answers plain http"
  fi
}

run_job() {
  local file=$1 name
  name=$(basename "$file" .hcl)
  log "running the ${name} job..."
  docker cp "$file" "${SERVER}:/tmp/${name}.hcl"
  # the functional checks below tell if it really works
  docker exec "$SERVER" timeout 300 nomad job run "/tmp/${name}.hcl" > "/tmp/e2e-${PROJECT}-${name}.log" 2>&1 \
    || { cat "/tmp/e2e-${PROJECT}-${name}.log"; fail "nomad job run ${name}"; }
}

# whoami_answers [service]: the whoami job, or its copy with a local image
whoami_answers() {
  local address
  address=$(docker exec "$SERVER" nomad service info -t '{{ range . }}{{ .Address }}:{{ .Port }}{{ end }}' "${1:-whoami}")
  docker exec "$SERVER" curl -sf "http://${address}/" | grep -q '^Hostname'
}

# the dashboard reaches the api through its envoy sidecar
countdash_mesh_answers() {
  local dashboard
  dashboard=$(docker exec "$WORKER" docker ps -q --filter name=dashboard- | head -n 1)
  [ -n "$dashboard" ] || return 1
  docker exec "$WORKER" docker run --rm --network "container:${dashboard}" curlimages/curl:8.11.1 \
    -sf -m 5 http://127.0.0.1:8080/ | grep -q '"count"'
}

vault_demo_reads() {
  docker exec "$SERVER" nomad alloc logs -job vault-demo | grep -q "password from vault: $1"
}

check_jobs() {
  log "checking the demo jobs..."
  retry 120 whoami_answers || fail "whoami does not answer"
  retry 120 whoami_answers whoami-local || fail "whoami-local (image copied from the host) does not answer"
  if [ -n "${CONSUL_HTTP_ADDR:-}" ]; then
    retry 180 countdash_mesh_answers || fail "countdash: the api is not reachable through the service mesh"
  fi
  if [ -n "${VAULT_ADDR:-}" ] && [ "$1" == "vault" ]; then
    retry 120 vault_demo_reads e2e || fail "vault-demo does not read its secret"
  fi
}

start() {
  log "starting ${EXAMPLE}..."
  "$BIN" --config "$CONFIG" --start
  "$BIN" --config "$CONFIG" --status || fail "ministack --status: a node is not ready"
  # reset, then load the variables of this cluster
  unset NOMAD_ADDR NOMAD_TOKEN NOMAD_CACERT CONSUL_HTTP_ADDR CONSUL_HTTP_TOKEN CONSUL_CACERT VAULT_ADDR VAULT_TOKEN VAULT_CACERT
  eval "$("$BIN" --config "$CONFIG" --env)"
  SERVER=$(node ministack/nomad=server)
  WORKER=$(node ministack/nomad=client)
  VAULT=$(node ministack/vault=server)
  [ -n "$SERVER" ] && [ -n "$WORKER" ] || fail "no nomad server or client found"
}

# ------------------------------------------------------------------ first start
start
check_apis

run_job "${JOBS_DIR}/nomad/minimum/jobs/whoami.hcl"

# an image that only exists on the host, copied into the clients
log "copying a local image into the clients..."
docker pull -q traefik/whoami:v1.10.3 > /dev/null
docker tag traefik/whoami:v1.10.3 ministack-local/whoami:e2e
"$BIN" --config "$CONFIG" --load-image ministack-local/whoami:e2e
LOCAL_JOB="$(mktemp -d)/whoami-local.hcl"
sed -e 's#traefik/whoami:v1.10.3#ministack-local/whoami:e2e#' -e 's/"whoami"/"whoami-local"/g' \
  "${JOBS_DIR}/nomad/minimum/jobs/whoami.hcl" > "$LOCAL_JOB"
run_job "$LOCAL_JOB"
if [ -n "${CONSUL_HTTP_ADDR:-}" ]; then
  run_job "${JOBS_DIR}/consul/minimum/jobs/countdash.hcl"
fi
if [ -n "${VAULT_ADDR:-}" ]; then
  docker exec "$VAULT" vault kv put secret/default/vault-demo password=e2e > /dev/null
  run_job "${JOBS_DIR}/vault/minimum/jobs/vault-demo.hcl"
fi
check_jobs vault

# ------------------------------------------------------------------ stop / start
log "stopping ${EXAMPLE} (data kept)..."
"$BIN" --config "$CONFIG" --stop > /dev/null
! "$BIN" --config "$CONFIG" --status > /dev/null || fail "ministack --status: ready after --stop"
start
check_apis
# a dev vault is in memory: its secret is gone after a restart
if [ "$RAFT" == "true" ]; then
  check_jobs vault
else
  check_jobs no-vault
fi

log "${EXAMPLE}: OK"

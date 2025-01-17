#!/bin/bash

set -e

# Directory containing the YAML files
files_dir="tests/files"

# Loop through each file in the directory
for file in "$files_dir"/*.yaml; do
    echo "Processing $file"

    # Run the cargo command with the current file
    cargo run -- --config "$file" > /dev/null 2>&1

    # Generate the Docker Compose config
    docker compose --file /tmp/ministack/cluster.yaml config > /tmp/gomplate-data.yaml

    # Use gomplate to process the config
    gomplate -d config=/tmp/gomplate-data.yaml -i '
{{- $data := (datasource "config") -}}
name: "{{ $data.name }}"
services:
{{- range $key, $service := $data.services }}
- name: "{{ $key }}"
  image: "{{ $service.image }}"
  {{- if has $service "volumes" }}
  volumes:
  {{- range $volume := $service.volumes }}
  - "{{ $volume }}"
  {{- end }}
  {{- end }}
  {{- if has $service "depends_on" }}
  depends_on:
  {{- range $dependency, $item := $service.depends_on }}
  - "{{ $dependency }}"
  {{- end }}
  {{- end }}
{{- end }}
' | yq
done

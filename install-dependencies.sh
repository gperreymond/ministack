#!/bin/bash

set -e

asdf plugin add jq
asdf plugin add yq
asdf plugin add gomplate
asdf plugin add task
asdf plugin add nomad
asdf plugin add consul
asdf plugin add vault
asdf plugin add rust

asdf install

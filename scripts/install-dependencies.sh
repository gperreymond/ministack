#!/bin/bash

set -e

asdf plugin add gomplate
asdf plugin add yq
asdf plugin add rust
asdf plugin add nomad
asdf plugin add consul
asdf plugin add vault

asdf install

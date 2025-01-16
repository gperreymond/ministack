#!/bin/bash

set -e

cd examples
terragrunt hclfmt
cd ..

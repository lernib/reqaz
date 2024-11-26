#!/bin/bash

set -xe

cargo build

echo "Serving on http://localhost:5000"

cargo run -- -C e2e/web -p 5000 --log true

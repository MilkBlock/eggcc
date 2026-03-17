#!/bin/bash

set -euo pipefail

files=(infra/nightly-resources/*.js)

if command -v prettier >/dev/null 2>&1; then
    exec prettier "${files[@]}" --write
fi

exec npx prettier "${files[@]}" --write

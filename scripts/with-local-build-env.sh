#!/bin/bash

set -euo pipefail

prepend_path_var() {
    local var_name="$1"
    local value="$2"
    local current="${!var_name:-}"
    if [ -n "$current" ]; then
        export "$var_name=$value:$current"
    else
        export "$var_name=$value"
    fi
}

if [ "$(uname -s)" = "Darwin" ]; then
    if [ -d /opt/homebrew/opt/llvm@18 ]; then
        export LLVM_SYS_180_PREFIX=/opt/homebrew/opt/llvm@18
        prepend_path_var PATH /opt/homebrew/opt/llvm@18/bin
    fi

    if [ -d /opt/homebrew/opt/cbc/lib/pkgconfig ]; then
        prepend_path_var PKG_CONFIG_PATH /opt/homebrew/opt/cbc/lib/pkgconfig
    fi

    if [ -d /opt/homebrew/opt/zstd/lib/pkgconfig ]; then
        prepend_path_var PKG_CONFIG_PATH /opt/homebrew/opt/zstd/lib/pkgconfig
    fi

    if [ -d /opt/homebrew/lib ]; then
        prepend_path_var LIBRARY_PATH /opt/homebrew/lib
        prepend_path_var DYLD_FALLBACK_LIBRARY_PATH /opt/homebrew/lib
    fi

    if [ -d /opt/homebrew/opt/cbc/lib ]; then
        prepend_path_var LIBRARY_PATH /opt/homebrew/opt/cbc/lib
        prepend_path_var DYLD_FALLBACK_LIBRARY_PATH /opt/homebrew/opt/cbc/lib
    fi

    if [ -d /opt/homebrew/opt/zstd/lib ]; then
        prepend_path_var LIBRARY_PATH /opt/homebrew/opt/zstd/lib
        prepend_path_var DYLD_FALLBACK_LIBRARY_PATH /opt/homebrew/opt/zstd/lib
    fi
fi

exec "$@"

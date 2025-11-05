#!/bin/bash

mkdir -p target/debug/ && mkdir -p target/release/

# ls common_service | xargs -n 1 printf "./scripts/build_common.sh %s $1\n" | bash

# shellcheck disable=SC2045
for module in $(ls common_service); do
    echo "build $module."
    ./scripts/build_common.sh "$module" "$1"
done

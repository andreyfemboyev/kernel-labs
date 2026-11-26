#!/bin/sh
set -eu
if [ "$#" -ne 4 ]; then
    echo "usage: $0 KLAB SOURCE_ROOT PROFILE NEW_OUTPUT_DIRECTORY" >&2
    exit 2
fi
if [ -e "$4" ]; then
    echo "choose a fresh output directory" >&2
    exit 2
fi
"$1" build --root "$2" --profile "$3" --out "$4" --dry-run
printf 'Execute the reviewed build plan? [y/N] '
read -r answer
case "$answer" in
    y|Y) exec "$1" build --root "$2" --profile "$3" --out "$4" --trust ;;
    *) exit 2 ;;
esac

#!/bin/sh
# The first compile proves imports and dependencies work. Negative cases must
# then fail with their intended borrow-checker code, not just any compiler error.
set -eu
export CARGO_TERM_COLOR=never
cargo_command=$1
shift
log=$(mktemp "${TMPDIR:-/tmp}/klover-ownership.XXXXXX")
trap 'rm -f "$log"' EXIT HUP INT TERM

if ! "$cargo_command" rustc "$@" --example ownership -- --emit=metadata >"$log" 2>&1; then
    cat "$log"
    exit 1
fi
printf '%s\n' 'ownership: valid borrows compile'
for case_name in leak share mutate handle; do
    expected=E0505
    if [ "$case_name" = mutate ]; then expected=E0502; fi
    if "$cargo_command" rustc "$@" --example ownership -- --emit=metadata --cfg "klover_case=\"$case_name\"" >"$log" 2>&1; then
        cat "$log"
        printf 'ownership: %s unexpectedly compiled\n' "$case_name" >&2
        exit 1
    fi
    if ! grep -F "error[$expected]" "$log" >/dev/null; then
        cat "$log"
        printf 'ownership: %s did not report %s\n' "$case_name" "$expected" >&2
        exit 1
    fi
    # Any second compiler failure (for example inaccessible modules) is suspect.
    if grep '^error\[' "$log" | grep -v -F "error[$expected]" >/dev/null; then
        cat "$log"
        exit 1
    fi
    printf 'ownership: %s rejected with %s\n' "$case_name" "$expected"
done

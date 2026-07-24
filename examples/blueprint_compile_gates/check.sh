#!/bin/sh
set -eu

cd "$(dirname "$0")"

# Positive control: public Blueprint types resolve through the umbrella.
cargo check --quiet

check_gate() {
    feature="$1"
    code="$2"
    output="target/${feature}.stderr"

    if cargo check --quiet --features "$feature" >"$output" 2>&1; then
        echo "gate unexpectedly compiled: $feature" >&2
        exit 1
    fi

    if ! grep -q "error\\[$code\\]" "$output"; then
        echo "gate failed for the wrong reason: $feature (wanted $code)" >&2
        sed -n '1,160p' "$output" >&2
        exit 1
    fi
}

# Stage 6 gates only. Stage 7 extends this script with App/builder application
# gates after those APIs exist.
check_gate inbound_rejects_outpoint E0308
check_gate outbound_rejects_endpoint E0308
check_gate erased_trait_is_private E0603
check_gate blueprint_has_no_build E0599
check_gate configured_has_no_build E0599

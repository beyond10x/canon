#!/bin/sh
# Classify a pull request by its changed files. The one path list of this repository is `classify`
# below; both workflows and crates/canon/tests/planning_only_classifier.rs use this file.
#
#   classify-changes.sh              reads changed paths, one per line, on stdin
#   classify-changes.sh --diff BASE  reads `git diff --no-renames --name-only BASE...HEAD`
#
# Prints one of: planning-only, docs-site-only, code. Any failure, and an empty list, print code.
set -eu

# Class of one path: planning, site or code. First match wins.
#   AGENTS.md                      code     read by crates/canon/tests/adversary2_ess_gate.rs
#   .engineering/planning/story/*  site     read by crates/canon-docs/src/status.rs into website/data/status.json
#   .engineering/*  docs/*         planning no test, gate step or generator reads them
#   website/*                      site     the built documentation site
#   *.md in the root               planning prose (a .md deeper in the tree is code: a crate may read it)
#   everything else                code
classify() {
    case "$1" in
    AGENTS.md | */AGENTS.md) echo code ;;
    .engineering/planning/story/*) echo site ;;
    .engineering/* | docs/*) echo planning ;;
    website/*) echo site ;;
    */*) echo code ;;
    *.md) echo planning ;;
    *) echo code ;;
    esac
}

answer() {
    seen=0
    site=0
    while IFS= read -r path; do
        [ -n "$path" ] || continue
        seen=1
        case "$(classify "$path")" in
        planning) ;;
        site) site=1 ;;
        *)
            echo code
            return 0
            ;;
        esac
    done
    if [ "$seen" = 0 ]; then
        echo code
    elif [ "$site" = 1 ]; then
        echo docs-site-only
    else
        echo planning-only
    fi
}

if [ "${1:-}" = "--diff" ]; then
    list=$(git -c core.quotepath=off diff --no-renames --name-only "${2:-}...HEAD" 2>/dev/null) || {
        echo code
        exit 0
    }
    printf '%s\n' "$list" | answer
else
    answer
fi

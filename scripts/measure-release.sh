#!/bin/sh
# Measure the three release builds of `canon` on two workloads, on this host, for
# docs/design/linux-release.md (story:musl-workload-measurement).
#
#   measure-release.sh [runs]     runs per build and workload, default 10
#
# Builds, each with scripts/package.sh binary into target/measure/:
#   gnu             x86_64-unknown-linux-gnu, glibc 2.17 at most, glibc's allocator
#   musl            x86_64-unknown-linux-musl, static, musl's allocator
#   musl-mimalloc   x86_64-unknown-linux-musl, static, mimalloc (feature `mimalloc` of canon-cli)
# Workloads:
#   conform         canon conform run --requirements conformance/requirements.yaml, as the gate runs it
#   check           canon check --path $CHECK_FIXTURE: the protocol fixture with the most states
#                   that `canon check` accepts (574464 states)
# One run of a workload runs its command REPEAT times one after another: one command takes about
# 75 ms, too short to compare on a busy host.
#
# The script measures, so it runs without the `slot` prefix; each build inside it takes a slot of
# the pool (`slot --fetch` copies the binary back when the job ran on a build node), and every run
# is on this host. After one unmeasured warm-up run of each build on each workload, the runs go
# round-robin (gnu, musl, musl-mimalloc, gnu, ...) so a drift in the host's load reaches all
# three builds alike. Each run's wall time (s) and peak resident set (KiB) come from
# /usr/bin/time -f '%e %M', or, without GNU time, from zsh's `time` with TIMEFMT='%*E %M': both
# read the child's rusage. POSIX sh has no way to read a peak RSS.
#
# Prints the host, the commit, every run as `run <workload> <build> <index> <seconds> <KiB>`, a
# Markdown table of median, min and max per build and workload, the comparisons and the decision
# of the rule in docs/design/linux-release.md. The runs are also in target/measure/runs.txt.
set -eu

CHECK_FIXTURE=fixtures/check-across-protocol-files/clean/review.yaml
REPEAT=20
BUILDS="gnu musl musl-mimalloc"
WORKLOADS="conform check"

fail() {
    echo "measure-release: $*" >&2
    exit 1
}

runs=${1:-10}
case "$runs" in '' | *[!0-9]*) fail "usage: measure-release.sh [runs]" ;; esac
[ "$runs" -ge 1 ] || fail "usage: measure-release.sh [runs]"
cd "$(dirname "$0")/.."
out=target/measure
mkdir -p "$out"

if [ -x /usr/bin/time ]; then
    timer=gnu-time
elif command -v zsh >/dev/null 2>&1; then
    timer=zsh
else
    fail "needs /usr/bin/time (GNU time) or zsh to read a run's peak RSS"
fi

# pool <binary> <command...>: the build through the pool when there is one, and its binary back here.
pool() {
    binary=$1
    shift
    if [ -z "${SLOT_JOB:-}" ] && command -v slot >/dev/null 2>&1; then
        slot --timeout 30m --fetch "$binary" "$@"
    else
        "$@"
    fi
}

for build in $BUILDS; do
    case "$build" in
    gnu) set -- x86_64-unknown-linux-gnu system ;;
    musl) set -- x86_64-unknown-linux-musl system ;;
    musl-mimalloc) set -- x86_64-unknown-linux-musl mimalloc ;;
    esac
    rm -f "$out/canon-$build"
    pool "$out/canon-$build" sh scripts/package.sh binary "$1" "$2" "$out/canon-$build" >"$out/build-$build.log" 2>&1 ||
        fail "the $build build failed; see $out/build-$build.log"
    [ -x "$out/canon-$build" ] || fail "the $build build left no $out/canon-$build"
    echo "built: $build ($(grep "^allocator: " "$out/build-$build.log"))"
done

# measure <file> <command...>: run the command once with stdout discarded; write
# `<seconds> <KiB>` into file. A failing command stops the measurement.
measure() {
    file=$1
    shift
    status=0
    case "$timer" in
    gnu-time) /usr/bin/time -f '%e %M' -o "$file" "$@" >/dev/null 2>"$out/stderr.txt" || status=$? ;;
    zsh) zsh -fc 'TIMEFMT="%*E %M"; time "$@" >/dev/null 2>"$0"' "$out/stderr.txt" "$@" 2>"$file" || status=$? ;;
    esac
    if [ "$status" -ne 0 ]; then
        cat "$out/stderr.txt" >&2
        fail "\`$*\` exited ${status}"
    fi
    tail -n 1 "$file" >"$file.last"
    mv "$file.last" "$file"
}

workload() {
    case "$1" in
    conform) set -- "$out/canon-$2" conform run --requirements conformance/requirements.yaml ;;
    check) set -- "$out/canon-$2" check --path "$CHECK_FIXTURE" ;;
    esac
    measure "$out/one.txt" sh -c '
        n=$1
        shift
        i=0
        while [ "$i" -lt "$n" ]; do
            "$@" || exit $?
            i=$((i + 1))
        done' repeat "$REPEAT" "$@"
}

load() { cut -d' ' -f1-3 /proc/loadavg; }

commit=$(git rev-parse HEAD)
if [ -n "$(git status --porcelain --untracked-files=no)" ]; then tree=modified; else tree=clean; fi
echo "commit: $commit ($tree)"
echo "cpu: $(sed -n 's/^model name[[:space:]]*: //p' /proc/cpuinfo | head -n 1), $(nproc) logical CPUs"
echo "timer: $timer"
echo "workloads: conform = conform run --requirements conformance/requirements.yaml;" \
    "check = check --path $CHECK_FIXTURE; each $REPEAT times per run"
echo "load before: $(load)"

for w in $WORKLOADS; do
    for b in $BUILDS; do workload "$w" "$b"; done
done
: >"$out/runs.txt"
i=1
while [ "$i" -le "$runs" ]; do
    for w in $WORKLOADS; do
        for b in $BUILDS; do
            workload "$w" "$b"
            echo "run $w $b $i $(cat "$out/one.txt")" | tee -a "$out/runs.txt"
        done
    done
    i=$((i + 1))
done
echo "load after: $(load)"

# The table, the comparisons and the decision. A median of an even count is the mean of the two
# middle values. Two builds whose medians differ by less than the spread (max - min) of either
# show no measurable difference.
awk -v builds="$BUILDS" -v workloads="$WORKLOADS" '
    function sort(a, n,    i, j, t) {
        for (i = 2; i <= n; i++)
            for (j = i; j > 1 && a[j - 1] > a[j]; j--) {
                t = a[j]; a[j] = a[j - 1]; a[j - 1] = t
            }
    }
    function mid(a, n) { return n % 2 ? a[(n + 1) / 2] : (a[n / 2] + a[n / 2 + 1]) / 2 }
    {
        key = $2 SUBSEP $3
        n[key]++
        s[key, n[key]] = $5 + 0
        r[key, n[key]] = $6 + 0
    }
    END {
        nb = split(builds, B, " ")
        nw = split(workloads, W, " ")
        print ""
        print "| workload | build | runs | median s | min s | max s | median peak RSS KiB | max peak RSS KiB |"
        print "|---|---|---|---|---|---|---|---|"
        for (w = 1; w <= nw; w++)
            for (b = 1; b <= nb; b++) {
                key = W[w] SUBSEP B[b]
                c = n[key]
                for (i = 1; i <= c; i++) { t[i] = s[key, i]; m[i] = r[key, i] }
                sort(t, c)
                sort(m, c)
                med[key] = mid(t, c)
                spread[key] = t[c] - t[1]
                printf "| %s | %s | %d | %.3f | %.3f | %.3f | %d | %d |\n", W[w], B[b], c, med[key], t[1], t[c], mid(m, c), m[c]
            }
        print ""
        for (w = 1; w <= nw; w++)
            for (x = 1; x <= nb; x++)
                for (y = x + 1; y <= nb; y++) {
                    a = W[w] SUBSEP B[x]
                    z = W[w] SUBSEP B[y]
                    d = med[a] - med[z]
                    if (d < 0) d = -d
                    if (d < spread[a] || d < spread[z])
                        printf "%s: %s vs %s: no measurable difference (medians %.3f and %.3f s)\n", W[w], B[x], B[y], med[a], med[z]
                    else if (med[a] < med[z])
                        printf "%s: %s is faster than %s by %.1f %%\n", W[w], B[x], B[y], 100 * (med[z] - med[a]) / med[z]
                    else
                        printf "%s: %s is faster than %s by %.1f %%\n", W[w], B[y], B[x], 100 * (med[a] - med[z]) / med[a]
                }
        mi = 1
        for (w = 1; w <= nw; w++)
            if (med[W[w] SUBSEP "musl-mimalloc"] > med[W[w] SUBSEP "musl"]) mi = 0
        chosen = mi ? "musl-mimalloc" : "musl"
        musl = 1
        for (w = 1; w <= nw; w++)
            if (med[W[w] SUBSEP chosen] > 1.05 * med[W[w] SUBSEP "gnu"]) musl = 0
        print ""
        printf "decision: MUSL_ALLOCATOR=%s (the mimalloc median is %s)\n", mi ? "mimalloc" : "system", mi ? "not above the system allocator'"'"'s on every workload" : "above the system allocator'"'"'s on a workload"
        printf "decision: default %s (the %s median is %s)\n", musl ? "musl" : "gnu", chosen, musl ? "at most 5 % above gnu'"'"'s on every workload" : "more than 5 % above gnu'"'"'s on a workload"
    }' "$out/runs.txt"

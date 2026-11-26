# Linux release archives: which build is the default, and mimalloc

**Status:** Measured (story:musl-workload-measurement, epic:linux-release-archives)
**Date:** 2026-11-26

## 1. Question

Each release carries two Linux x86_64 archives of `canon`: `x86_64-unknown-linux-gnu`, linked
against glibc 2.17 at most, and `x86_64-unknown-linux-musl`, fully static
(`scripts/package.sh`). musl's own allocator is known to be slower than glibc's under
allocation-heavy work, so before the installation page names musl first, the three candidate builds
are measured on one workload: gnu, musl with musl's allocator, and musl with mimalloc (the
`mimalloc` cargo feature of `canon-cli`, off by default).

## 2. Rule

The rule was fixed before the numbers were taken. It reads the medians of 10 runs per build and
workload.

- **mimalloc.** The musl archive links mimalloc only if the median of musl with mimalloc is not
  above the median of musl with its own allocator on both workloads. Otherwise the feature stays
  off. `MUSL_ALLOCATOR` in `scripts/package.sh` carries the result.
- **Default.** The installation page names the musl archive first only if the chosen musl build
  (with or without mimalloc, as above) has a median at most 5 % above gnu's on both workloads
  (a faster musl passes). Otherwise it names the gnu archive first and offers musl for hosts whose
  glibc is older than 2.17 or that have no glibc.
- **Ranking.** When the medians of two builds differ by less than the spread (max − min) of either
  build, the comparison says "no measurable difference" instead of a ranking. The two decisions
  above still read the medians.

`crates/canon/tests/release_decision.rs` reads the runs in section 5, recomputes the medians and
fails when `MUSL_ALLOCATOR` or the installation page does not follow this rule.

## 3. Method

`scripts/measure-release.sh` (run without the `slot` prefix; each build inside it takes a build
slot):

- builds the three binaries with `scripts/package.sh binary <target> <allocator> <out>`, the same
  build steps and checks as the release archives (gnu with `--target
  x86_64-unknown-linux-gnu.2.17`, musl static, mimalloc checked present or absent);
- workload `conform`: `canon conform run --requirements conformance/requirements.yaml`, the
  invocation of the gate (35 scenarios);
- workload `check`: `canon check --path fixtures/check-across-protocol-files/clean/review.yaml`,
  the protocol fixture with the most states that `canon check` accepts (574464 states; the next
  largest accepted one, `fixtures/investigation/simulate/children-count/protocol.yaml`, has 348160
  states);
- one command of either workload takes about 75 ms, so one run executes it 20 times in a row
  (N = 20) and the run's time is the time of those 20;
- one unmeasured warm-up run of each build on each workload, then 10 rounds; each round runs
  `conform` with gnu, musl, musl-mimalloc, then `check` with gnu, musl, musl-mimalloc, so a drift
  in the host's load reaches all three builds alike;
- per run, wall time in seconds and peak resident set in KiB from the child's rusage (GNU time
  `%e %M` when installed; on this host it is not, so zsh's `time` with `TIMEFMT='%*E %M'`).

## 4. Host and commit

Commit measured: `cf13ea94ec6dfcd1a8e90ef6b1b622fbce2173d2`

- CPU: Intel Core i9-10900K at 3.70 GHz, 10 cores, 20 logical CPUs; about 63 GiB memory; Linux 6.18.
- A shared workstation: other build jobs and sessions ran during the measurement, so it is not an
  idle host. Load average (1, 5, 15 min) before: 2.37 1.86 2.98; after: 4.25 2.77 3.19.
- Rounds 5 and 6 caught a load spike that slowed one run of each build (gnu `conform` 2.181 s in
  round 5, musl `check` 2.786 s in round 5, musl `conform` 2.100 s and musl-mimalloc `conform`
  1.566 s in round 6). These single runs make up most of each affected spread.

## 5. Results

| workload | build | runs | median s | min s | max s | median peak RSS KiB | max peak RSS KiB |
|---|---|---|---|---|---|---|---|
| conform | gnu | 10 | 1.564 | 1.527 | 2.181 | 11732 | 11844 |
| conform | musl | 10 | 1.684 | 1.621 | 2.100 | 16184 | 19000 |
| conform | musl-mimalloc | 10 | 1.260 | 1.191 | 1.566 | 26926 | 26968 |
| check | gnu | 10 | 1.512 | 1.468 | 1.594 | 8902 | 9016 |
| check | musl | 10 | 2.031 | 1.961 | 2.786 | 9484 | 11284 |
| check | musl-mimalloc | 10 | 1.200 | 1.133 | 1.251 | 18168 | 18356 |

Times are for 20 commands per run. Comparisons, by the ranking rule:

- `conform`: gnu vs musl, gnu vs musl-mimalloc, musl vs musl-mimalloc: no measurable difference
  (each pair's medians differ by less than one build's spread, which the round 5 and 6 outliers
  widen).
- `check`: gnu vs musl: no measurable difference by the rule (medians 1.512 and 2.031 s; musl's
  spread is 0.825 s because of its one 2.786 s run; its other nine runs are all above gnu's
  slowest).
- `check`: musl-mimalloc is faster than gnu by 20.6 %, and faster than musl by 40.9 %.
- Peak RSS: mimalloc holds about twice the resident memory of the other two builds (27 MiB against
  12 to 16 MiB on `conform`, 18 MiB against 9 MiB on `check`).

Every run, as the script prints it (`run <workload> <build> <round> <seconds> <peak RSS KiB>`):

```text
run conform gnu 1 1.528 11812
run conform musl 1 1.621 15368
run conform musl-mimalloc 1 1.259 26968
run check gnu 1 1.501 8924
run check musl 1 2.017 11284
run check musl-mimalloc 1 1.196 16124
run conform gnu 2 1.559 11672
run conform musl 2 1.726 19000
run conform musl-mimalloc 2 1.227 26952
run check gnu 2 1.502 8932
run check musl 2 2.019 9484
run check musl-mimalloc 2 1.217 16124
run conform gnu 3 1.606 11672
run conform musl 3 1.682 15720
run conform musl-mimalloc 3 1.296 26956
run check gnu 3 1.594 8848
run check musl 3 2.063 9484
run check musl-mimalloc 3 1.250 18148
run conform gnu 4 1.602 11672
run conform musl 4 1.731 16520
run conform musl-mimalloc 4 1.311 26964
run check gnu 4 1.520 8876
run check musl 4 2.008 9484
run check musl-mimalloc 4 1.205 18168
run conform gnu 5 2.181 11844
run conform musl 5 1.885 17080
run conform musl-mimalloc 5 1.206 26968
run check gnu 5 1.468 8936
run check musl 5 2.786 9484
run check musl-mimalloc 5 1.175 18168
run conform gnu 6 1.548 11832
run conform musl 6 2.100 15848
run conform musl-mimalloc 6 1.566 24928
run check gnu 6 1.577 8860
run check musl 6 2.043 9356
run check musl-mimalloc 6 1.155 18356
run conform gnu 7 1.527 11824
run conform musl 7 1.684 17112
run conform musl-mimalloc 7 1.214 26896
run check gnu 7 1.511 8916
run check musl 7 1.983 9356
run check musl-mimalloc 7 1.171 16320
run conform gnu 8 1.558 11672
run conform musl 8 1.621 15432
run conform musl-mimalloc 8 1.191 26900
run check gnu 8 1.546 8888
run check musl 8 2.053 11160
run check musl-mimalloc 8 1.251 18356
run conform gnu 9 1.569 11724
run conform musl 9 1.685 15752
run conform musl-mimalloc 9 1.262 26892
run check gnu 9 1.513 9016
run check musl 9 2.047 9196
run check musl-mimalloc 9 1.231 18356
run conform gnu 10 1.591 11740
run conform musl 10 1.673 16664
run conform musl-mimalloc 10 1.270 26896
run check gnu 10 1.495 8856
run check musl 10 1.961 11000
run check musl-mimalloc 10 1.133 18344
```

## 6. Decision

- **mimalloc: on.** musl-mimalloc's median is not above musl's on either workload (1.260 against
  1.684 s on `conform`, 1.200 against 2.031 s on `check`). `scripts/package.sh` sets
  `MUSL_ALLOCATOR=mimalloc`, so the musl archive is built with `--features mimalloc`. The gnu
  archive keeps glibc's allocator.
- **Default: musl.** The chosen musl build, musl-mimalloc, has a median at most 5 % above gnu's
  on both workloads; it is below gnu's on both (1.260 against 1.564 s, 1.200 against 1.512 s). The
  installation page names the musl archive first and the gnu archive second.
- **Cost.** The musl archive's peak resident memory is about twice the gnu archive's on these
  workloads (section 5).

Measured on one shared host under load. A later measurement that changes either median comparison
changes `MUSL_ALLOCATOR` and the installation page with the new table, by the same rule.

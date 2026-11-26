#!/bin/sh
# Build, check and archive the `canon` binary for one Linux x86_64 target. The packaging workflow
# (.github/workflows/package.yml) and `task package -- <target>` both run this file, so a local
# package is built the way the release one is.
#
#   package.sh <target>                       build, check, smoke-run and archive into
#                                             target/package/canon-<version>-<target>.tar.gz and
#                                             its line in target/package/SHA256SUMS
#   package.sh check-glibc <binary> [floor]   fail when <binary> needs a GLIBC_ version above
#                                             floor (default 2.17), naming each such symbol
#   package.sh check-static <binary>          fail when <binary> has a program interpreter
#   package.sh binary <target> <allocator> <out>
#                                             build and check the release binary with allocator
#                                             `system` or `mimalloc` (musl only) and copy it to
#                                             <out>; scripts/measure-release.sh uses it
#
# <target> is x86_64-unknown-linux-gnu (linked against glibc 2.17 at most) or
# x86_64-unknown-linux-musl (static). Both build with cargo-zigbuild and zig, pinned below by
# version and SHA-256 and unpacked under target/package-tools/. Needs curl, tar, xz, sha256sum,
# objdump, readelf, rustup and git.
set -eu

ZIG_VERSION=0.16.0
ZIG_SHA256=70e49664a74374b48b51e6f3fdfbf437f6395d42509050588bd49abe52ba3d00
ZIGBUILD_VERSION=0.24.1
ZIGBUILD_SHA256=f09b180a517af0e4dd4bffc921bb88e5ab705452a9dd948c61e489b3020c68c2
GLIBC_FLOOR=2.17
# The allocator of the musl archive: `system` (musl's own) or `mimalloc` (the `mimalloc` feature
# of canon-cli). The gnu archive always uses glibc's. docs/design/linux-release.md holds the
# measurement that decides it; crates/canon/tests/release_decision.rs holds this line to it.
MUSL_ALLOCATOR=mimalloc

fail() {
    echo "package: $*" >&2
    exit 1
}

# check_glibc <binary> [floor]: every GLIBC_ version in `objdump -T` is at most floor. A binary
# that names no GLIBC_ version at all is refused too: it is not the gnu build.
check_glibc() {
    binary=$1
    floor=${2:-$GLIBC_FLOOR}
    [ -f "$binary" ] || fail "check-glibc: $binary is not a file"
    symbols=$(objdump -T "$binary" 2>&1) || fail "check-glibc: objdump -T cannot read $binary: $symbols"
    printf '%s\n' "$symbols" | awk -v floor="$floor" -v binary="$binary" '
        function newer(a, b,    x, y, i) {
            split(a, x, ".")
            split(b, y, ".")
            for (i = 1; i <= 4; i++)
                if (x[i] + 0 != y[i] + 0)
                    return x[i] + 0 > y[i] + 0
            return 0
        }
        {
            for (i = 1; i < NF; i++) {
                field = $i
                gsub(/[()]/, "", field)
                if (field !~ /^GLIBC_[0-9]+(\.[0-9]+)*$/)
                    continue
                version = substr(field, 7)
                seen++
                if (highest == "" || newer(version, highest)) {
                    highest = version
                    top = $NF
                }
                if (newer(version, floor))
                    above[++n] = "GLIBC_" version " (symbol " $NF ")"
            }
        }
        END {
            if (!seen) {
                print "package: check-glibc: " binary " names no GLIBC_ version, so it is not linked against glibc" > "/dev/stderr"
                exit 1
            }
            if (n) {
                print "package: check-glibc: " binary " needs GLIBC_" highest " (symbol " top "), above the floor GLIBC_" floor "; every symbol above it:" > "/dev/stderr"
                for (i = 1; i <= n; i++)
                    print "  " above[i] > "/dev/stderr"
                exit 1
            }
            print "check-glibc: " binary ": highest GLIBC_" highest " (symbol " top "), floor GLIBC_" floor
        }'
}

# check_static <binary>: `readelf -l` reads program headers and none of them is INTERP.
check_static() {
    binary=$1
    [ -f "$binary" ] || fail "check-static: $binary is not a file"
    headers=$(readelf -lW "$binary" 2>&1) || fail "check-static: readelf -l cannot read $binary: $headers"
    case "$headers" in
    *"Program Headers:"*) ;;
    *) fail "check-static: readelf -l shows no program headers in $binary: $headers" ;;
    esac
    if printf '%s\n' "$headers" | grep -q '^ *INTERP '; then
        interpreter=$(printf '%s\n' "$headers" | sed -n 's/.*\[Requesting program interpreter: \(.*\)\]$/\1/p')
        fail "check-static: $binary has a program interpreter (INTERP ${interpreter:-unreadable}), so it is not static"
    fi
    echo "check-static: $binary has no INTERP"
}

# fetch <url> <sha256>: unpack the .tar.xz at url into target/package-tools/<sha256>/ once, after
# checking its SHA-256; print that directory.
fetch() {
    into=target/package-tools/$2
    if [ ! -d "$into" ]; then
        mkdir -p target/package-tools
        curl --fail --silent --show-error --location --output "$into.$$.tar.xz" "$1"
        echo "$2  $into.$$.tar.xz" | sha256sum --check --strict >&2 ||
            fail "$1 does not have the SHA-256 $2"
        mkdir "$into.$$"
        tar -xJf "$into.$$.tar.xz" -C "$into.$$"
        rm -f "$into.$$.tar.xz"
        mv "$into.$$" "$into"
    fi
    echo "$PWD/$into"
}

known_target() {
    case "$1" in
    x86_64-unknown-linux-gnu | x86_64-unknown-linux-musl) ;;
    *) fail "unknown target \`$1\`: give x86_64-unknown-linux-gnu or x86_64-unknown-linux-musl" ;;
    esac
}

# known_allocator <target> <allocator>: `system` for either target, `mimalloc` for musl only.
known_allocator() {
    case "$2" in
    system) ;;
    mimalloc)
        [ "$1" = x86_64-unknown-linux-musl ] ||
            fail "allocator mimalloc is used by only the musl build, not $1"
        ;;
    *) fail "unknown allocator \`$2\`: give system or mimalloc" ;;
    esac
}

# compile <target> <allocator>: build target/<target>/release/canon with the pinned tools from the
# repository root, check its linking and that it holds mimalloc exactly when asked for.
compile() {
    target=$1
    allocator=$2
    zig=$(fetch "https://ziglang.org/download/$ZIG_VERSION/zig-x86_64-linux-$ZIG_VERSION.tar.xz" "$ZIG_SHA256")
    zigbuild=$(fetch "https://github.com/rust-cross/cargo-zigbuild/releases/download/v$ZIGBUILD_VERSION/cargo-zigbuild-x86_64-unknown-linux-musl.tar.xz" "$ZIGBUILD_SHA256")
    PATH="$zig/zig-x86_64-linux-$ZIG_VERSION:$zigbuild/cargo-zigbuild-x86_64-unknown-linux-musl:$PATH"
    export PATH
    # The tools' caches stay in this tree; no host rustflags reach the release build, as on CI.
    CARGO_ZIGBUILD_CACHE_DIR=$PWD/target/package-tools/cache
    ZIG_GLOBAL_CACHE_DIR=$PWD/target/package-tools/zig-cache
    ZIG_LOCAL_CACHE_DIR=$PWD/target/package-tools/zig-cache
    CARGO_ENCODED_RUSTFLAGS=
    export CARGO_ZIGBUILD_CACHE_DIR ZIG_GLOBAL_CACHE_DIR ZIG_LOCAL_CACHE_DIR CARGO_ENCODED_RUSTFLAGS
    [ "$(zig version)" = "$ZIG_VERSION" ] || fail "zig $ZIG_VERSION is not the zig on PATH"
    [ "$(cargo-zigbuild --version)" = "cargo-zigbuild $ZIGBUILD_VERSION" ] ||
        fail "cargo-zigbuild $ZIGBUILD_VERSION is not the cargo-zigbuild on PATH"

    rustup target add "$target"
    binary=target/$target/release/canon
    case "$target" in
    x86_64-unknown-linux-gnu)
        cargo zigbuild --locked --release -p canon-cli --target x86_64-unknown-linux-gnu.2.17
        check_glibc "$binary"
        ;;
    x86_64-unknown-linux-musl)
        set --
        if [ "$allocator" = mimalloc ]; then set -- --features mimalloc; fi
        cargo zigbuild --locked --release -p canon-cli --target x86_64-unknown-linux-musl "$@"
        check_static "$binary"
        ;;
    esac
    # mimalloc's own messages name it; musl's and glibc's allocators do not.
    if grep -q 'mimalloc' "$binary"; then linked=mimalloc; else linked=system; fi
    [ "$linked" = "$allocator" ] ||
        fail "$binary was built with allocator $allocator but holds the $linked allocator"
    echo "allocator: $binary uses the $allocator allocator"
}

build() {
    target=$1
    known_target "$target"
    allocator=system
    if [ "$target" = x86_64-unknown-linux-musl ]; then allocator=$MUSL_ALLOCATOR; fi
    known_allocator "$target" "$allocator"
    cd "$(dirname "$0")/.."
    version=$(sed -n '/^\[workspace\.package\]/,/^\[/s/^version = "\(.*\)"$/\1/p' Cargo.toml)
    [ -n "$version" ] || fail "Cargo.toml has no [workspace.package] version"
    compile "$target" "$allocator"

    name=canon-$version-$target
    out=target/package
    mkdir -p "$out/stage-$target/$name" "$out/smoke-$target"
    cp "$binary" LICENSE README.md "$out/stage-$target/$name/"
    epoch=$(git log -1 --format=%ct)
    tar --sort=name --mtime="@$epoch" --owner=0 --group=0 --numeric-owner \
        --mode='u+rwX,go+rX,go-w' -C "$out/stage-$target" -cf - "$name" |
        gzip -n -9 >"$out/$name.tar.gz"
    listed=$(tar -tzf "$out/$name.tar.gz" | tr '\n' ' ')
    [ "$listed" = "$name/ $name/LICENSE $name/README.md $name/canon " ] ||
        fail "$name.tar.gz holds $listed, not the binary, LICENSE and README.md"

    # Smoke run of the archived binary, not the one in target/.
    tar -xzf "$out/$name.tar.gz" -C "$out/smoke-$target"
    packaged=$out/smoke-$target/$name/canon
    printed=$("$packaged" --version)
    [ "$printed" = "canon $version" ] ||
        fail "the packaged $target binary prints \`$printed\`, not \`canon $version\`"
    echo "smoke: $printed"
    "$packaged" conform run --requirements conformance/requirements.yaml ||
        fail "conform run of the packaged $target binary failed"

    line=$(cd "$out" && sha256sum "$name.tar.gz")
    {
        if [ -f "$out/SHA256SUMS" ]; then awk -v file="$name.tar.gz" '$2 != file' "$out/SHA256SUMS"; fi
        echo "$line"
    } | sort -k2 >"$out/SHA256SUMS.$$"
    mv "$out/SHA256SUMS.$$" "$out/SHA256SUMS"
    echo "packaged: $out/$name.tar.gz ($(wc -c <"$out/$name.tar.gz") bytes)"
    echo "$line"
}

case "${1:-}" in
check-glibc)
    [ $# -ge 2 ] || fail "usage: package.sh check-glibc <binary> [floor]"
    check_glibc "$2" "${3:-$GLIBC_FLOOR}"
    ;;
check-static)
    [ $# -ge 2 ] || fail "usage: package.sh check-static <binary>"
    check_static "$2"
    ;;
binary)
    [ $# -eq 4 ] || fail "usage: package.sh binary <target> system|mimalloc <out>"
    known_target "$2"
    known_allocator "$2" "$3"
    case "$4" in /*) out=$4 ;; *) out=$PWD/$4 ;; esac
    cd "$(dirname "$0")/.."
    compile "$2" "$3"
    mkdir -p "$(dirname "$out")"
    cp "$binary" "$out"
    echo "binary: $out"
    ;;
*)
    [ $# -eq 1 ] || fail "usage: package.sh x86_64-unknown-linux-gnu | x86_64-unknown-linux-musl"
    build "$1"
    ;;
esac

#!/bin/sh
# Compiles and lints the helper for Windows on a Mac (rust-helper workspace-and-crates.md, "Gates"): clippy with the
# workspace's lint table for x86_64-pc-windows-msvc and aarch64-pc-windows-msvc, tests included. It checks types and lints
# only: nothing links and nothing runs, so Windows behaviour stays unobserved until the Windows acceptance run or a
# release's Windows jobs (helper-release.yml; the pull requests' Windows jobs, helper.yml, went on 2026-10-09).
# tauri-build compiles the Windows resource file through embed-resource, which calls llvm-rc for an msvc target; a
# check never links it, so a stand-in that answers the version probe and writes an empty .res is enough. mlua-sys
# compiles Lua 5.1's C (lua-src, questpump-engine's `vendored`) through cc, which needs cl.exe and the Windows SDK's
# headers for an msvc target: the stand-ins CC_<target> and AR_<target> name answer cc's probes as cl.exe does (no
# __clang__ or __GNUC__ on -E, `-?` accepted) and write an empty object at -Fo and an empty library at -out:; a
# release's Windows runners compile the real C. They live at one path under target/, so the build scripts that watch
# PATH and those variables see the same values on every run and nothing rebuilds.
set -eu
cd "$(dirname "$0")/.."
stub="$PWD/target/windows-check"
mkdir -p "$stub"
cat >"$stub/llvm-rc" <<'EOF'
#!/bin/sh
for a in "$@"; do [ "$a" = "/?" ] && { echo "OVERVIEW: LLVM Resource Converter"; echo "  --no-preprocess"; exit 0; }; done
out=""; prev=""
for a in "$@"; do [ "$prev" = "/fo" ] && out="$a"; prev="$a"; done
[ -n "$out" ] && : >"$out"
exit 0
EOF
cat >"$stub/cl" <<'EOF'
#!/bin/sh
for a in "$@"; do case "$a" in -Fo*) : >"${a#-Fo}" ;; esac; done
exit 0
EOF
cat >"$stub/lib" <<'EOF'
#!/bin/sh
for a in "$@"; do case "$a" in -out:*) : >"${a#-out:}" ;; esac; done
exit 0
EOF
chmod +x "$stub/llvm-rc" "$stub/cl" "$stub/lib"
for target in x86_64-pc-windows-msvc aarch64-pc-windows-msvc; do
	rustup target add "$target" >/dev/null
	t=$(echo "$target" | tr - _)
	env PATH="$stub:$PATH" "CC_$t=$stub/cl" "AR_$t=$stub/lib" \
		cargo clippy --target "$target" --workspace --all-targets --all-features -- -D warnings
done

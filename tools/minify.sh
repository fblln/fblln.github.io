#!/bin/sh
# Trunk post_build hook: minify the JS/CSS trunk emits, using esbuild.
# Trunk's bundled minifier (minify-js 0.6.0) can't parse wasm-bindgen's
# `export { __wbg_init as default }`, so trunk minify is off (Trunk.toml
# minify = "never") and we do it here instead. Requires no_sri = true, since
# rewriting these files would otherwise invalidate trunk's integrity hashes.
set -eu

for f in "$TRUNK_STAGING_DIR"/*.js "$TRUNK_STAGING_DIR"/*.css; do
    [ -e "$f" ] || continue
    esbuild "$f" --minify --allow-overwrite --outfile="$f"
done

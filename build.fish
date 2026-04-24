#!/usr/bin/env fish

# build.fish — compile fetch for Linux and Windows, then collect outputs

set OUT (realpath (dirname (status filename)))/output/fetch
rm -rf $OUT
mkdir -p $OUT

echo "==> Building Linux (x86_64-unknown-linux-gnu) ..."
cargo build --release
or begin
    echo "Linux build failed"
    exit 1
end

echo "==> Building Windows (x86_64-pc-windows-gnu) ..."
cargo build --release --target x86_64-pc-windows-gnu
or begin
    echo "Windows build failed"
    exit 1
end

echo "==> Copying outputs to $OUT ..."

# Linux shared library — Lua expects 'fetch.so'
cp target/release/libcore.so        $OUT/core.so

# Windows DLL — Lua expects 'fetch.dll'
cp target/x86_64-pc-windows-gnu/release/core.dll  $OUT/core.dll

# Lua glue (works on both platforms)
cp init.lua  $OUT/init.lua

echo ""
echo "Done. Output:"
ls -lh $OUT

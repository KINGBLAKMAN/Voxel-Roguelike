#!/bin/bash
# Builds the Windows exes locally (cross-compiled with the x86_64-pc-windows-gnu target) and zips them:
#   dist/Emberwild-<v>-windows.zip      the game
#   dist/Emberwild-<v>-windows-dev.zip  the game plus the balance bot and test hooks
# Needs: rustup target add x86_64-pc-windows-gnu, and the mingw-w64 linker.
set -euo pipefail
cd "$(dirname "$0")"
v=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
out=dist; rm -rf "$out"; mkdir -p "$out/Emberwild-$v" "$out/Emberwild-$v-dev"
t=x86_64-pc-windows-gnu
cargo build --release --target $t
cp target/$t/release/emberwild.exe "$out/Emberwild-$v/Emberwild-$v.exe"
cargo build --release --target $t --features dev
cp target/$t/release/emberwild.exe "$out/Emberwild-$v-dev/Emberwild-$v-dev.exe"
for d in "Emberwild-$v" "Emberwild-$v-dev"; do cp ../README.md ../RELEASE_NOTES.md "$out/$d/"; done
printf '@echo off\r\nrem Plays 3 full runs per class on its own and writes balance-*.csv and a summary in this folder.\r\ncd /d "%%~dp0"\r\nstart "" /wait Emberwild-%s-dev.exe --bot "runs=3;minutes=20;turbo=8"\r\necho Done. See the balance-*-summary.txt file in this folder.\r\npause\r\n' "$v" > "$out/Emberwild-$v-dev/Run balance bot.bat"
(cd "$out" && zip -qr "Emberwild-$v-windows.zip" "Emberwild-$v" && zip -qr "Emberwild-$v-windows-dev.zip" "Emberwild-$v-dev")
ls -la "$out"/*.zip

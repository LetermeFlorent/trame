#!/bin/sh
# Wraps a built trame binary into trame.app.
# Usage: packaging/macos/bundle.sh <binary> <version> <output folder>
set -eu

binary=$1
version=$2
out=$3
root=$(cd "$(dirname "$0")/../.." && pwd)
app="$out/trame.app"

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$binary" "$app/Contents/MacOS/trame"

iconset=$(mktemp -d)/trame.iconset
mkdir -p "$iconset"
for size in 16 32 128 256 512; do
  sips -z $size $size "$root/assets/logo-512.png" --out "$iconset/icon_${size}x${size}.png" > /dev/null
  double=$((size * 2))
  if [ $double -le 512 ]; then
    sips -z $double $double "$root/assets/logo-512.png" --out "$iconset/icon_${size}x${size}@2x.png" > /dev/null
  fi
done
cp "$root/assets/logo-512.png" "$iconset/icon_256x256@2x.png"
iconutil -c icns "$iconset" -o "$app/Contents/Resources/trame.icns"
rm -rf "$(dirname "$iconset")"

sed "s/@VERSION@/$version/g" "$root/packaging/macos/Info.plist" > "$app/Contents/Info.plist"

# No Developer ID here: an ad-hoc signature is the least Apple Silicon accepts.
codesign --force --deep --sign - "$app"

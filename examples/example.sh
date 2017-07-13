#!/bin/bash

DST="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
APPDIR="RubrailExample.app"


echo "Building OS X app..."

rm -rf "$DST/$APPDIR"
mkdir "$DST/$APPDIR/"
mkdir "$DST/$APPDIR/Contents/"
mkdir "$DST/$APPDIR/Contents/Resources/"
mkdir "$DST/$APPDIR/Contents/MacOS/"

cp -a "$DST/example" "$DST/$APPDIR/Contents/MacOS/"
cp -a "$DST/icon.png" "$DST/$APPDIR/Contents/Resources/"

cat > "$DST/$APPDIR/Contents/Info.plist" << EOF
{
   CFBundleName = rubrail;
   CFBundleDisplayName = RubrailExample;
   CFBundleIdentifier = "com.trevorbentley.rubrail";
   CFBundleExecutable = example;
   CFBundleIconFile = "rubrail.icns";

   CFBundleVersion = "0.0.2";
   CFBundleInfoDictionaryVersion = "6.0";
   CFBundlePackageType = APPL;
   CFBundleSignature = xxxx;

   LSMinimumSystemVersion = "10.10.0";
}
EOF
echo "Done!"

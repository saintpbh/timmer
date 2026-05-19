#!/bin/bash
set -e

# App name
APP_NAME="PresentationTimer"
APP_BUNDLE="${APP_NAME}.app"
DEST_DIR="$HOME/Desktop"

echo "Building release version..."
cargo build --release

echo "Creating App Bundle..."
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Copy binary
cp target/release/presentation-timer "$APP_BUNDLE/Contents/MacOS/$APP_NAME"

# Copy icon
cp AppIcon.icns "$APP_BUNDLE/Contents/Resources/AppIcon.icns"

# Create Info.plist
cat > "$APP_BUNDLE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.example.presentationtimer</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon.icns</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# Copy to Desktop
echo "Moving to Desktop..."
rm -rf "$DEST_DIR/$APP_BUNDLE"
mv "$APP_BUNDLE" "$DEST_DIR/"

echo "Successfully created and moved $APP_NAME.app to Desktop!"

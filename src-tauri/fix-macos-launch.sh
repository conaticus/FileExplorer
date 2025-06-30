#!/bin/bash

# Automatischer Fix für macOS Launch Services Problem
# Führe dieses Script nach 'cargo tauri build' aus

APP_NAME="file-explorer"
BUILD_DIR="/Users/lauritzwiebusch/WebstormProjects/FileExplorer/target/release/bundle/macos"
APP_PATH="$BUILD_DIR/$APP_NAME.app"

if [ ! -d "$APP_PATH" ]; then
    echo "❌ App nicht gefunden: $APP_PATH"
    echo "Stelle sicher dass 'cargo tauri build' erfolgreich war"
    exit 1
fi

echo "🔧 Fixe Launch Services Problem für $APP_NAME..."

# 1. Benenne Original Binary um
MACOS_DIR="$APP_PATH/Contents/MacOS"
if [ -f "$MACOS_DIR/src-tauri" ] && [ ! -f "$MACOS_DIR/src-tauri-real" ]; then
    echo "📦 Benenne Original Binary um..."
    mv "$MACOS_DIR/src-tauri" "$MACOS_DIR/src-tauri-real"
fi

# 2. Erstelle Wrapper
echo "🛠️  Erstelle Launch-Wrapper..."
cat > "$MACOS_DIR/src-tauri" << 'EOF'
#!/bin/bash

# macOS Launch Services Fix - Simuliert Terminal-Umgebung
export TERM="xterm-256color"
export TERM_PROGRAM="Apple_Terminal"
export SHELL="/bin/zsh"
export XPC_FLAGS="0x0"
export XPC_SERVICE_NAME="0"
export __CFBundleIdentifier="com.apple.Terminal"

# Vollständiger PATH
export PATH="/Library/Frameworks/Python.framework/Versions/3.11/bin:/opt/local/bin:/opt/local/sbin:/opt/homebrew/bin:/opt/homebrew/sbin:/usr/local/bin:/System/Cryptexes/App/usr/bin:/usr/bin:/bin:/usr/sbin:/sbin:/var/run/com.apple.security.cryptexd/codex.system/bootstrap/usr/local/bin:/var/run/com.apple.security.cryptexd/codex.system/bootstrap/usr/bin:/var/run/com.apple.security.cryptexd/codex.system/bootstrap/usr/appleinternal/bin:/Library/Apple/usr/bin"

cd "$(dirname "$0")"
exec ./src-tauri-real "$@"
EOF

# 3. Mache Wrapper ausführbar
chmod +x "$MACOS_DIR/src-tauri"

# 4. Fixe Info.plist
PLIST="$APP_PATH/Contents/Info.plist"
echo "⚙️  Aktualisiere Info.plist..."

# Entferne problematische Keys falls vorhanden
/usr/libexec/PlistBuddy -c "Delete :LSRequiresCarbon" "$PLIST" 2>/dev/null || true

# Füge benötigte Keys hinzu
/usr/libexec/PlistBuddy -c "Add :LSUIElement bool false" "$PLIST" 2>/dev/null || true
/usr/libexec/PlistBuddy -c "Add :LSBackgroundOnly bool false" "$PLIST" 2>/dev/null || true
/usr/libexec/PlistBuddy -c "Add :NSPrincipalClass string NSApplication" "$PLIST" 2>/dev/null || true
/usr/libexec/PlistBuddy -c "Add :LSApplicationCategoryType string public.app-category.utilities" "$PLIST" 2>/dev/null || true

# 5. Signiere App neu
echo "🔏 Signiere App neu..."
codesign --force --deep --sign - "$APP_PATH"

echo "✅ Fix erfolgreich angewendet!"
echo "📱 App kann jetzt normal über Finder gestartet werden"
echo ""
echo "🚀 Zum Testen:"
echo "   open '$APP_PATH'"
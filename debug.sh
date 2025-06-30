#!/bin/bash

# Detailliertes Debug-Script für Tauri v2 App
APP_NAME="file-explorer"
APP_PATH="./target/release/bundle/macos/$APP_NAME.app"

echo "🔍 Tauri App Debug Analysis"
echo "=============================="
echo ""

# 1. Build-Status prüfen
echo "1. Build-Status:"
if [ ! -d "$APP_PATH" ]; then
    echo "❌ App bundle NICHT gefunden: $APP_PATH"
    echo "   Führen Sie zuerst aus: cargo tauri build"
    exit 1
else
    echo "✅ App bundle gefunden: $APP_PATH"
fi

# 2. Bundle-Struktur prüfen
echo ""
echo "2. Bundle-Struktur:"
EXECUTABLE_PATH="$APP_PATH/Contents/MacOS/src-tauri"
if [ -f "$EXECUTABLE_PATH" ]; then
    echo "✅ Executable gefunden: $EXECUTABLE_PATH"
    echo "   Größe: $(ls -lh "$EXECUTABLE_PATH" | awk '{print $5}')"
    echo "   Berechtigungen: $(ls -l "$EXECUTABLE_PATH" | awk '{print $1}')"
else
    echo "❌ Executable NICHT gefunden: $EXECUTABLE_PATH"
    echo "   Verfügbare Dateien in MacOS/:"
    ls -la "$APP_PATH/Contents/MacOS/" 2>/dev/null || echo "   Ordner nicht gefunden"
    exit 1
fi

# 3. Info.plist prüfen
echo ""
echo "3. Info.plist Analyse:"
INFO_PLIST="$APP_PATH/Contents/Info.plist"
if [ -f "$INFO_PLIST" ]; then
    echo "✅ Info.plist gefunden"
    echo "   Bundle Identifier: $(plutil -p "$INFO_PLIST" | grep CFBundleIdentifier | cut -d'"' -f4)"
    echo "   Bundle Name: $(plutil -p "$INFO_PLIST" | grep CFBundleName | cut -d'"' -f4)"
    echo "   LSUIElement: $(plutil -p "$INFO_PLIST" | grep LSUIElement || echo "   LSUIElement: nicht gesetzt")"
else
    echo "❌ Info.plist nicht gefunden"
fi

# 4. Console Logs vor Start löschen
echo ""
echo "4. Console Logs zurücksetzen..."
log show --predicate 'process == "file-explorer"' --last 1s >/dev/null 2>&1

# 5. Direkter Start testen
echo ""
echo "5. Direkter Start Test:"
echo "   Befehl: $EXECUTABLE_PATH"
echo "   Umgebung:"
echo "     PATH Länge: $(echo $PATH | wc -c | tr -d ' ')"
echo "     DISPLAY: ${DISPLAY:-'nicht gesetzt'}"
echo "     HOME: ${HOME:-'nicht gesetzt'}"
echo ""

# Direkter Start mit detailliertem Output
echo "   Starte App direkt..."
"$EXECUTABLE_PATH" &
DIRECT_PID=$!
echo "   PID: $DIRECT_PID"

# 3 Sekunden warten und Status prüfen
sleep 3
if kill -0 $DIRECT_PID 2>/dev/null; then
    echo "   ✅ Direkter Start: App läuft (PID: $DIRECT_PID)"

    # Prüfen ob WebView lädt
    sleep 2
    if pgrep -f "$APP_NAME" >/dev/null; then
        echo "   ✅ App-Prozess aktiv"
    else
        echo "   ⚠️  App-Prozess nicht mehr aktiv"
    fi

    # App beenden
    kill $DIRECT_PID 2>/dev/null
    wait $DIRECT_PID 2>/dev/null
else
    echo "   ❌ Direkter Start: App ist abgestürzt oder nicht gestartet"
fi

# 6. Launch Services Test
echo ""
echo "6. Launch Services Test:"
echo "   Befehl: open -W -n $APP_PATH"

# Vorher prüfen ob schon Instanzen laufen
if pgrep -f "$APP_NAME" >/dev/null; then
    echo "   🧹 Bestehende App-Instanzen beenden..."
    pkill -f "$APP_NAME"
    sleep 1
fi

echo "   Starte App über Launch Services..."
open -n "$APP_PATH" &
OPEN_PID=$!

# 5 Sekunden warten
sleep 5

# Prüfen ob App läuft
if pgrep -f "$APP_NAME" >/dev/null; then
    APP_PID=$(pgrep -f "$APP_NAME")
    echo "   ✅ Launch Services Start: App läuft (PID: $APP_PID)"

    # Fenster-Status prüfen
    sleep 2
    WINDOW_COUNT=$(osascript -e 'tell application "System Events" to count windows of application process "file-explorer"' 2>/dev/null || echo "0")
    echo "   📱 Sichtbare Fenster: $WINDOW_COUNT"

else
    echo "   ❌ Launch Services Start: App läuft NICHT"
fi

# 7. Console Logs analysieren
echo ""
echo "7. Console Logs (letzte 30 Sekunden):"
echo "   Logs für 'file-explorer':"
LOGS=$(log show --predicate 'process == "file-explorer"' --last 30s --style compact 2>/dev/null)
if [ -n "$LOGS" ]; then
    echo "$LOGS" | head -10
    if [ $(echo "$LOGS" | wc -l) -gt 10 ]; then
        echo "   ... ($(echo "$LOGS" | wc -l | tr -d ' ') Zeilen gesamt)"
    fi
else
    echo "   Keine relevanten Logs gefunden"
fi

echo ""
echo "   System-Logs (Fehler):"
SYSTEM_LOGS=$(log show --predicate 'subsystem == "com.apple.launchservices"' --last 30s --style compact 2>/dev/null | grep -i error)
if [ -n "$SYSTEM_LOGS" ]; then
    echo "$SYSTEM_LOGS" | head -5
else
    echo "   Keine System-Fehler gefunden"
fi

# 8. Cleanup
echo ""
echo "8. Cleanup..."
pkill -f "$APP_NAME" 2>/dev/null
sleep 1

# 9. Empfehlungen
echo ""
echo "🔧 LÖSUNGSVORSCHLÄGE:"
echo "========================"

if ! pgrep -f "$APP_NAME" >/dev/null; then
    echo "📋 Die App startet nicht über Launch Services. Mögliche Lösungen:"
    echo ""
    echo "   A) Vereinfachte main.rs ohne macOS-spezifische Änderungen testen"
    echo "   B) tauri.conf.json weiter vereinfachen"
    echo "   C) App-Bundle neu signieren (falls notwendig)"
    echo "   D) Gatekeeper-Probleme prüfen: spctl --assess '$APP_PATH'"
    echo ""
    echo "▶️  Soll ich eine vereinfachte Version der main.rs erstellen? (j/n)"
fi

echo ""
echo "🏁 Debug-Analyse abgeschlossen"
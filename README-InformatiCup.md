# InformatiCup 2025

Hallo liebes Team. Schön euch hier zu sehen. Im folgenden befindet sich eine Anleitung um den
Explorer laufen zu lassen. Diese ist teilweise gleich, wie im README des Repos, bis auf das, dass
wir hier den dev modus nutzen und auch die tests laufen lassen werden. Falls es zu Problemen kommen
sollte, bitte meldet euch umgehend bei mir per e-mail unter welcher wir unsere Abgabe getätigt
haben. Ich werde mich so schnell wie möglich um die Probleme kümmern. (Es sollte keine geben, jedoch
haben wir alle nötigen dependencies auf unseren Maschinen installiert, welche die Anleitung auch
beinhalten sollte, aber es kann immer zu Schwierigkeiten kommen)

## Komplettlandleitung

### Voraussetzungen

- Cargo (mindestens Version 1.80.0) -> damit auch rust
- Node.js (mindestens Version 20.0.0)
- Tauri CLI (mindestens Version 2.4.0)
- npm (mindestens Version 9.0.0)

### Clonen des Projektes

```bash
git clone https://github.com/CodeMarco05/FileExplorer
cd FileExplorer
```

### Abhängigkeiten installieren

Die Tauri cli kann auch anders installiert werden. Eine Möglichkeit ist eine locale Installation mit
npm. Im Folgenden wird die Installation mit cargo gezeigt, da dies die offizielle ist und auch die
von Tauri empfohlene Variante.

```bash
npm install
cargo install tauri-cli # The version should be >2.4.0 or best ist 2.4.1 with the next comand
cargo install tauri-cli --force --version 2.4.1
```

Es kann sein, dass die umgebung von Ihnen noch weitere Abhängigkeiten benötigt, wie zum Beispiel im
Folgenden gezeigt. Diese sollten aber nur hinzugefügt werden, wenn es zu Problemen kommt. Sonst
einfach bei erstem build weiter machen.

### Linux

```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.0-dev build-essential curl wget file libssl-dev libgtk-3-dev
```

### macOS

```bash
xcode-select --install
brew install coreutils
```

### Windows

Visual Studio build tools.

## Erster Build

Dieser kann je nach Leistung des Systems und Internetverbindung einige Minuten dauern. Wenn der
Build fertig ist, wird dies angezeigt und das Programm sollte sich direkt starten.

```bash
cargo tauri dev
```

Dieser Befehl startet den Tauri-Entwicklungsmodus, welcher auch alle Features des Explorers enthält.

## Build der binary

Der Build der Binary kann einige Minuten dauern. Wenn der Build fertig ist, kann das Programm über
die Commandline gestartet werden. Der Build wird im Ordner `./target/release/` abgelegt mit dem
Namen `src-tauri`. (Wird sich später noch ändern, aber ist noch in aktiver Entwicklung). Dieser
Binary kann ausgeführt werden, um den Explorer zu starten. Diese lässt sich auch den Systembynaries
hinzufügen, sodass sie über die Kommandozeile gestartet werden kann.

```bash
cargo tauri build
```

# Tests

Die Tests können mit dem folgenden Befehl ausgeführt werden. Diese sollten alle erfolgreich
durchlaufen. Falls nicht, bitte umgehend bei uns melden.

```bash
# Zuerst die nötigen Testdaten generieren.
# Dies generiert die Testdaten in ./src-tauri/test-data-for-fuzzy-search
# Es werden 176,840 leere Dateien generiert, welche dann für das indizieren dienen.
# Logs werden in ./src-tauri/logs/ erstellt.
cargo test create_test_data --features "generate-test-data" -- --nocapture

# Dann die Tests ausführen
# Es werden eine selektion für wichtige Tests ausgeführt, welche die Funktionalität des Explorers testen,
# jedoch nicht die Performance explizit testen. Trotzdem werden für alles logs erstellt, sodass auch unter
# ./src-tauri/logs/ error logs erstellt werden, falls es zu welchen kommt. Wichtig ist, dass manche Errors dort
# mit Absicht auftreten, da diese getestet werden.
cargo test

# Um die Performance zu testen, kann der folgende Befehl ausgeführt werden.
# WICHTIG es werden auch default apps geöffnet um dies zu testen, nicht erschrecken lassen.
# Wichtig, es kann sein dass Tests fehlschlagen, einer dieser ist das Generieren der Testdaten.
cargo test --features "full"
```

Die einzelnen Tests können gerne nachvollziehbar eingesehen werden. Diese befinden sich immer in den
entsprechenden Modulen. Entweder können diese durch den Output in der Konsole während des testens
gefunden werden. Sonst auch gene in allen source Dateien. Wichtig ist das zum Beispiel der State von
Tauri während des Startens generiert wird. Diesen initialisieren wir während den Tests selber. Der
Sourcecode ist unter `./src-tauri/src/` zu finden.

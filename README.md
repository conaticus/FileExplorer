<div align="center">

# 🚀 Fast File Explorer

<img src="./src-tauri/assets/images/original.png" alt="Rust Logo" width="100"/>
  
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

**A fast file explorer built with Rust and Tauri**

<img src="assets/screenshot_ThisPC.png" alt="Fast Search Feature" width="1000px"/>

</div>

## ✨ Features

- **🚀 Blazing Fast Search**: Multiple search algorithms with ~15ms response time vs 3min 45sec for Windows Explorer
- **🔍 Advanced Search Engine**: 
  - Fast fuzzy search with ART (Adaptive Radix Tree) implementation
  - LRU caching for optimal performance
  - Multiple search algorithms for different use cases
- **📁 Comprehensive File Operations**: Copy, move, delete, rename with robust error handling
- **🌐 SFTP Support**: Full remote file system operations including browsing, uploading, and downloading
- **🔐 Advanced Permissions**: File and directory permission management
- **📊 File Metadata**: Comprehensive metadata viewing and management
- **🔨 File Hashing**: MD5, SHA2, and CRC32 hash generation for file integrity
- **📄 File Templates**: Template system for creating new files
- **👁️ File Preview**: Built-in preview system for various file types (spotlight-like)
- **💾 Volume Operations**: Drive management and volume operations
- **⚙️ Customizable Settings**: Extensive configuration options
- **🎨 Modern UI**: React-based interface with context menus and responsive design

## 🔍 Current Status

Cross platform compatibility is given and it supports all common Linux distros, macOS, and Windows
which are supported by Tauri. If there is an interest in contributing feel free to join the
[discord channel](https://discord.com/invite/dnVJQtNXjr) from Connaticus or message me or my team.

## 🏗️ Architecture

This is a Tauri-based application with a **Rust backend** and **React frontend**:

### Backend (Rust)
- **Search Engine**: Multiple algorithms with LRU caching
- **File System Operations**: Local and SFTP file operations
- **Command System**: Modular command handlers for different operations
- **Error Handling**: Centralized error management with standardized codes (401-500)
- **Feature Flags**: Extensive Cargo features for different build configurations

### Frontend (React)
- **Provider Pattern**: Hierarchical context providers for state management
- **Modern UI**: Component-based architecture with custom hooks
- **Responsive Design**: Adaptive layouts for different screen sizes

## Coming Soon

- Real-time file watching with caching service
- Search/caching progress indicators
- Enhanced terminal integration

# 🛠️ Installation

Our plan is to provide installers for the supported operating systems or common package installers.
Unfortunately we have serious Problems with Tauri and creating installers. There are some installers
for linux under `dist-builds`. In the future there will be ready to go packages for macOS, Linux and
Windows until then please refer to the compilation from source for your computer.

## Installation from source

### Prerequisites for installing from source

- [NodeJS](https://nodejs.org/) (stable version)
- [Rust](https://www.rust-lang.org/) (stable version)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites/) (version >2.4.0)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) (comes with Rust)
- [Vite](https://vitejs.dev/guide/#scaffolding-your-first-vite-project) (comes with Tauri CLI)

Other required dependencies are installed automatically by the Tauri CLI. If they are not please
contact us.

### Install with compiling from source

Note that on macOS you need XCode installed with the build tools.

```bash
# Install Tauri CLI
cargo install tauri-cli # The version should be >2.4.0 if not already installed

# Build for production
cargo tauri build
```

Go into the build directory and run the created binary. The binary is located in
`FileExplorer/src-tauri/target/release/bundle/`. The name of the binary is `file-explorer` or

### 🐧 Linux

Under Linux the given command generates an `.deb`, `.rpm`, `AppImage` in the
`FileExplorer/src-tauri/target/release/bundle` folder. Select the one which fits your distribution.
Either run the AppImage, Binary or install the `.deb` or `.rpm` package.

#### For the `AppImage`

```bash
#Make sure the image is runnable
chmod +x yourapp-x.y.z.AppImage
#Run the image. After that it should behaving like a native application
./yourapp-x.y.z.AppImage
```

Recommended is to use the binary created in `FileExplorer/src-tauri/target/release/src-tauri`. Give
it executable permissions and then run it from the terminal. You can also put it into your user
binaries folder, e.g. `~/bin`, and add it to your PATH variable.

### 🍎 macOS

```bash
# Install Tauri CLI
cargo install tauri-cli # The version should be >2.4.0 if not already installed

# Build for production
cargo tauri build
```

Tauri creates an `.dmg` or `.app` bundle under the folder
`FileExplorer/src-tauri/target/release/bundle/macos/`. Recommended is to use the binary created in
`FileExplorer/src-tauri/target/release/src-tauri`. Give it executable permissions and then run it
from the terminal. You can also put it into your user binaries folder, e.g. `~/bin`, and add it to
your PATH variable.

### 🪟 Windows

This generates an installer for your system, which lays in
`FileExplorer/src-tauri/target/release/bundle/msi/`. There should be an `.exe` or `.msi` which is
called `file-explorer`. To install it you need to double click the file and install like any other
application. Then you can completely remove the `FileExplorer` folder.

### Development Setup

```bash
# Install Tauri CLI
cargo install tauri-cli # The version should be >2.4.0

# Build for production
cargo tauri build

# Run the development server
cargo tauri dev
```

### Testing and Development Commands

The project uses feature flags for different configurations:

```bash
# Run all tests including long-running ones
cargo test --features full

# Run with benchmark features
cargo test --features benchmarks

# Enable all logging during tests
cargo test --features log-all
```

Available feature combinations:
- `full` - All features including long tests, benchmarks, and file opening
- `log-search` - Enable search progress and error logging
- `log-index` - Enable indexing progress and error logging

## 📸 Images

<div align="center">
<img src="assets/screenshot_details.png" width="700px"/>

<img src="assets/screenshot_overview.png" width="700px"/>

<img src="assets/screenshot_terminal.png" width="700px"/>

<img src="assets/screenshot_settings.png" width="700px"/>
</div>

## 📄 History

The Explorer was started as a project from the youtuber
[Connaticus](https://www.youtube.com/@conaticus). He documented parts of his development journey
online in two Videos:
[I Made a FAST File Explorer](https://youtu.be/Z60f2g-COJY?si=PHWogkV1R_wD8dza) and
[How I RUINED My Rust Project](https://youtu.be/4wdAZQROc4A?si=9ksfN2TcxdDI41BD).

Lots of changes were made in the course of the InformatiCup from the year 2025. It is a competition
in Germany. The given task was to contribute to existing open source projects. The team members were
[Marco Brandt](https://github.com/CodeMarco05), [Daniel Schatz](https://github.com/xd1i0),
[Lauritz Wiebusch](https://github.com/wielauritz), [Sören Panten](https://github.com/SPKonig). The
repo can be found under [FileExplorer](https://github.com/CodeMarco05/FileExplorer).

## ⚡ Performance

This file explorer emphasizes extreme performance with benchmarks showing significant improvements
over native solutions (tested on 170,000 paths):

| Operation   | Fast File Explorer | Windows Explorer |
| ----------- |:------------------:| :--------------: |
| File search |       ~15ms        |   3min 45sec     |

### Technical Implementation
- **Multiple Search Algorithms**: Fast fuzzy search, ART (Adaptive Radix Tree)
- **LRU Caching**: Intelligent caching for search results
- **Rust Backend**: Memory-safe, zero-cost abstractions
- **Modular Architecture**: Command-based system with feature flags

## ⚙️ Configuration

The application uses several configuration files:

- `src-tauri/config/settings.json` - Application settings
- `src-tauri/config/meta_data.json` - Metadata configuration  
- `src-tauri/tauri.conf.json` - Tauri application configuration
- `package.json` - Frontend dependencies and scripts

## 🤝 Contributing

Contributions are welcome! Before contributing, please read our
[contributing guidelines](CONTRIBUTING.md).

## 📝 License

This project is licensed under the GNU General Public License v3.0 – see the LICENSE file for
details.

## 📬 Contact

Have questions or feedback? Open an issue on our GitHub repository!

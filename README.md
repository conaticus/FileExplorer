# Fast File Explorer
This is a fast file explorer written in Rust. After testing on my C drive, this file explorer was able to find a file in 280ms. In comparison, Windows took 3 minutes and 45 seconds.

Bare in mind this was just a proof of concept and this is **not complete**, sadly I did not have time to implement these features for the video:
- Up to date cache with file watching
- Ability to search specific directories instead of just a cached disk
- Run on startup
- Top navigation bar
- Search/caching progress counter
- Ability to search for file extensions without including any name
- Ability to open files

![Fast Search Feature](./screenshots/search.jpg)

# Dev Setup/Installation
## Prerequisites
- Stable [NodeJS](https://nodejs.org/) Install
- Stable [Rust](https://www.rust-lang.org/) Install
- Correct packages installed for Tauri to work (For Linux users)

## Steps
```
#  Make sure you have Tauri CLI installed
cargo install tauri-cli

# Install dependencies
npm i

# Run app for development
cargo tauri dev

# Build for production
cargo tauri build
```

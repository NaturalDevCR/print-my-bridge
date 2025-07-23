#!/bin/bash

echo "🚀 Compilando Print My Bridge para todas las plataformas..."

# macOS
echo "📱 Compilando para macOS..."
cargo tauri build --target universal-apple-darwin

# Windows
echo "🪟 Compilando para Windows..."
rustup target add x86_64-pc-windows-msvc
cargo tauri build --target x86_64-pc-windows-msvc

# Linux
echo "🐧 Compilando para Linux..."
rustup target add x86_64-unknown-linux-gnu
cargo tauri build --target x86_64-unknown-linux-gnu

echo "✅ Compilación completada!"
echo "📦 Los ejecutables están en src-tauri/target/[target]/release/bundle/"
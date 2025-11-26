#!/bin/bash
# Скрипт для сборки Windows EXE файла

set -e

echo "🔨 Сборка Adaptive Entity Engine для Windows..."

# Проверка наличия cargo
if ! command -v cargo &> /dev/null; then
    echo "❌ Ошибка: cargo не найден. Установите Rust: https://rustup.rs/"
    exit 1
fi

# Проверка наличия target для Windows
if ! rustup target list --installed | grep -q "x86_64-pc-windows-msvc"; then
    echo "📦 Установка target для Windows..."
    rustup target add x86_64-pc-windows-msvc
fi

# Сборка release версии
echo "⚙️ Компиляция release версии..."
cargo build --release --target x86_64-pc-windows-msvc --features gui

# Проверка результата
EXE_PATH="target/x86_64-pc-windows-msvc/release/adaptive-entity-engine.exe"

if [ -f "$EXE_PATH" ]; then
    SIZE=$(du -h "$EXE_PATH" | cut -f1)
    echo "✅ Сборка успешна!"
    echo "📦 Файл: $EXE_PATH"
    echo "📏 Размер: $SIZE"
    
    # Создание директории для дистрибутива
    DIST_DIR="dist/windows"
    mkdir -p "$DIST_DIR"
    
    # Копирование EXE
    cp "$EXE_PATH" "$DIST_DIR/"
    
    echo "📁 EXE файл скопирован в: $DIST_DIR/"
    echo ""
    echo "🎉 Готово! Вы можете распространять файл:"
    echo "   $DIST_DIR/adaptive-entity-engine.exe"
else
    echo "❌ Ошибка: EXE файл не найден"
    exit 1
fi

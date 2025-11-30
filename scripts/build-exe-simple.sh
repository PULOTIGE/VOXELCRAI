#!/bin/bash
# Упрощенная сборка - попытка собрать без проблемных зависимостей

set -e

echo "🔨 Попытка упрощенной сборки EXE..."

# Пробуем собрать только базовую версию без интеграций
echo "Сборка без интеграций..."
cargo build --release --target x86_64-pc-windows-gnu --no-default-features --features "gui" --bin adaptive-entity-engine 2>&1 | tee /tmp/simple_build.log

if [ -f "target/x86_64-pc-windows-gnu/release/adaptive-entity-engine.exe" ]; then
    echo "✅ Успех! Копирую EXE..."
    mkdir -p dist/windows
    cp target/x86_64-pc-windows-gnu/release/adaptive-entity-engine.exe dist/windows/
    echo "✅ EXE скопирован в dist/windows/"
    ls -lh dist/windows/adaptive-entity-engine.exe
else
    echo "❌ Сборка не удалась. Проверьте /tmp/simple_build.log"
    exit 1
fi

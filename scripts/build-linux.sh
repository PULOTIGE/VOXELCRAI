#!/bin/bash
# Скрипт для сборки Linux исполняемого файла

set -e

echo "🔨 Сборка Adaptive Entity Engine для Linux..."

# Проверка наличия cargo
if ! command -v cargo &> /dev/null; then
    echo "❌ Ошибка: cargo не найден. Установите Rust: https://rustup.rs/"
    exit 1
fi

# Сборка release версии
echo "⚙️ Компиляция release версии..."
cargo build --release --features gui

# Проверка результата
BIN_PATH="target/release/adaptive-entity-engine"

if [ -f "$BIN_PATH" ]; then
    SIZE=$(du -h "$BIN_PATH" | cut -f1)
    echo "✅ Сборка успешна!"
    echo "📦 Файл: $BIN_PATH"
    echo "📏 Размер: $SIZE"
    
    # Создание директории для дистрибутива
    DIST_DIR="dist/linux"
    mkdir -p "$DIST_DIR"
    
    # Копирование бинарника
    cp "$BIN_PATH" "$DIST_DIR/"
    
    # Создание скрипта запуска
    cat > "$DIST_DIR/run.sh" << 'EOF'
#!/bin/bash
cd "$(dirname "$0")"
./adaptive-entity-engine "$@"
EOF
    chmod +x "$DIST_DIR/run.sh"
    
    echo "📁 Исполняемый файл скопирован в: $DIST_DIR/"
    echo ""
    echo "🎉 Готово! Вы можете распространять файл:"
    echo "   $DIST_DIR/adaptive-entity-engine"
    echo "   $DIST_DIR/run.sh"
else
    echo "❌ Ошибка: Исполняемый файл не найден"
    exit 1
fi

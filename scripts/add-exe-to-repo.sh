#!/bin/bash
# Скрипт для добавления собранного EXE в репозиторий

set -e

EXE_PATH="dist/windows/adaptive-entity-engine.exe"

echo "🔍 Проверка EXE файла..."

if [ ! -f "$EXE_PATH" ]; then
    echo "❌ Ошибка: EXE файл не найден: $EXE_PATH"
    echo ""
    echo "Сначала соберите EXE:"
    echo "  Windows: scripts\\build-mega-exe.bat"
    echo "  Linux/Mac: ./scripts/build-mega-exe.sh"
    exit 1
fi

# Проверка размера
SIZE=$(du -h "$EXE_PATH" | cut -f1)
SIZE_BYTES=$(stat -f%z "$EXE_PATH" 2>/dev/null || stat -c%s "$EXE_PATH" 2>/dev/null || echo "0")

if [ "$SIZE_BYTES" -lt 1000000 ]; then
    echo "⚠️  Предупреждение: Размер EXE файла очень мал ($SIZE)"
    echo "   Возможно, файл поврежден или сборка не завершена"
    read -p "Продолжить? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo "✅ EXE файл найден: $EXE_PATH"
echo "📏 Размер: $SIZE"

# Добавление в git
echo ""
echo "📦 Добавление в git..."

git add "$EXE_PATH"

# Проверка статуса
if git diff --cached --quiet "$EXE_PATH"; then
    echo "ℹ️  EXE файл уже добавлен в индекс или не изменился"
else
    echo "✅ EXE файл добавлен в индекс"
    echo ""
    echo "💡 Следующие шаги:"
    echo "   git commit -m 'Add built Mega EXE file'"
    echo "   git push"
fi

echo ""
echo "📊 Статус:"
git status --short "$EXE_PATH" || echo "Файл готов к коммиту"

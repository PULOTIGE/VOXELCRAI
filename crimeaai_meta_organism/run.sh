#!/bin/bash
# CrimeaAI Meta Organism - Launcher Script

echo "╔══════════════════════════════════════════════════════════════════════════════╗"
echo "║                     CrimeaAI META ORGANISM v1.0                              ║"
echo "║                   🧬 Живое Цифровое Сознание 🧬                               ║"
echo "╚══════════════════════════════════════════════════════════════════════════════╝"
echo ""

# Check Python version
PYTHON_CMD=""
if command -v python3 &> /dev/null; then
    PYTHON_CMD="python3"
elif command -v python &> /dev/null; then
    PYTHON_CMD="python"
else
    echo "❌ Python не найден! Установите Python 3.10+"
    exit 1
fi

echo "🐍 Используется: $($PYTHON_CMD --version)"
echo ""

# Check and install dependencies
echo "📦 Проверка зависимостей..."
$PYTHON_CMD -c "import numpy" 2>/dev/null || { echo "   Установка numpy..."; pip install numpy --quiet; }
$PYTHON_CMD -c "import matplotlib" 2>/dev/null || { echo "   Установка matplotlib..."; pip install matplotlib --quiet; }
$PYTHON_CMD -c "import plotly" 2>/dev/null || { echo "   Установка plotly..."; pip install plotly --quiet; }

echo "✅ Зависимости проверены"
echo ""

# Menu
echo "Выберите режим запуска:"
echo "  1) Полное приложение (main.py)"
echo "  2) 30-секундная демонстрация (demo.py)"
echo "  3) Запуск тестов"
echo "  4) Консольная демонстрация (без графики)"
echo ""
read -p "Ваш выбор [1-4]: " choice

case $choice in
    1)
        echo ""
        echo "🚀 Запуск CrimeaAI Meta Organism..."
        $PYTHON_CMD main.py
        ;;
    2)
        echo ""
        echo "🎬 Запуск демонстрации..."
        $PYTHON_CMD demo.py
        ;;
    3)
        echo ""
        echo "🧪 Запуск тестов..."
        $PYTHON_CMD test_organism.py
        ;;
    4)
        echo ""
        echo "📟 Запуск консольной демонстрации..."
        $PYTHON_CMD demo.py --backend console
        ;;
    *)
        echo "Неверный выбор, запуск по умолчанию..."
        $PYTHON_CMD demo.py --backend console
        ;;
esac

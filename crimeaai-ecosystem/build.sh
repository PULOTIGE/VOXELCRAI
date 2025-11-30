#!/bin/bash
# CrimeaAI Build Script for Linux/Mac
# ===================================

echo ""
echo "╔═══════════════════════════════════════╗"
echo "║  CrimeaAI EXE Builder                 ║"
echo "╚═══════════════════════════════════════╝"
echo ""

# Check Python
if ! command -v python3 &> /dev/null; then
    echo "❌ Python не найден! Установите Python 3.10+"
    exit 1
fi

# Install dependencies
echo "📦 Установка зависимостей..."
pip3 install -r requirements.txt

# Build
echo ""
echo "🔨 Сборка..."
python3 build_exe.py --onefile

echo ""
echo "✅ Готово! Файл: dist/CrimeaAI"

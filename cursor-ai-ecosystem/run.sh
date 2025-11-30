#!/bin/bash
# Скрипт запуска Cursor AI Ecosystem для Linux/Mac

echo "================================"
echo "  Cursor AI Ecosystem - 19V"
echo "================================"
echo ""

# Проверка Python
if ! command -v python3 &> /dev/null; then
    echo "Ошибка: Python 3 не найден!"
    echo "Установите Python 3.8 или выше"
    exit 1
fi

# Проверка виртуального окружения
if [ ! -d "venv" ]; then
    echo "Виртуальное окружение не найдено. Создание..."
    python3 -m venv venv
    echo "✓ Виртуальное окружение создано"
fi

# Активация
echo "Активация виртуального окружения..."
source venv/bin/activate

# Проверка зависимостей
if [ ! -f "venv/installed" ]; then
    echo "Установка зависимостей..."
    pip install -r requirements.txt
    touch venv/installed
    echo "✓ Зависимости установлены"
fi

# Запуск
echo ""
echo "Запуск экосистемы..."
python main.py "$@"

# Деактивация
deactivate

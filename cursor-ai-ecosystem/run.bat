@echo off
REM Скрипт запуска Cursor AI Ecosystem для Windows

echo ================================
echo   Cursor AI Ecosystem - 19V
echo ================================
echo.

REM Проверка Python
python --version >nul 2>&1
if errorlevel 1 (
    echo Ошибка: Python не найден!
    echo Установите Python 3.8 или выше
    pause
    exit /b 1
)

REM Проверка виртуального окружения
if not exist "venv" (
    echo Виртуальное окружение не найдено. Создание...
    python -m venv venv
    echo ✓ Виртуальное окружение создано
)

REM Активация
echo Активация виртуального окружения...
call venv\Scripts\activate.bat

REM Проверка зависимостей
if not exist "venv\installed" (
    echo Установка зависимостей...
    pip install -r requirements.txt
    echo. > venv\installed
    echo ✓ Зависимости установлены
)

REM Запуск
echo.
echo Запуск экосистемы...
python main.py %*

REM Деактивация
call venv\Scripts\deactivate.bat

pause

@echo off
REM CrimeaAI Build Script for Windows
REM ==================================

echo.
echo ╔═══════════════════════════════════════╗
echo ║  CrimeaAI EXE Builder                 ║
echo ╚═══════════════════════════════════════╝
echo.

REM Check Python
python --version >nul 2>&1
if errorlevel 1 (
    echo ❌ Python не найден! Установите Python 3.10+
    pause
    exit /b 1
)

REM Install dependencies
echo 📦 Установка зависимостей...
pip install -r requirements.txt

REM Build EXE
echo.
echo 🔨 Сборка EXE...
python build_exe.py --onefile

echo.
echo ✅ Готово! Файл: dist\CrimeaAI.exe
pause

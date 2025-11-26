@echo off
REM Скрипт для сборки МЕГА EXE файла на Windows

echo.
echo ========================================
echo   Сборка МЕГА EXE файла
echo   Adaptive Entity Engine v1.0
echo ========================================
echo.

REM Проверка наличия cargo
where cargo >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo [ОШИБКА] cargo не найден. Установите Rust: https://rustup.rs/
    pause
    exit /b 1
)

REM Проверка наличия target для Windows
rustup target list --installed | findstr "x86_64-pc-windows-msvc" >nul
if %ERRORLEVEL% NEQ 0 (
    echo [ИНФО] Установка target для Windows...
    rustup target add x86_64-pc-windows-msvc
)

REM Очистка предыдущих сборок
echo [ИНФО] Очистка предыдущих сборок...
cargo clean --target x86_64-pc-windows-msvc

REM Установка переменных окружения для статической линковки
set RUSTFLAGS=-C target-feature=+crt-static
set CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS=-C target-feature=+crt-static

REM Сборка release версии
echo.
echo [ИНФО] Компиляция release версии...
echo        Это может занять несколько минут...
echo        - Оптимизация размера включена
echo        - LTO (Link Time Optimization) включена
echo        - Статическая линковка включена
echo.

cargo build --release --target x86_64-pc-windows-msvc --features "gui,all-integrations" --bin adaptive-entity-engine

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ОШИБКА] Сборка не удалась!
    pause
    exit /b 1
)

REM Проверка результата
set EXE_PATH=target\x86_64-pc-windows-msvc\release\adaptive-entity-engine.exe

if exist "%EXE_PATH%" (
    echo.
    echo ========================================
    echo   Сборка успешна!
    echo ========================================
    echo.
    echo [ИНФО] Файл: %EXE_PATH%
    
    REM Создание директории для дистрибутива
    if not exist "dist\windows" mkdir "dist\windows"
    
    REM Копирование EXE
    copy /Y "%EXE_PATH%" "dist\windows\adaptive-entity-engine.exe" >nul
    
    REM Создание README
    (
        echo ===============================================================
        echo   Adaptive Entity Engine v1.0 - МЕГА EXE версия
        echo ===============================================================
        echo.
        echo Это standalone исполняемый файл, который содержит все необходимые
        echo зависимости и библиотеки. Не требует установки дополнительных
        echo компонентов.
        echo.
        echo ЗАПУСК:
        echo   Просто дважды кликните на adaptive-entity-engine.exe
        echo.
        echo ТРЕБОВАНИЯ:
        echo   - Windows 10 или новее
        echo   - Видеокарта с поддержкой Vulkan или OpenGL
        echo   - 2 ГБ свободной оперативной памяти
        echo.
        echo ФУНКЦИИ:
        echo   [OK] Чат-интерфейс в белых тонах
        echo   [OK] Режим обучения с загрузкой файлов
        echo   [OK] Интеграции с AI-сервисами
        echo   [OK] Воксельный движок с визуализацией
        echo   [OK] Все зависимости встроены
        echo.
        echo ===============================================================
    ) > "dist\windows\README.txt"
    
    echo.
    echo [УСПЕХ] МЕГА EXE файл создан:
    echo        dist\windows\adaptive-entity-engine.exe
    echo.
    echo [ИНФО] Этот файл содержит все зависимости и готов к распространению!
    echo.
    
) else (
    echo.
    echo [ОШИБКА] EXE файл не найден!
    pause
    exit /b 1
)

pause

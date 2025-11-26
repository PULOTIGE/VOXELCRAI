# Инструкции по сборке МЕГА EXE файла

## Что такое МЕГА EXE?

МЕГА EXE - это один исполняемый файл, который содержит:
- ✅ Все зависимости и библиотеки
- ✅ Встроенные ресурсы (шейдеры, иконки)
- ✅ Статическую линковку для максимальной портативности
- ✅ Оптимизацию размера с LTO (Link Time Optimization)

**Результат**: Один файл, который работает на любой Windows машине без установки дополнительных компонентов!

## Требования для сборки

1. **Rust** (установите с https://rustup.rs/)
2. **Windows SDK** (для сборки на Windows)
3. **Visual Studio Build Tools** (для Windows, если используете MSVC)

## Сборка

### На Linux/Mac (кросскомпиляция)

```bash
# Установите target для Windows
rustup target add x86_64-pc-windows-msvc

# Запустите скрипт сборки
./scripts/build-mega-exe.sh
```

### На Windows

```cmd
# Просто запустите bat файл
scripts\build-mega-exe.bat
```

Или вручную:

```cmd
cargo build --release --target x86_64-pc-windows-msvc --features "gui,all-integrations" --bin adaptive-entity-engine
```

## Что делает скрипт сборки?

1. **Проверяет зависимости** - убеждается, что Rust и target установлены
2. **Очищает предыдущие сборки** - для чистого билда
3. **Настраивает статическую линковку** - через переменные окружения
4. **Компилирует с оптимизацией**:
   - `opt-level = "z"` - оптимизация размера
   - `lto = "fat"` - полная Link Time Optimization
   - `codegen-units = 1` - единая единица компиляции для лучшей оптимизации
5. **Копирует результат** в `dist/windows/`

## Оптимизации размера

### Включенные оптимизации:

- ✅ **LTO (Link Time Optimization)** - максимальная оптимизация на этапе линковки
- ✅ **Оптимизация размера** (`opt-level = "z`) - минимизация размера бинарника
- ✅ **Strip символов** - удаление отладочной информации
- ✅ **Статическая линковка CRT** - встраивание C runtime библиотек
- ✅ **Panic = abort** - уменьшение размера за счет удаления unwinding кода

### Результат:

- **Размер**: 80-150 МБ (зависит от включенных функций)
- **Зависимости**: Минимальные (только системные DLL Windows)
- **Портативность**: Работает на любой Windows 10+ машине

## Устранение проблем

### Ошибка: "linker not found"

**Решение**: Установите Visual Studio Build Tools или используйте GNU toolchain:
```bash
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
```

### Ошибка: "target not found"

**Решение**: Установите target:
```bash
rustup target add x86_64-pc-windows-msvc
```

### Большой размер EXE (>200 МБ)

**Причины**:
- Отладочная информация не удалена
- LTO не включена
- Включены ненужные зависимости

**Решение**: Проверьте настройки в `Cargo.toml`:
```toml
[profile.release]
opt-level = "z"
lto = "fat"
strip = true
```

### EXE не запускается на другой машине

**Причины**:
- Отсутствуют системные DLL
- Несовместимая версия Windows

**Решение**: 
- Убедитесь, что используется статическая линковка CRT
- Проверьте минимальные требования (Windows 10+)
- Установите Visual C++ Redistributable (если не используется статическая линковка)

## Проверка зависимостей

После сборки можно проверить зависимости:

**Windows (PowerShell):**
```powershell
dumpbin /dependents dist\windows\adaptive-entity-engine.exe
```

**Linux (objdump):**
```bash
objdump -p dist/windows/adaptive-entity-engine.exe | grep "DLL Name"
```

Идеально, если видны только системные DLL Windows (kernel32.dll, user32.dll и т.д.)

## Распространение

МЕГА EXE файл готов к распространению:

1. Скопируйте `dist/windows/adaptive-entity-engine.exe`
2. При необходимости добавьте `README.txt` (создается автоматически)
3. Распространяйте как есть - никаких дополнительных файлов не требуется!

## Дополнительные опции

### Сборка без интеграций (меньший размер)

```bash
cargo build --release --target x86_64-pc-windows-msvc --features gui --bin adaptive-entity-engine
```

### Сборка с отладочной информацией

Измените в `Cargo.toml`:
```toml
[profile.release]
strip = false
debug = true
```

### Сборка для других платформ

**Linux:**
```bash
cargo build --release --features "gui,all-integrations"
```

**macOS:**
```bash
cargo build --release --target x86_64-apple-darwin --features "gui,all-integrations"
```

## Производительность сборки

- **Время сборки**: 5-15 минут (зависит от CPU)
- **Память**: Требуется 4-8 ГБ RAM для LTO
- **Диск**: ~2-5 ГБ свободного места

Совет: Используйте SSD для ускорения сборки!

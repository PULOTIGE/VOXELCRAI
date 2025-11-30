# Руководство по установке Cursor AI Ecosystem

## Системные требования

### Минимальные
- **ОС**: Windows 7+, Linux (Ubuntu 18.04+), macOS 10.14+
- **Python**: 3.8 или выше
- **RAM**: 2 ГБ
- **CPU**: 2 ядра
- **Место на диске**: 500 МБ

### Рекомендуемые
- **ОС**: Windows 10/11, Linux (Ubuntu 20.04+), macOS 11+
- **Python**: 3.10 или выше
- **RAM**: 8 ГБ
- **CPU**: 4+ ядер
- **Место на диске**: 2 ГБ

## Установка Python

### Windows

1. Скачайте Python с [python.org](https://www.python.org/downloads/)
2. Запустите установщик
3. **ВАЖНО**: Отметьте "Add Python to PATH"
4. Нажмите "Install Now"

Проверка:
```cmd
python --version
```

### Linux (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install python3 python3-pip python3-venv
```

Проверка:
```bash
python3 --version
pip3 --version
```

### macOS

```bash
# Через Homebrew
brew install python3

# Или скачайте с python.org
```

Проверка:
```bash
python3 --version
```

## Установка Cursor AI Ecosystem

### Вариант 1: Из исходников

```bash
# 1. Клонирование (если есть git)
git clone <repository-url>
cd cursor-ai-ecosystem

# 2. Создание виртуального окружения
python -m venv venv

# 3. Активация виртуального окружения
# Windows:
venv\Scripts\activate
# Linux/Mac:
source venv/bin/activate

# 4. Установка зависимостей
pip install -r requirements.txt

# 5. Запуск
python main.py
```

### Вариант 2: Автоматическая установка

#### Windows
Просто запустите `run.bat` - он автоматически:
- Создаст виртуальное окружение
- Установит зависимости
- Запустит программу

#### Linux/Mac
```bash
chmod +x run.sh
./run.sh
```

### Вариант 3: EXE файл (только Windows)

1. Соберите EXE:
```bash
python scripts/build_exe.py
```

2. Запустите:
```bash
dist\CursorAI.exe
```

## Проверка установки

### Тест 1: Импорты

```bash
python -c "import numpy; import pygame; import scipy; print('OK')"
```

Должно вывести: `OK`

### Тест 2: Примеры

```bash
python examples/simple_usage.py
```

Должны увидеть вывод с нуклеотидами, вокселями и паттернами.

### Тест 3: Основная программа

```bash
python main.py --voxels 100 --nucleotides 1000
```

Должно открыться окно с визуализацией.

## Решение проблем

### Ошибка: "python не найден"

**Windows:**
- Переустановите Python с галочкой "Add to PATH"
- Или используйте `py` вместо `python`

**Linux:**
- Используйте `python3` вместо `python`
- Установите: `sudo apt install python3`

### Ошибка: "pip не найден"

**Windows:**
```cmd
python -m ensurepip --upgrade
```

**Linux:**
```bash
sudo apt install python3-pip
```

### Ошибка: "ModuleNotFoundError: No module named 'numpy'"

Установите зависимости:
```bash
pip install -r requirements.txt
```

### Ошибка: "pygame.error: video system not initialized"

**Linux:**
Установите SDL2:
```bash
sudo apt install libsdl2-dev libsdl2-image-dev libsdl2-mixer-dev libsdl2-ttf-dev
```

### Ошибка памяти при большом количестве нуклеотидов

Уменьшите количество:
```bash
python main.py --nucleotides 10000 --voxels 100
```

### Низкий FPS

Отключите визуализацию:
```bash
python main.py --no-visualization
```

Или уменьшите компоненты:
```bash
python main.py --nucleotides 50000 --voxels 200
```

## Обновление

### Обновление зависимостей

```bash
pip install --upgrade -r requirements.txt
```

### Обновление из Git

```bash
git pull
pip install -r requirements.txt
```

## Удаление

### Удаление виртуального окружения

```bash
# Windows
rmdir /s venv

# Linux/Mac
rm -rf venv
```

### Полное удаление

Просто удалите папку `cursor-ai-ecosystem`

## Дополнительные зависимости (опционально)

### Для разработки

```bash
pip install black flake8 mypy pytest
```

### Для ускорения

```bash
# Numba для JIT компиляции
pip install numba

# CuPy для GPU (если есть NVIDIA)
pip install cupy
```

## Настройка IDE

### VS Code

1. Установите расширение Python
2. Выберите интерпретатор: `Ctrl+Shift+P` → "Python: Select Interpreter"
3. Выберите `./venv/bin/python`

### PyCharm

1. File → Settings → Project → Python Interpreter
2. Add Interpreter → Existing Environment
3. Выберите `venv/bin/python`

## Проверка всех компонентов

```bash
# Через Makefile (Linux/Mac)
make install
make examples
make run

# Через Python
python -c "from src.core import Nucleotide, Voxel; print('Core OK')"
python -c "from src.utils import ConceptSearcher; print('Utils OK')"
python -c "from src.visualization import EcosystemDisplay; print('Viz OK')"
```

## Следующие шаги

1. Прочитайте [QUICKSTART_RU.md](QUICKSTART_RU.md)
2. Изучите [ARCHITECTURE.md](ARCHITECTURE.md)
3. Попробуйте примеры в `examples/`
4. Настройте `config.example.py` → `config.py`

## Поддержка

При возникновении проблем:
1. Проверьте версию Python: `python --version` (должна быть >= 3.8)
2. Убедитесь, что все зависимости установлены: `pip list`
3. Попробуйте запустить в легком режиме: `python main.py --nucleotides 1000 --voxels 10`

---

**Успешной установки! 🚀**

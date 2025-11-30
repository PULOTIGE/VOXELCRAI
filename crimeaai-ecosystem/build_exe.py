#!/usr/bin/env python3
"""
Build Script - Компиляция в EXE
===============================

Скрипт для создания исполняемого файла с помощью PyInstaller.

Использование:
    python build_exe.py           # Создать EXE
    python build_exe.py --onefile # Создать один файл
    python build_exe.py --debug   # С отладочной информацией
"""

import os
import sys
import shutil
import subprocess
import argparse
from pathlib import Path


def check_pyinstaller():
    """Проверка наличия PyInstaller"""
    try:
        import PyInstaller
        return True
    except ImportError:
        print("❌ PyInstaller не установлен!")
        print("   Установите: pip install pyinstaller")
        return False


def clean_build():
    """Очистка предыдущей сборки"""
    dirs_to_clean = ['build', 'dist', '__pycache__']
    
    for dir_name in dirs_to_clean:
        if os.path.exists(dir_name):
            shutil.rmtree(dir_name)
            print(f"🗑️ Удалена директория: {dir_name}")
    
    # Удаляем .spec файлы
    for f in Path('.').glob('*.spec'):
        f.unlink()
        print(f"🗑️ Удалён файл: {f}")


def create_spec_file(onefile: bool = True, debug: bool = False) -> str:
    """
    Создание .spec файла для PyInstaller
    
    Args:
        onefile: создавать один файл
        debug: включить отладку
    
    Returns:
        Путь к .spec файлу
    """
    spec_content = f'''# -*- mode: python ; coding: utf-8 -*-

import sys
from PyInstaller.utils.hooks import collect_all

# Собираем все данные pygame
pygame_datas, pygame_binaries, pygame_hiddenimports = collect_all('pygame')

block_cipher = None

a = Analysis(
    ['main.py'],
    pathex=['{os.getcwd()}'],
    binaries=pygame_binaries,
    datas=[
        ('assets', 'assets'),
        ('plugins', 'plugins'),
    ] + pygame_datas,
    hiddenimports=[
        'pygame',
        'pygame.locals',
        'numpy',
        'requests',
        'bs4',
        'msgpack',
        'cloudpickle',
    ] + pygame_hiddenimports,
    hookspath=[],
    hooksconfig={{}},
    runtime_hooks=[],
    excludes=[
        'tkinter',
        'matplotlib',
        'scipy',
        'torch',  # Исключаем тяжёлые библиотеки
    ],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive={'not ' if not debug else ''}False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

{'exe = EXE(' if onefile else 'exe = EXE('}
    pyz,
    a.scripts,
    {'a.binaries,' if onefile else ''}
    {'a.zipfiles,' if onefile else ''}
    {'a.datas,' if onefile else ''}
    [],
    exclude_binaries={'not ' if onefile else ''}True,
    name='CrimeaAI',
    debug={debug},
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    runtime_tmpdir=None,
    console={'True' if debug else 'False'},
    disable_windowed_traceback=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon='assets/icon.ico' if os.path.exists('assets/icon.ico') else None,
)

{'coll = COLLECT(' if not onefile else ''}
{'    exe,' if not onefile else ''}
{'    a.binaries,' if not onefile else ''}
{'    a.zipfiles,' if not onefile else ''}
{'    a.datas,' if not onefile else ''}
{"    strip=False," if not onefile else ''}
{"    upx=True," if not onefile else ''}
{"    upx_exclude=[]," if not onefile else ''}
{"    name='CrimeaAI'," if not onefile else ''}
{')' if not onefile else ''}
'''
    
    spec_path = 'CrimeaAI.spec'
    with open(spec_path, 'w') as f:
        f.write(spec_content)
    
    print(f"📝 Создан {spec_path}")
    return spec_path


def create_assets():
    """Создание директории assets с необходимыми файлами"""
    os.makedirs('assets', exist_ok=True)
    
    # Создаём placeholder для иконки
    icon_placeholder = 'assets/icon.ico.placeholder'
    if not os.path.exists('assets/icon.ico') and not os.path.exists(icon_placeholder):
        with open(icon_placeholder, 'w') as f:
            f.write("Place your icon.ico file here")
        print("📁 Создана директория assets/")


def build_exe(spec_file: str, debug: bool = False):
    """
    Сборка EXE файла
    
    Args:
        spec_file: путь к .spec файлу
        debug: режим отладки
    """
    print("\n🔨 Начинаем сборку EXE...")
    print("   Это может занять несколько минут...\n")
    
    cmd = [
        sys.executable, '-m', 'PyInstaller',
        spec_file,
        '--clean',
        '--noconfirm',
    ]
    
    if debug:
        cmd.append('--log-level=DEBUG')
    
    try:
        result = subprocess.run(cmd, check=True)
        
        if result.returncode == 0:
            print("\n✅ Сборка успешно завершена!")
            print("   Исполняемый файл: dist/CrimeaAI.exe")
            return True
    
    except subprocess.CalledProcessError as e:
        print(f"\n❌ Ошибка сборки: {e}")
        return False
    
    except FileNotFoundError:
        print("\n❌ PyInstaller не найден!")
        return False


def create_readme():
    """Создание README для дистрибутива"""
    readme_content = """# CrimeaAI Ecosystem

## 🧠 AI-экосистема с биологическими структурами данных

### Запуск
1. Запустите `CrimeaAI.exe`
2. Или используйте командную строку:
   ```
   CrimeaAI.exe --help
   ```

### Управление
- **Space**: Пауза/Продолжить
- **+/-**: Увеличить/Уменьшить масштаб
- **ESC**: Выход

### Функции
- 🧬 Пул нуклеотидов (256 байт на ячейку)
- 🌍 Мир вокселей (9 КБ микро-организмы)
- 💡 База паттернов освещения
- ⚡ Движок кайфа (производная энтропии)
- 🔍 Поиск концептов (DuckDuckGo)

### Структура
```
CrimeaAI/
├── CrimeaAI.exe      # Исполняемый файл
├── data/             # Данные и сохранения
├── plugins/          # Плагины
└── README.txt        # Этот файл
```

### Автор
CrimeaAI Team

### Лицензия
MIT License
"""
    
    readme_path = 'dist/README.txt'
    os.makedirs('dist', exist_ok=True)
    
    with open(readme_path, 'w', encoding='utf-8') as f:
        f.write(readme_content)
    
    print(f"📝 Создан {readme_path}")


def main():
    parser = argparse.ArgumentParser(description='Build CrimeaAI EXE')
    parser.add_argument('--onefile', action='store_true', help='Create single file')
    parser.add_argument('--debug', action='store_true', help='Enable debug mode')
    parser.add_argument('--clean', action='store_true', help='Clean build directories only')
    parser.add_argument('--no-clean', action='store_true', help='Skip cleaning')
    
    args = parser.parse_args()
    
    print("""
    ╔═══════════════════════════════════════╗
    ║  CrimeaAI EXE Builder                 ║
    ╚═══════════════════════════════════════╝
    """)
    
    # Проверяем PyInstaller
    if not check_pyinstaller():
        sys.exit(1)
    
    # Только очистка
    if args.clean:
        clean_build()
        print("\n✅ Очистка завершена!")
        return
    
    # Очищаем если нужно
    if not args.no_clean:
        clean_build()
    
    # Создаём assets
    create_assets()
    
    # Создаём .spec файл
    spec_file = create_spec_file(
        onefile=args.onefile,
        debug=args.debug
    )
    
    # Собираем
    if build_exe(spec_file, args.debug):
        # Создаём README
        create_readme()
        
        # Копируем необходимые файлы
        os.makedirs('dist/data', exist_ok=True)
        os.makedirs('dist/plugins', exist_ok=True)
        
        print("""
    ╔═══════════════════════════════════════╗
    ║  ✅ Сборка завершена!                 ║
    ║                                       ║
    ║  Файл: dist/CrimeaAI.exe             ║
    ╚═══════════════════════════════════════╝
        """)
    else:
        sys.exit(1)


if __name__ == "__main__":
    main()

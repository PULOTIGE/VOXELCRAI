#!/usr/bin/env python3
"""
Скрипт для сборки проекта в EXE файл с помощью PyInstaller
"""

import os
import sys
import subprocess
import shutil


def build_executable():
    """Сборка исполняемого файла."""
    print("=" * 60)
    print("  СБОРКА CURSOR AI ECOSYSTEM В EXE")
    print("=" * 60)
    
    # Проверка наличия PyInstaller
    try:
        import PyInstaller
        print(f"✓ PyInstaller найден: {PyInstaller.__version__}")
    except ImportError:
        print("✗ PyInstaller не найден!")
        print("  Установите: pip install pyinstaller")
        return False
    
    # Путь к главному файлу
    main_script = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'main.py')
    
    if not os.path.exists(main_script):
        print(f"✗ Главный файл не найден: {main_script}")
        return False
    
    print(f"✓ Главный файл: {main_script}")
    
    # Параметры сборки
    build_args = [
        'pyinstaller',
        '--onefile',  # один файл
        '--name', 'CursorAI',  # имя выходного файла
        '--clean',  # очистка кеша
        '--windowed' if sys.platform == 'win32' else '--console',  # режим окна
        '--icon', 'NONE',  # можно добавить иконку
    ]
    
    # Добавление данных и библиотек
    # build_args.extend(['--add-data', 'src:src'])
    
    # Скрытие импортов
    build_args.extend([
        '--hidden-import', 'numpy',
        '--hidden-import', 'scipy',
        '--hidden-import', 'pygame',
        '--hidden-import', 'requests',
        '--hidden-import', 'bs4',
    ])
    
    # Главный скрипт
    build_args.append(main_script)
    
    print("\nПараметры сборки:")
    print(f"  {' '.join(build_args)}")
    
    print("\n[1/3] Запуск PyInstaller...")
    try:
        result = subprocess.run(build_args, check=True)
        print("✓ PyInstaller завершен успешно")
    except subprocess.CalledProcessError as e:
        print(f"✗ Ошибка PyInstaller: {e}")
        return False
    
    # Проверка результата
    dist_dir = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'dist')
    exe_name = 'CursorAI.exe' if sys.platform == 'win32' else 'CursorAI'
    exe_path = os.path.join(dist_dir, exe_name)
    
    if os.path.exists(exe_path):
        file_size = os.path.getsize(exe_path)
        print(f"\n✓ EXE файл создан: {exe_path}")
        print(f"  Размер: {file_size / (1024*1024):.2f} MB")
        
        print("\n[2/3] Очистка временных файлов...")
        # Удаление build директории
        build_dir = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'build')
        if os.path.exists(build_dir):
            shutil.rmtree(build_dir)
            print("✓ Удалена папка build/")
        
        # Удаление .spec файла
        spec_file = os.path.join(os.path.dirname(os.path.dirname(__file__)), 'CursorAI.spec')
        if os.path.exists(spec_file):
            os.remove(spec_file)
            print("✓ Удален файл CursorAI.spec")
        
        print("\n[3/3] Готово!")
        print("\nДля запуска:")
        print(f"  {exe_path}")
        
        return True
    else:
        print(f"\n✗ EXE файл не найден: {exe_path}")
        return False


def main():
    """Точка входа."""
    if len(sys.argv) > 1 and sys.argv[1] == '--help':
        print("Использование: python build_exe.py")
        print("\nСоздает исполняемый файл CursorAI.exe в папке dist/")
        return
    
    success = build_executable()
    
    if success:
        print("\n" + "=" * 60)
        print("  СБОРКА ЗАВЕРШЕНА УСПЕШНО!")
        print("=" * 60)
        sys.exit(0)
    else:
        print("\n" + "=" * 60)
        print("  ОШИБКА СБОРКИ")
        print("=" * 60)
        sys.exit(1)


if __name__ == '__main__':
    main()

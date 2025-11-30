# Добавление EXE файла в репозиторий

## Текущий статус

EXE файл будет автоматически добавлен в репозиторий после успешной сборки.

## Автоматическая сборка

Создан GitHub Actions workflow (`.github/workflows/build-exe.yml`), который:
- Автоматически собирает EXE при push в main
- Добавляет собранный EXE в репозиторий
- Сохраняет артефакт для скачивания

## Ручная сборка и добавление

Если вы хотите собрать и добавить EXE вручную:

### 1. Соберите EXE

**Windows:**
```cmd
scripts\build-mega-exe.bat
```

**Linux/Mac:**
```bash
./scripts/build-mega-exe.sh
```

### 2. Проверьте результат

EXE файл должен быть в `dist/windows/adaptive-entity-engine.exe`

### 3. Добавьте в git

```bash
git add dist/windows/adaptive-entity-engine.exe
git commit -m "Add built Mega EXE file"
git push
```

## Важно

- EXE файлы исключены из `.gitignore` только для `dist/**/*.exe`
- Размер EXE: 80-150 МБ
- Файл готов к распространению и не требует дополнительных зависимостей

## Структура

```
dist/
└── windows/
    ├── adaptive-entity-engine.exe  # МЕГА EXE файл (будет добавлен после сборки)
    ├── README.txt                  # Инструкция для пользователей
    ├── BUILD_NOTE.md               # Примечания о сборке
    └── .gitkeep                    # Сохраняет директорию в git
```

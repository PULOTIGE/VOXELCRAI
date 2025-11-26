# Статус сборки EXE

## Текущая ситуация

EXE файл должен быть собран на Windows машине или через GitHub Actions.

## Почему не собралось локально?

Кросскомпиляция Windows EXE на Linux требует:
- MSVC linker (часть Windows SDK)
- Дополнительные системные библиотеки

## Решения

### Вариант 1: GitHub Actions (рекомендуется)

Создан workflow `.github/workflows/build-exe.yml`, который автоматически:
- Соберет EXE на Windows runner
- Добавит его в репозиторий
- Создаст артефакт для скачивания

**Как запустить:**
1. Запушьте код в репозиторий
2. GitHub Actions автоматически соберет EXE
3. EXE будет добавлен в `dist/windows/`

### Вариант 2: Локальная сборка на Windows

Если у вас есть Windows машина:

```cmd
scripts\build-mega-exe.bat
```

Затем добавьте EXE в git:
```bash
git add dist/windows/adaptive-entity-engine.exe
git commit -m "Add built Mega EXE"
git push
```

### Вариант 3: Использовать готовый EXE

Если у вас уже есть собранный EXE:
1. Скопируйте его в `dist/windows/adaptive-entity-engine.exe`
2. Добавьте в git: `git add dist/windows/adaptive-entity-engine.exe`
3. Закоммитьте и запушьте

## Структура после добавления

```
dist/windows/
├── adaptive-entity-engine.exe  # ← МЕГА EXE файл (добавить сюда)
├── README.txt                  # Инструкция для пользователей
├── BUILD_NOTE.md               # Примечания о сборке
└── EXE_BUILD_STATUS.md         # Этот файл
```

## Размер файла

Ожидаемый размер: 80-150 МБ

## Проверка

После добавления EXE проверьте:
- ✅ Файл существует: `dist/windows/adaptive-entity-engine.exe`
- ✅ Размер файла: 80-150 МБ
- ✅ Файл добавлен в git: `git status` должен показать файл

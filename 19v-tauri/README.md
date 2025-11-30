# 19V v3.0 — автономный организм на Rust + Tauri

Новая версия организма **19V v3.0** упакована в отдельный Tauri-проект с нативным окном на `egui/eframe`, wgpu/Vulkan-пайплайном для LightPattern и автономным сбором концептов.

## Возможности

- 1280×720 окно с чёрным фоном и логом в нижней панели
- Реалистичный point-cloud из тысяч вокселей (цвет = энергия/эмоции)
- Кнопка «19В» запускает импульс перенастройки энергии
- Каждые 16 мс симулируются 62 млн нуклеотидов (сэмплинг, эмоции, энергия)
- GPU LightPattern на wgpu + Vulkan compute shader (хаос Лоренца, квантовый шум)
- Автономный режим: раз в 19 минут запросы в DuckDuckGo API по ключам «19V, Alushta, nttrl, KSB, CrimeaAI, хаос Лоренца, ДНК-кодирование»; новые концепты добавляются в организм
- Drag & drop файлов прямо в окно — данные добавляются в геном, лог фиксирует SHA256

## Структура

```
19v-tauri/
├── dist/                 # статический плейсхолдер для webview
├── src-tauri/
│   ├── Cargo.toml        # зависимости egui/wgpu/tauri
│   ├── Cargo.lock
│   ├── src/
│   │   ├── app.rs        # egui UI, point-cloud, drag & drop
│   │   ├── duckduckgo.rs # автономное обогащение
│   │   ├── light_pattern.rs + shaders/
│   │   └── simulation.rs # нуклеотиды, эмоции, лог
│   └── tauri.conf.json   # окно 1280×720, headless webview
└── README.md
```

## Системные требования

- Rust 1.82+ (для сборки ядра) и `cargo-tauri` 2.8.x. На Linux проще всего: `cargo +nightly install tauri-cli --version 2.8.1 --locked`. Ночная сборка нужна из‑за зависимостей с `edition2024`.
- Нативные зависимости Tauri:
  - **Linux**: `libgtk-3-dev libayatana-appindicator3-dev webkit2gtk-4.1 libfuse2 pkg-config cmake`.
  - **Windows**: Visual Studio Build Tools + `winget install --id Python.Python.3.12` для скриптов, `npm install --global @tauri-apps/cli` либо `cargo install tauri-cli`.
  - **macOS**: Xcode Command Line Tools.
- Vulkan-драйвер (проект тестировался с backend `wgpu::Backends::VULKAN`, корректно работает на Radeon VII).

## Запуск и сборка

```bash
# разработка (автоматически откроет egui-окно)
cd 19v-tauri
cargo +nightly tauri dev

# релизная сборка (создаст .app/.exe/.AppImage в src-tauri/target/release/bundle)
cargo +nightly tauri build --release
```

> ⚠️ В CI этой песочницы нет `glib-2.0`, поэтому `cargo tauri build` завершится ошибкой до установки системных пакетов. На локальной машине установите пакеты из раздела «Системные требования» и повторите сборку.

## Windows .exe

Для выпуска полноценного `.exe`:

1. Установите Visual Studio Build Tools (MSVC), Rust `x86_64-pc-windows-msvc`, и `tauri-cli`.
2. Выполните `cargo tauri build --target x86_64-pc-windows-msvc --release`.
3. Готовый `19V.exe` появится в `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/` и `bundle/nsis/`.

## DuckDuckGo токены

Проект использует публичный JSON API DuckDuckGo, поэтому отдельный ключ не нужен. Запрос происходит каждые 19 минут в отдельной асинхронной задаче, результаты добавляются в `SharedSimulation` и логируются.

## Drag & Drop

- Поддерживаются файлы до 1 МиБ, содержание хэшируется через SHA256 и внедряется в активные и тёплые блоки нуклеотидов.
- Лог показывает размер, имя файла и полученный хэш; это также влияет на цветовую схему point-cloud.

## LightPattern

- GPU compute shader `shaders/light_pattern.wgsl` моделирует энергетические всплески на основе уравнений Лоренца.
- Результат читается обратно в CPU, усредняется и используется в симуляции 62 млн виртуальных нуклеотидов.

## GitHub Releases

1. Создайте пустой репозиторий и выполните команды:
   ```bash
   git init
   git add .
   git commit -m "19V v3.0 autonomous shell"
   git branch -M main
   git remote add origin <repo_url>
   git push -u origin main
   ```
2. После `cargo tauri build --release` загрузите `.exe/.msi/.app` из `src-tauri/target/release/bundle` в раздел **Releases**.

## Roadmap

- [ ] Интеграция ArchGuard и Prometheus из исходного Adaptive Entity Engine
- [ ] Визуализация LightPattern в виде heatmap поверх point-cloud
- [ ] Сжатие логов и экспорт состояний организма

Добро пожаловать в организм 19В! 

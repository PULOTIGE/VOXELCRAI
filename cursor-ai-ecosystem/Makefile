.PHONY: help install run clean build test examples

help:
	@echo "Cursor AI Ecosystem - Makefile"
	@echo ""
	@echo "Команды:"
	@echo "  make install     - Установка зависимостей"
	@echo "  make run         - Запуск экосистемы"
	@echo "  make run-no-vis  - Запуск без визуализации"
	@echo "  make examples    - Запуск примеров"
	@echo "  make build       - Сборка в EXE"
	@echo "  make clean       - Очистка временных файлов"
	@echo "  make test        - Запуск тестов (TODO)"

install:
	@echo "Установка зависимостей..."
	pip install -r requirements.txt
	@echo "✓ Готово!"

run:
	@echo "Запуск экосистемы..."
	python main.py

run-no-vis:
	@echo "Запуск без визуализации..."
	python main.py --no-visualization

run-small:
	@echo "Запуск (легкий режим)..."
	python main.py --nucleotides 10000 --voxels 100

examples:
	@echo "Запуск примеров..."
	python examples/simple_usage.py

build:
	@echo "Сборка в EXE..."
	python scripts/build_exe.py

clean:
	@echo "Очистка..."
	find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete 2>/dev/null || true
	find . -type f -name "*.pyo" -delete 2>/dev/null || true
	rm -rf build/ dist/ *.spec 2>/dev/null || true
	@echo "✓ Очистка завершена"

test:
	@echo "Тесты еще не реализованы"
	@echo "TODO: pytest"

#!/bin/bash
# Script to run 4K benchmark and collect results

echo "=== Запуск 4K Benchmark для RTX 4070 ==="
echo ""

# Check if running in headless mode
if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ]; then
    echo "Предупреждение: DISPLAY не установлен. Бенчмарк может не запуститься."
    echo "Попытка запуска с Xvfb (виртуальный дисплей)..."
    
    # Try to use Xvfb if available
    if command -v xvfb-run &> /dev/null; then
        echo "Использование xvfb-run для виртуального дисплея"
        xvfb-run -a -s "-screen 0 3840x2160x24" timeout 60 cargo run --release --bin benchmark light 2>&1 | tee benchmark_results.txt
    else
        echo "Ошибка: требуется графический дисплей или xvfb-run"
        echo "Установите xvfb: sudo apt-get install xvfb"
        exit 1
    fi
else
    echo "Запуск бенчмарка на доступном дисплее..."
    timeout 60 cargo run --release --bin benchmark light 2>&1 | tee benchmark_results.txt
fi

echo ""
echo "=== Результаты сохранены в benchmark_results.txt ==="

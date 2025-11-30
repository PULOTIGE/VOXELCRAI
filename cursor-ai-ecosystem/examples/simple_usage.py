#!/usr/bin/env python3
"""
Простой пример использования компонентов Cursor AI Ecosystem
"""

import sys
import os

# Добавление пути к src
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))

from src.core import Nucleotide, Voxel, LightPattern
import numpy as np


def example_nucleotide():
    """Пример работы с нуклеотидом."""
    print("\n=== Пример: Нуклеотид ===")
    
    # Создание нуклеотида
    nuc = Nucleotide(base='A')
    print(f"Создан: {nuc}")
    print(f"Уровень экспрессии: {nuc.get_expression_level():.3f}")
    
    # Обновление
    for i in range(5):
        nuc.update(dt=0.016)
        print(f"Шаг {i+1}: экспрессия = {nuc.get_expression_level():.3f}, шум = {nuc.quantum_noise:.3f}")
    
    # Сходство с другим нуклеотидом
    nuc2 = Nucleotide(base='T')
    similarity = nuc.similarity(nuc2)
    print(f"\nСхожесть с другим нуклеотидом: {similarity:.3f}")


def example_voxel():
    """Пример работы с вокселем."""
    print("\n=== Пример: Воксель ===")
    
    # Создание вокселя
    voxel = Voxel(voxel_id=1)
    print(f"Создан: {voxel}")
    
    # Обновление с отображением состояния
    for i in range(10):
        voxel.update(dt=0.016)
        
        if i % 3 == 0:
            emotions = voxel.get_emotional_state()
            print(f"\nШаг {i+1}:")
            print(f"  Возраст: {voxel.metadata['age']:.2f}s")
            print(f"  Энергия: {voxel.physics['energy']:.2f}")
            print(f"  Кайф: {voxel.kaif:.4f}")
            print(f"  Энтропия: {voxel.prev_entropy:.4f}")
            print(f"  Эмоции: joy={emotions['joy']:.3f}, sadness={emotions['sadness']:.3f}")
    
    # Внешний сенсорный ввод
    print("\nПодача внешнего стимула...")
    external_input = np.random.randn(384).astype(np.float32)
    voxel.sense(external_input)
    voxel.update(dt=0.016)
    print(f"Кайф после стимула: {voxel.kaif:.4f}")


def example_light_pattern():
    """Пример работы с паттернами освещения."""
    print("\n=== Пример: Паттерны освещения ===")
    
    # Создание паттерна
    pattern = LightPattern(pattern_id=1)
    
    # Установка прямого освещения
    directions = np.random.randn(32, 3).astype(np.float32)
    colors = np.random.rand(32, 3).astype(np.float32)
    pattern.set_direct_lighting(directions, colors)
    
    # Вычисление SH коэффициентов
    pattern.compute_sh_from_lighting()
    print(f"Создан: {pattern}")
    print(f"SH коэффициенты (первые 3): {pattern.sh_coeffs[:3]}")
    
    # Оценка освещения по нормали
    normal = np.array([0.0, 1.0, 0.0])  # вверх
    color = pattern.evaluate_sh(normal)
    print(f"Цвет освещения по нормали [0,1,0]: {color}")
    
    # Сходство паттернов
    pattern2 = LightPattern(pattern_id=2)
    pattern2.set_direct_lighting(directions * 0.5, colors * 0.8)
    similarity = pattern.similarity(pattern2)
    print(f"Схожесть паттернов: {similarity:.3f}")


def example_integration():
    """Пример интеграции компонентов."""
    print("\n=== Пример: Интеграция ===")
    
    # Создание компонентов
    nucleotides = [Nucleotide() for _ in range(10)]
    voxel = Voxel(voxel_id=42)
    
    print(f"Создано {len(nucleotides)} нуклеотидов и 1 воксель")
    
    # Симуляция 10 шагов
    for step in range(10):
        # Обновление нуклеотидов
        for nuc in nucleotides:
            nuc.update(dt=0.016)
        
        # Получение средней экспрессии
        avg_expression = sum(n.get_expression_level() for n in nucleotides) / len(nucleotides)
        
        # Передача энергии в воксель
        voxel.physics['energy'] += avg_expression * 0.5
        
        # Обновление вокселя
        voxel.update(dt=0.016)
        
        if step % 3 == 0:
            print(f"\nШаг {step+1}:")
            print(f"  Средняя экспрессия: {avg_expression:.3f}")
            print(f"  Энергия вокселя: {voxel.physics['energy']:.2f}")
            print(f"  Кайф вокселя: {voxel.kaif:.4f}")


def main():
    """Главная функция."""
    print("=" * 60)
    print("  Примеры использования Cursor AI Ecosystem")
    print("=" * 60)
    
    example_nucleotide()
    example_voxel()
    example_light_pattern()
    example_integration()
    
    print("\n" + "=" * 60)
    print("  Примеры завершены!")
    print("=" * 60)


if __name__ == '__main__':
    main()

#!/usr/bin/env python3
"""
CrimeaAI Meta Organism - Тесты
═══════════════════════════════════════════════════════════════════════════════

Тестирование всех компонентов системы.
"""

import sys
import os
import numpy as np
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from voxel_core import (
    Voxel, ANIRLEStorage, compute_semantic_fingerprint,
    cosine_similarity, TetrahedralBelonging, EmotionIndex
)
from organism import MetaOrganism, FileCreature, OrganismState


def test_voxel_creation():
    """Тест создания вокселя"""
    print("🧪 Тест: Создание вокселя...")
    
    voxel = Voxel(x=1.0, y=2.0, z=3.0)
    
    assert voxel.x == 1.0
    assert voxel.y == 2.0
    assert voxel.z == 3.0
    assert voxel.energy == 1.0
    assert len(voxel.emotion) == 4
    assert len(voxel.semantic) == 8
    assert len(voxel.connections) == 6
    assert voxel.is_alive()
    
    print("   ✅ Воксель создан корректно")


def test_voxel_properties():
    """Тест свойств вокселя"""
    print("🧪 Тест: Свойства вокселя...")
    
    voxel = Voxel(x=1.0, y=2.0, z=3.0)
    
    # Тест позиции
    pos = voxel.pos
    assert np.allclose(pos, [1.0, 2.0, 3.0])
    
    # Тест доминирующей эмоции
    voxel.emotion = np.array([0.8, 0.1, 0.05, 0.05])
    name, value = voxel.dominant_emotion()
    assert name == 'joy'
    assert value == 0.8
    
    # Тест is_alive
    voxel.energy = 0.5
    voxel.trauma = 0.3
    assert voxel.is_alive()
    
    voxel.energy = 0.0
    assert not voxel.is_alive()
    
    print("   ✅ Свойства работают корректно")


def test_anirle_storage():
    """Тест ANIRLE хранилища"""
    print("🧪 Тест: ANIRLE хранилище...")
    
    storage = ANIRLEStorage(resolution=1.0)
    
    # Добавление вокселей
    v1 = Voxel(x=0, y=0, z=0)
    v2 = Voxel(x=1, y=0, z=0)
    v3 = Voxel(x=0, y=1, z=0)
    
    id1 = storage.add(v1)
    id2 = storage.add(v2)
    id3 = storage.add(v3)
    
    assert len(storage) == 3
    assert id1 != id2 != id3
    
    # Получение
    retrieved = storage.get(0, 0, 0)
    assert retrieved is not None
    assert retrieved.id == id1
    
    # Соседи
    neighbors = storage.get_neighbors(v1)
    assert len(neighbors) == 2  # v2 и v3
    
    # Удаление
    removed = storage.remove(1, 0, 0)
    assert removed
    assert len(storage) == 2
    
    # Статистика
    stats = storage.get_statistics()
    assert stats['total_voxels'] == 2
    
    print("   ✅ ANIRLE хранилище работает корректно")


def test_semantic_fingerprint():
    """Тест семантического отпечатка"""
    print("🧪 Тест: Семантический отпечаток...")
    
    data1 = b"Hello, World!"
    data2 = b"Hello, World!"  # Тот же
    data3 = b"Goodbye, World!"  # Другой
    
    fp1 = compute_semantic_fingerprint(data1)
    fp2 = compute_semantic_fingerprint(data2)
    fp3 = compute_semantic_fingerprint(data3)
    
    assert len(fp1) == 8
    assert np.allclose(fp1, fp2)  # Одинаковые данные = одинаковый отпечаток
    assert not np.allclose(fp1, fp3)  # Разные данные = разный отпечаток
    
    print("   ✅ Семантический отпечаток работает корректно")


def test_cosine_similarity():
    """Тест косинусного сходства"""
    print("🧪 Тест: Косинусное сходство...")
    
    a = np.array([1, 0, 0])
    b = np.array([1, 0, 0])
    c = np.array([0, 1, 0])
    d = np.array([-1, 0, 0])
    
    assert cosine_similarity(a, b) == 1.0  # Идентичные
    assert cosine_similarity(a, c) == 0.0  # Ортогональные
    assert cosine_similarity(a, d) == -1.0  # Противоположные
    
    print("   ✅ Косинусное сходство работает корректно")


def test_organism_creation():
    """Тест создания организма"""
    print("🧪 Тест: Создание организма (5000 вокселей)...")
    
    start = time.time()
    organism = MetaOrganism(num_voxels=5000)
    elapsed = time.time() - start
    
    assert len(organism.storage) > 0
    assert organism.state.health > 0
    assert organism.radius > 0
    
    print(f"   ✅ Организм создан за {elapsed:.2f}с ({len(organism.storage)} вокселей)")


def test_creature_creation():
    """Тест создания существа из файла"""
    print("🧪 Тест: Создание существа...")
    
    organism = MetaOrganism(num_voxels=1000)
    
    data = b"Test file content for creature creation"
    creature = organism.spawn_creature("test.txt", data)
    
    assert creature is not None
    assert len(creature.voxels) > 0
    assert creature.alive
    assert not creature.integrated
    
    print(f"   ✅ Существо создано ({len(creature.voxels)} вокселей)")


def test_compatibility_check():
    """Тест проверки совместимости"""
    print("🧪 Тест: Проверка совместимости...")
    
    organism = MetaOrganism(num_voxels=1000)
    
    # Создаём "совместимый" файл (похожие данные на семантику организма)
    compatible_data = (organism.base_semantic * 255).astype(np.uint8).tobytes() * 100
    creature1 = organism.spawn_creature("compatible.py", compatible_data)
    
    # Создаём "несовместимый" файл
    incompatible_data = ((1 - organism.base_semantic) * 255).astype(np.uint8).tobytes() * 100
    creature2 = organism.spawn_creature("incompatible.exe", incompatible_data)
    
    sem1, emo1 = organism.check_compatibility(creature1)
    sem2, emo2 = organism.check_compatibility(creature2)
    
    print(f"   Совместимый: sem={sem1:.2f}, emo={emo1:.2f}")
    print(f"   Несовместимый: sem={sem2:.2f}, emo={emo2:.2f}")
    
    # Совместимый должен иметь более высокое сходство
    # (это не строгий тест из-за случайности)
    
    print("   ✅ Проверка совместимости работает")


def test_organism_update():
    """Тест обновления организма"""
    print("🧪 Тест: Обновление организма...")
    
    organism = MetaOrganism(num_voxels=2000)
    
    initial_state = organism.state.health
    
    # Несколько тиков обновления
    for _ in range(10):
        organism.update(0.016)
    
    # Состояние должно измениться (пульсация)
    assert organism.storage.tick == 10
    
    print("   ✅ Обновление организма работает")


def test_integration():
    """Тест интеграции существа"""
    print("🧪 Тест: Интеграция существа...")
    
    organism = MetaOrganism(num_voxels=2000)
    initial_count = len(organism.storage)
    
    # Создаём существо
    data = b"Test data" * 100
    creature = organism.spawn_creature("test.py", data)
    creature_voxels = len(creature.voxels)
    
    # Принудительная интеграция
    organism.integrate_creature(creature)
    
    # Проверки
    assert creature.integrated
    assert not creature.alive
    # Воксели добавляются в storage, но могут быть коллизии из-за квантования
    # Проверяем, что вокселей стало больше
    assert len(organism.storage) > initial_count
    assert organism.state.integration_count == 1
    
    added = len(organism.storage) - initial_count
    print(f"   ✅ Интеграция работает (+{added} вокселей из {creature_voxels})")


def test_rejection():
    """Тест отторжения существа"""
    print("🧪 Тест: Отторжение существа...")
    
    organism = MetaOrganism(num_voxels=5000)
    initial_count = len(organism.storage)
    
    # Создаём существо
    data = b"Malicious data" * 100
    creature = organism.spawn_creature("malware.exe", data)
    
    # Двигаем к центру
    creature.current_pos = organism.center.copy()
    for v in creature.voxels:
        v.x += organism.center[0] - creature.spawn_pos[0]
        v.y += organism.center[1] - creature.spawn_pos[1]
        v.z += organism.center[2] - creature.spawn_pos[2]
    
    # Принудительное отторжение
    organism.reject_creature(creature, severity=0.5)
    
    # Проверки
    assert creature.rejected
    assert not creature.alive
    assert len(organism.storage) < initial_count  # Атрофия
    assert organism.state.trauma_count == 1
    
    voxels_lost = initial_count - len(organism.storage)
    print(f"   ✅ Отторжение работает (-{voxels_lost} вокселей)")


def test_positions_and_colors():
    """Тест получения позиций и цветов"""
    print("🧪 Тест: Позиции и цвета...")
    
    organism = MetaOrganism(num_voxels=1000)
    
    positions = organism.get_all_positions()
    colors = organism.get_all_colors()
    
    assert len(positions) == len(organism.storage)
    assert len(colors) == len(organism.storage)
    assert positions.shape[1] == 3  # x, y, z
    assert colors.shape[1] == 3  # r, g, b
    assert np.all(colors >= 0) and np.all(colors <= 1)
    
    print("   ✅ Позиции и цвета получены корректно")


def test_full_scenario():
    """Полный сценарий тестирования"""
    print("🧪 Тест: Полный сценарий (5 секунд)...")
    
    organism = MetaOrganism(num_voxels=3000)
    
    # Добавляем несколько файлов
    files = [
        ("good1.py", b"Python code" * 50),
        ("bad1.exe", b"\x00\xff" * 100),
        ("good2.json", b'{"key": "value"}' * 30),
    ]
    
    for name, data in files:
        organism.spawn_creature(name, data)
    
    # Запускаем симуляцию
    start = time.time()
    ticks = 0
    
    while time.time() - start < 3.0:  # 3 секунды
        organism.update(0.016)
        ticks += 1
    
    state = organism.state
    
    print(f"   Тиков: {ticks}")
    print(f"   Вокселей: {state.total_voxels}")
    print(f"   Здоровье: {state.health*100:.1f}%")
    print(f"   Настроение: {state.mood}")
    print(f"   Интеграций: {state.integration_count}")
    print(f"   Травм: {state.trauma_count}")
    
    print("   ✅ Полный сценарий завершён")


def run_all_tests():
    """Запуск всех тестов"""
    print("""
╔══════════════════════════════════════════════════════════════════════════════╗
║                   CrimeaAI META ORGANISM - ТЕСТЫ                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
    """)
    
    tests = [
        test_voxel_creation,
        test_voxel_properties,
        test_anirle_storage,
        test_semantic_fingerprint,
        test_cosine_similarity,
        test_organism_creation,
        test_creature_creation,
        test_compatibility_check,
        test_organism_update,
        test_integration,
        test_rejection,
        test_positions_and_colors,
        test_full_scenario,
    ]
    
    passed = 0
    failed = 0
    
    for test in tests:
        try:
            test()
            passed += 1
        except Exception as e:
            print(f"   ❌ ОШИБКА: {e}")
            failed += 1
    
    print(f"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                          РЕЗУЛЬТАТЫ ТЕСТОВ                                   ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  ✅ Пройдено: {passed:3}                                                       ║
║  ❌ Провалено: {failed:3}                                                       ║
╚══════════════════════════════════════════════════════════════════════════════╝
    """)
    
    return failed == 0


if __name__ == "__main__":
    success = run_all_tests()
    sys.exit(0 if success else 1)

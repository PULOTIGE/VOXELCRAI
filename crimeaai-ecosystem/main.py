#!/usr/bin/env python3
"""
CrimeaAI Ecosystem - Main Entry Point
=====================================

Запуск AI-экосистемы с биологическими структурами данных,
эмоциональной моделью и поиском концептов.

Использование:
    python main.py              # Запуск с GUI
    python main.py --no-gui     # Запуск без GUI (только консоль)
    python main.py --test       # Запуск тестов
"""

import sys
import os
import argparse
import asyncio

# Добавляем путь к модулям
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


def print_banner():
    """Вывод баннера"""
    banner = """
    ╔═══════════════════════════════════════════════════════════╗
    ║                                                           ║
    ║   ██████╗██████╗ ██╗███╗   ███╗███████╗ █████╗            ║
    ║  ██╔════╝██╔══██╗██║████╗ ████║██╔════╝██╔══██╗           ║
    ║  ██║     ██████╔╝██║██╔████╔██║█████╗  ███████║           ║
    ║  ██║     ██╔══██╗██║██║╚██╔╝██║██╔══╝  ██╔══██║           ║
    ║  ╚██████╗██║  ██║██║██║ ╚═╝ ██║███████╗██║  ██║           ║
    ║   ╚═════╝╚═╝  ╚═╝╚═╝╚═╝     ╚═╝╚══════╝╚═╝  ╚═╝           ║
    ║                                                           ║
    ║          🧠 AI ECOSYSTEM v1.0 🧬                          ║
    ║                                                           ║
    ║   • Nucleotide Pool (256 bytes per cell)                  ║
    ║   • Voxel World (9KB micro-organisms)                     ║
    ║   • LightPattern Database (1KB patterns)                  ║
    ║   • Kaif Engine (entropy derivative)                      ║
    ║   • Concept Search (DuckDuckGo integration)               ║
    ║                                                           ║
    ╚═══════════════════════════════════════════════════════════╝
    """
    print(banner)


def run_gui():
    """Запуск с графическим интерфейсом"""
    from ui.main_window import CrimeaAIApp, AppConfig
    
    config = AppConfig(
        window_width=1400,
        window_height=900,
        target_fps=60,
        nucleotide_pool_size=10000,
        initial_voxels=50
    )
    
    app = CrimeaAIApp(config)
    app.run()


async def run_headless():
    """Запуск без GUI (консольный режим)"""
    from core.nucleotide import NucleotidePool
    from core.voxel import VoxelWorld
    from core.kaif_engine import KaifEngine
    from core.concept_search import ConceptSearcher
    from core.scheduler import CrimeaScheduler, TaskPriority
    
    print("🚀 Запуск в консольном режиме...")
    
    # Инициализация
    print("🧬 Инициализация пула нуклеотидов...")
    nucleotide_pool = NucleotidePool(size=10000)
    nucleotide_pool.initialize(random_init=True)
    
    print("🌍 Инициализация мира вокселей...")
    voxel_world = VoxelWorld(max_voxels=100)
    import random
    for _ in range(20):
        pos = (random.uniform(-10, 10), random.uniform(-10, 10), 0)
        voxel_world.spawn_voxel(pos)
    
    print("⚡ Инициализация KaifEngine...")
    kaif_engine = KaifEngine()
    kaif_engine.register_component('nucleotides', weight=0.5)
    kaif_engine.register_component('voxels', weight=0.5)
    
    print("🔍 Инициализация поиска концептов...")
    concept_searcher = ConceptSearcher(
        base_keywords=['AI', 'neural network', 'machine learning']
    )
    
    # Планировщик
    scheduler = CrimeaScheduler()
    
    def update_all():
        nucleotide_pool.update_all(0.016)
        voxel_world.update(0.016)
        
        # Обновляем kaif
        import numpy as np
        nuc_sample = nucleotide_pool.semantic_matrix[:100].flatten() if nucleotide_pool.semantic_matrix is not None else np.zeros(100)
        kaif_engine.update_component('nucleotides', nuc_sample)
        
        emotions = []
        for voxel in list(voxel_world.voxels.values())[:20]:
            emotions.extend(voxel.emotions.emotion_vector[:5])
        if emotions:
            kaif_engine.update_component('voxels', np.array(emotions))
        
        kaif_engine.update(0.016)
    
    scheduler.add_task(
        name="main_update",
        callback=update_all,
        interval=0.016,
        priority=TaskPriority.HIGH
    )
    
    print("\n✅ Система запущена! Нажмите Ctrl+C для выхода.\n")
    
    # Главный цикл
    tick = 0
    try:
        while True:
            update_all()
            tick += 1
            
            # Выводим статистику каждые 100 тиков
            if tick % 100 == 0:
                stats = kaif_engine.get_statistics()
                voxel_stats = voxel_world.get_statistics()
                
                print(f"[Tick {tick:6d}] "
                      f"Kaif: {stats['smoothed_kaif']:.4f} ({stats['state']}) | "
                      f"Voxels: {voxel_stats['voxel_count']} | "
                      f"Health: {voxel_stats['avg_health']:.2%} | "
                      f"Concepts: {len(concept_searcher.concepts)}")
            
            # Поиск концептов каждые 1000 тиков (для теста)
            if tick % 1000 == 0 and tick > 0:
                print("\n🔍 Поиск концептов...")
                concepts = concept_searcher.search_concepts()
                print(f"✅ Найдено {len(concepts)} новых концептов\n")
            
            await asyncio.sleep(0.016)
    
    except KeyboardInterrupt:
        print("\n\n🛑 Остановка...")
        print(f"📊 Итого: {tick} тиков, {len(concept_searcher.concepts)} концептов")
        print("👋 До свидания!")


def run_tests():
    """Запуск тестов"""
    print("🧪 Запуск тестов...")
    
    # Тест нуклеотидов
    print("\n📋 Тест Nucleotide...")
    from core.nucleotide import Nucleotide, NucleotideBase, EpigeneticTag
    import numpy as np
    
    nuc = Nucleotide(base=NucleotideBase.ADENINE)
    nuc.semantic_vector = np.random.randn(512).astype(np.float16)
    nuc.update(0.016)
    nuc.add_epigenetic_tag(EpigeneticTag.METHYLATION, 0.8)
    
    # Тест сериализации
    data = nuc.to_bytes()
    assert len(data) == 256, f"Expected 256 bytes, got {len(data)}"
    nuc_restored = Nucleotide.from_bytes(data)
    assert nuc_restored.base == nuc.base
    print("✅ Nucleotide тест пройден")
    
    # Тест вокселей
    print("\n📋 Тест Voxel...")
    from core.voxel import Voxel, VoxelWorld
    
    voxel = Voxel(voxel_id=1)
    voxel.update(0.016)
    state = voxel.get_state()
    assert 'kaif' in state
    assert 'health' in state
    print(f"   Voxel state: kaif={state['kaif']:.4f}, health={state['health']:.2%}")
    print("✅ Voxel тест пройден")
    
    # Тест мира вокселей
    print("\n📋 Тест VoxelWorld...")
    world = VoxelWorld(max_voxels=10)
    for i in range(5):
        world.spawn_voxel((i, i, 0))
    
    world.update(0.016)
    stats = world.get_statistics()
    assert stats['voxel_count'] == 5
    print(f"   World stats: {stats}")
    print("✅ VoxelWorld тест пройден")
    
    # Тест паттернов освещения
    print("\n📋 Тест LightPattern...")
    from core.light_pattern import LightPattern, PatternDatabase
    
    pattern = LightPattern()
    pattern.direct_lighting = np.random.rand(32, 3).astype(np.float32)
    
    data = pattern.to_bytes()
    assert len(data) == 1024, f"Expected 1024 bytes, got {len(data)}"
    pattern_restored = LightPattern.from_bytes(data)
    print("✅ LightPattern тест пройден")
    
    # Тест базы паттернов
    print("\n📋 Тест PatternDatabase...")
    db = PatternDatabase(max_patterns=100)
    db.generate_random_patterns(10)
    
    query = np.random.rand(300).astype(np.float32)
    similar = db.find_similar(query, top_k=3)
    assert len(similar) == 3
    print(f"   Found {len(similar)} similar patterns")
    print("✅ PatternDatabase тест пройден")
    
    # Тест KaifEngine
    print("\n📋 Тест KaifEngine...")
    from core.kaif_engine import KaifEngine, compute_entropy
    
    engine = KaifEngine()
    engine.register_component('test', np.random.randn(64))
    
    for _ in range(10):
        engine.update_component('test', np.random.randn(64))
        engine.update(0.016)
    
    stats = engine.get_statistics()
    print(f"   Kaif stats: {stats}")
    print("✅ KaifEngine тест пройден")
    
    # Тест поиска концептов
    print("\n📋 Тест ConceptSearcher...")
    from core.concept_search import ConceptSearcher, ConceptExtractor
    
    extractor = ConceptExtractor()
    terms = extractor.extract_terms("Machine learning is a subset of artificial intelligence")
    assert len(terms) > 0
    print(f"   Extracted terms: {terms[:5]}")
    
    searcher = ConceptSearcher()
    # Симуляция поиска (не делаем реальный запрос в тесте)
    concepts = searcher._simulate_search(['AI'])
    assert len(concepts) > 0
    print(f"   Simulated concepts: {[c.term for c in concepts[:3]]}")
    print("✅ ConceptSearcher тест пройден")
    
    print("\n" + "="*50)
    print("🎉 Все тесты пройдены успешно!")
    print("="*50)


def main():
    """Главная функция"""
    parser = argparse.ArgumentParser(
        description="CrimeaAI Ecosystem - Bio-inspired AI System"
    )
    parser.add_argument(
        '--no-gui', 
        action='store_true',
        help='Запуск без графического интерфейса'
    )
    parser.add_argument(
        '--test',
        action='store_true',
        help='Запуск тестов'
    )
    parser.add_argument(
        '--nucleotides',
        type=int,
        default=10000,
        help='Количество нуклеотидов в пуле'
    )
    parser.add_argument(
        '--voxels',
        type=int,
        default=50,
        help='Начальное количество вокселей'
    )
    
    args = parser.parse_args()
    
    print_banner()
    
    if args.test:
        run_tests()
    elif args.no_gui:
        asyncio.run(run_headless())
    else:
        run_gui()


if __name__ == "__main__":
    main()

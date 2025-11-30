#!/usr/bin/env python3
"""
Cursor AI Ecosystem - Главный файл запуска
"""

import asyncio
import time
import argparse
from typing import Dict

from src.core import NucleotidePool, VoxelGrid, LightPatternDatabase
from src.utils import ConceptSearcher, ConceptIntegrator, TickerScheduler
from src.visualization import EcosystemDisplay


class CursorAIEcosystem:
    """
    Главный класс экосистемы ИИ.
    Управляет всеми компонентами и координирует их работу.
    """
    
    def __init__(self, 
                 num_nucleotides: int = 100000,
                 num_voxels: int = 400,
                 enable_visualization: bool = True,
                 enable_concept_search: bool = True):
        """
        Инициализация экосистемы.
        
        Args:
            num_nucleotides: Количество нуклеотидов
            num_voxels: Количество вокселей
            enable_visualization: Включить визуализацию
            enable_concept_search: Включить поиск концептов
        """
        print(f"[Ecosystem] Инициализация экосистемы...")
        print(f"  Нуклеотидов: {num_nucleotides}")
        print(f"  Вокселей: {num_voxels}")
        print(f"  Визуализация: {enable_visualization}")
        print(f"  Поиск концептов: {enable_concept_search}")
        
        # Ядро
        self.nucleotide_pool = NucleotidePool(num_nucleotides)
        self.voxel_grid = VoxelGrid(num_voxels)
        self.light_db = LightPatternDatabase()
        
        # Утилиты
        self.concept_searcher = ConceptSearcher(['19V', 'CrimeaAI', 'consciousness', 'neural networks'])
        self.concept_integrator = ConceptIntegrator()
        
        # Планировщики
        self.ticker = TickerScheduler(fps=60)
        
        # Визуализация
        self.display = EcosystemDisplay() if enable_visualization else None
        self.enable_visualization = enable_visualization
        self.enable_concept_search = enable_concept_search
        
        # Состояние
        self.start_time = time.time()
        self.running = False
        
        # Настройка задач
        self._setup_tasks()
        
        print("[Ecosystem] Инициализация завершена!")
    
    def _setup_tasks(self):
        """Настройка периодических задач."""
        # Основной тик: обновление нуклеотидов и вокселей
        self.ticker.add_task('update_nucleotides', self._update_nucleotides, interval=0.016)
        self.ticker.add_task('update_voxels', self._update_voxels, interval=0.016)
        
        # Поиск концептов каждые 19 минут (1140 секунд)
        if self.enable_concept_search:
            # Для тестирования используем меньший интервал
            self.ticker.add_task('search_concepts', self._search_concepts, interval=60)  # 60 сек для теста
        
        # Статистика каждые 5 секунд
        self.ticker.add_task('print_stats', self._print_stats, interval=5.0)
    
    def _update_nucleotides(self, dt: float):
        """Обновление нуклеотидов."""
        # Обновляем только подвыборку для производительности
        sample_size = min(1000, len(self.nucleotide_pool.nucleotides))
        import random
        sample = random.sample(self.nucleotide_pool.nucleotides, sample_size)
        for nuc in sample:
            nuc.update(dt)
    
    def _update_voxels(self, dt: float):
        """Обновление вокселей."""
        self.voxel_grid.update_all(dt)
        
        # Передача энергии от нуклеотидов (случайно)
        if len(self.nucleotide_pool.nucleotides) > 0:
            for voxel in self.voxel_grid.voxels[:10]:  # первые 10
                if voxel.metadata['alive']:
                    nuc = self.nucleotide_pool.get_random_nucleotide()
                    expression = nuc.get_expression_level()
                    voxel.physics['energy'] += expression * 0.1
    
    def _search_concepts(self, dt: float = None):
        """Поиск новых концептов."""
        print("[Ecosystem] Поиск новых концептов...")
        try:
            concepts = self.concept_searcher.search_concepts()
            
            if concepts:
                print(f"[Ecosystem] Найдено {len(concepts)} концептов: {concepts[:5]}")
                
                # Интеграция в систему
                for concept in concepts:
                    self.concept_integrator.integrate_concept(concept, source='duckduckgo')
                
                # Добавление в ключевые слова
                self.concept_searcher.add_keywords(concepts[:3])
            else:
                print("[Ecosystem] Концепты не найдены")
        except Exception as e:
            print(f"[Ecosystem] Ошибка поиска концептов: {e}")
    
    def _print_stats(self, dt: float = None):
        """Вывод статистики."""
        uptime = time.time() - self.start_time
        voxel_stats = self.voxel_grid.get_stats()
        nuc_stats = self.nucleotide_pool.get_stats()
        concept_stats = self.concept_searcher.get_stats()
        
        print(f"\n[Stats] Uptime: {uptime:.1f}s | FPS: {self.ticker.actual_fps:.1f}")
        print(f"  Voxels: {voxel_stats['alive_count']}/{self.voxel_grid.num_voxels} alive")
        print(f"  Avg Kaif: {voxel_stats['avg_kaif']:.4f}")
        print(f"  Avg Energy: {voxel_stats['avg_energy']:.2f}")
        print(f"  Concepts: {concept_stats['unique_concepts']} unique")
    
    def get_ecosystem_state(self) -> Dict:
        """
        Получение текущего состояния экосистемы.
        
        Returns:
            Словарь с состоянием
        """
        uptime = time.time() - self.start_time
        voxel_stats = self.voxel_grid.get_stats()
        
        return {
            'uptime': uptime,
            'fps': self.ticker.actual_fps,
            'voxels': self.voxel_grid.voxels,
            'voxels_alive': voxel_stats['alive_count'],
            'avg_kaif': voxel_stats['avg_kaif'],
            'avg_energy': voxel_stats['avg_energy'],
            'concepts_found': self.concept_searcher.get_stats()['unique_concepts'],
            'recent_concepts': list(self.concept_searcher.discovered_concepts)[-10:] if self.concept_searcher.discovered_concepts else []
        }
    
    async def run(self):
        """Основной цикл выполнения экосистемы."""
        print("[Ecosystem] Запуск экосистемы...")
        self.running = True
        
        # Запуск планировщика в фоне
        ticker_task = asyncio.create_task(self.ticker.run())
        
        try:
            # Основной цикл
            while self.running:
                # Обработка визуализации
                if self.enable_visualization and self.display:
                    if not self.display.handle_events():
                        self.running = False
                        break
                    
                    # Обновление состояния для визуализации
                    state = self.get_ecosystem_state()
                    
                    # Обновление истории
                    if state['voxels_alive'] > 0:
                        self.display.update_history(
                            state['avg_kaif'],
                            state['voxels'][0].prev_entropy if state['voxels'] else 0.0,
                            state['avg_energy']
                        )
                    
                    # Отрисовка
                    self.display.draw_full_dashboard(state)
                    self.display.update()
                else:
                    # Без визуализации - просто ждем
                    await asyncio.sleep(0.1)
        
        except KeyboardInterrupt:
            print("\n[Ecosystem] Получен сигнал прерывания...")
        finally:
            self.stop()
            ticker_task.cancel()
            try:
                await ticker_task
            except asyncio.CancelledError:
                pass
    
    def stop(self):
        """Остановка экосистемы."""
        print("[Ecosystem] Остановка...")
        self.running = False
        self.ticker.stop()
        
        if self.display:
            self.display.close()
        
        # Финальная статистика
        print("\n[Ecosystem] Финальная статистика:")
        print(f"  Nucleotides: {self.nucleotide_pool.get_stats()}")
        print(f"  Voxels: {self.voxel_grid.get_stats()}")
        print(f"  Concepts: {self.concept_searcher.get_stats()}")
        print(f"  Integrator: {self.concept_integrator.get_stats()}")
        
        print("[Ecosystem] Завершение работы.")


def main():
    """Точка входа в программу."""
    parser = argparse.ArgumentParser(description='Cursor AI Ecosystem')
    parser.add_argument('--nucleotides', type=int, default=100000,
                       help='Количество нуклеотидов (по умолчанию: 100000)')
    parser.add_argument('--voxels', type=int, default=400,
                       help='Количество вокселей (по умолчанию: 400)')
    parser.add_argument('--no-visualization', action='store_true',
                       help='Отключить визуализацию')
    parser.add_argument('--no-search', action='store_true',
                       help='Отключить поиск концептов')
    
    args = parser.parse_args()
    
    print("=" * 60)
    print("  CURSOR AI ECOSYSTEM - 19V")
    print("  Экосистема самоорганизующегося ИИ")
    print("=" * 60)
    print()
    
    # Создание экосистемы
    ecosystem = CursorAIEcosystem(
        num_nucleotides=args.nucleotides,
        num_voxels=args.voxels,
        enable_visualization=not args.no_visualization,
        enable_concept_search=not args.no_search
    )
    
    # Запуск
    try:
        asyncio.run(ecosystem.run())
    except KeyboardInterrupt:
        print("\n[Main] Прерывание пользователем")
    except Exception as e:
        print(f"\n[Main] Ошибка: {e}")
        import traceback
        traceback.print_exc()
    finally:
        print("\n[Main] Программа завершена.")


if __name__ == '__main__':
    main()

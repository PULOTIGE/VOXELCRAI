"""
Main Window - Главное окно приложения
=====================================

Основной интерфейс CrimeaAI Ecosystem.
"""

import sys
import time
import asyncio
import threading
from typing import Optional, Dict, Any, List
from dataclasses import dataclass

try:
    import pygame
    PYGAME_AVAILABLE = True
except ImportError:
    PYGAME_AVAILABLE = False
    print("⚠️ pygame не установлен. UI будет недоступен.")

from .theme import COLORS, SIZES, FONT_SIZES
from .visualizer import VoxelVisualizer, NucleotideVisualizer, KaifGraph
from .widgets import StatusPanel, ControlPanel, GraphWidget, EmotionWheel, Button

# Импортируем ядро
import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from core.nucleotide import Nucleotide, NucleotidePool
from core.voxel import Voxel, VoxelWorld
from core.light_pattern import LightPattern, PatternDatabase
from core.kaif_engine import KaifEngine, KaifState
from core.scheduler import CrimeaScheduler, TaskPriority
from core.concept_search import ConceptSearcher, ConceptIntegrator


@dataclass
class AppConfig:
    """Конфигурация приложения"""
    window_width: int = 1400
    window_height: int = 900
    target_fps: int = 60
    
    nucleotide_pool_size: int = 10000
    initial_voxels: int = 50
    
    auto_save_interval: float = 300.0  # 5 минут
    concept_search_interval: float = 1140.0  # 19 минут
    
    data_dir: str = "data"


class CrimeaAIApp:
    """
    Главное приложение CrimeaAI Ecosystem
    """
    
    def __init__(self, config: Optional[AppConfig] = None):
        """
        Инициализация приложения
        
        Args:
            config: конфигурация (или по умолчанию)
        """
        self.config = config or AppConfig()
        
        # Состояние
        self.running = False
        self.paused = False
        self.clock = None
        self.screen = None
        
        # Компоненты экосистемы
        self.nucleotide_pool: Optional[NucleotidePool] = None
        self.voxel_world: Optional[VoxelWorld] = None
        self.pattern_db: Optional[PatternDatabase] = None
        self.kaif_engine: Optional[KaifEngine] = None
        self.scheduler: Optional[CrimeaScheduler] = None
        self.concept_searcher: Optional[ConceptSearcher] = None
        self.concept_integrator: Optional[ConceptIntegrator] = None
        
        # UI компоненты
        self.status_panel: Optional[StatusPanel] = None
        self.control_panel: Optional[ControlPanel] = None
        self.voxel_viz: Optional[VoxelVisualizer] = None
        self.nucleotide_viz: Optional[NucleotideVisualizer] = None
        self.kaif_graph: Optional[KaifGraph] = None
        self.emotion_wheel: Optional[EmotionWheel] = None
        self.graphs: List[GraphWidget] = []
        
        # Статистика
        self.fps = 0.0
        self.frame_count = 0
        self.start_time = 0.0
        
        # Логи
        self.logs: List[str] = []
        self.max_logs = 20
    
    def log(self, message: str):
        """Добавление записи в лог"""
        timestamp = time.strftime("%H:%M:%S")
        self.logs.append(f"[{timestamp}] {message}")
        if len(self.logs) > self.max_logs:
            self.logs.pop(0)
        print(f"[{timestamp}] {message}")
    
    def initialize(self):
        """Инициализация всех компонентов"""
        self.log("🚀 Инициализация CrimeaAI Ecosystem...")
        
        # Создаём директорию данных
        os.makedirs(self.config.data_dir, exist_ok=True)
        
        # Инициализация ядра
        self._init_core()
        
        # Инициализация pygame
        if PYGAME_AVAILABLE:
            self._init_pygame()
            self._init_ui()
        
        # Настройка планировщика
        self._setup_scheduler()
        
        self.start_time = time.time()
        self.log("✅ Инициализация завершена!")
    
    def _init_core(self):
        """Инициализация ядра экосистемы"""
        # Пул нуклеотидов
        self.log(f"🧬 Создание пула нуклеотидов ({self.config.nucleotide_pool_size:,})...")
        self.nucleotide_pool = NucleotidePool(size=self.config.nucleotide_pool_size)
        self.nucleotide_pool.initialize(random_init=True)
        
        # Мир вокселей
        self.log(f"🌍 Создание мира вокселей...")
        self.voxel_world = VoxelWorld(max_voxels=1000)
        
        # Спавним начальные воксели
        import random
        for _ in range(self.config.initial_voxels):
            pos = (
                random.uniform(-20, 20),
                random.uniform(-20, 20),
                0
            )
            self.voxel_world.spawn_voxel(pos)
        
        # База паттернов освещения
        self.log("💡 Создание базы паттернов...")
        self.pattern_db = PatternDatabase(max_patterns=1000)
        self.pattern_db.generate_random_patterns(100)
        
        # Движок кайфа
        self.log("⚡ Инициализация KaifEngine...")
        self.kaif_engine = KaifEngine()
        self.kaif_engine.register_component('nucleotides', weight=0.3)
        self.kaif_engine.register_component('voxels', weight=0.5)
        self.kaif_engine.register_component('emotions', weight=0.2)
        
        # Поисковик концептов
        self.log("🔍 Инициализация поиска концептов...")
        self.concept_searcher = ConceptSearcher(
            base_keywords=['AI', 'neural network', 'machine learning', 'cognitive science']
        )
        self.concept_integrator = ConceptIntegrator()
    
    def _init_pygame(self):
        """Инициализация pygame"""
        pygame.init()
        pygame.font.init()
        
        # Создаём окно
        self.screen = pygame.display.set_mode(
            (self.config.window_width, self.config.window_height),
            pygame.RESIZABLE
        )
        pygame.display.set_caption("🧠 CrimeaAI Ecosystem v1.0")
        
        self.clock = pygame.time.Clock()
        
        self.log("🎮 Pygame инициализирован")
    
    def _init_ui(self):
        """Инициализация UI компонентов"""
        w = self.config.window_width
        h = self.config.window_height
        panel_width = SIZES['panel_width']
        
        # Визуализатор вокселей (центр)
        viz_width = w - panel_width * 2 - 40
        viz_height = h - 250
        self.voxel_viz = VoxelVisualizer(viz_width, viz_height)
        self.voxel_viz_surface = pygame.Surface((viz_width, viz_height))
        self.voxel_viz.set_surface(self.voxel_viz_surface)
        
        # Визуализатор нуклеотидов (правая панель)
        nuc_height = h - 300
        self.nucleotide_viz = NucleotideVisualizer(panel_width - 20, nuc_height)
        self.nucleotide_viz_surface = pygame.Surface((panel_width - 20, nuc_height))
        self.nucleotide_viz.set_surface(self.nucleotide_viz_surface)
        
        # Панель статуса (левая)
        self.status_panel = StatusPanel(
            10, 10, panel_width - 10, 250
        )
        
        # Панель управления (левая, под статусом)
        self.control_panel = ControlPanel(10, 270, panel_width - 10)
        self.control_panel.setup_buttons(
            on_start_stop=self._on_start_stop,
            on_reset=self._on_reset,
            on_save=self._on_save,
            on_load=self._on_load,
            on_search=self._on_search_concepts
        )
        
        # График кайфа
        self.kaif_graph = KaifGraph(300, 100)
        self.kaif_graph_surface = pygame.Surface((300, 100))
        self.kaif_graph.set_surface(self.kaif_graph_surface)
        
        # Колесо эмоций
        self.emotion_wheel = EmotionWheel(
            w - panel_width + panel_width // 2,
            h - 100,
            60
        )
        
        # Графики метрик
        graph_y = h - 200
        self.graphs = [
            GraphWidget(panel_width + 10, graph_y, 200, 80, "Health"),
            GraphWidget(panel_width + 220, graph_y, 200, 80, "Energy"),
            GraphWidget(panel_width + 430, graph_y, 200, 80, "Concepts"),
        ]
        self.graphs[0].line_color = COLORS['success']
        self.graphs[0].fill_color = (*COLORS['success'][:3], 50)
        self.graphs[1].line_color = COLORS['accent_yellow']
        self.graphs[1].fill_color = (*COLORS['accent_yellow'][:3], 50)
        self.graphs[2].line_color = COLORS['accent_purple']
        self.graphs[2].fill_color = (*COLORS['accent_purple'][:3], 50)
        
        self.log("🖼️ UI компоненты созданы")
    
    def _setup_scheduler(self):
        """Настройка планировщика"""
        self.scheduler = CrimeaScheduler()
        
        # Обновление нуклеотидов
        self.scheduler.add_task(
            name="nucleotide_update",
            callback=self._update_nucleotides,
            interval=0.016,
            priority=TaskPriority.HIGH
        )
        
        # Обновление вокселей
        self.scheduler.add_task(
            name="voxel_update",
            callback=self._update_voxels,
            interval=0.016,
            priority=TaskPriority.HIGH
        )
        
        # Обновление кайфа
        self.scheduler.add_task(
            name="kaif_update",
            callback=self._update_kaif,
            interval=0.05,
            priority=TaskPriority.NORMAL
        )
        
        # Автосохранение
        self.scheduler.add_task(
            name="auto_save",
            callback=self._auto_save,
            interval=self.config.auto_save_interval,
            priority=TaskPriority.BACKGROUND
        )
        
        self.log("📋 Планировщик настроен")
    
    def _update_nucleotides(self):
        """Обновление нуклеотидов"""
        if self.paused or self.nucleotide_pool is None:
            return
        
        # Обновляем пул
        self.nucleotide_pool.update_all(0.016)
        
        # Обновляем компонент кайфа
        if self.kaif_engine and self.nucleotide_pool.semantic_matrix is not None:
            sample = self.nucleotide_pool.semantic_matrix[:100].flatten()
            self.kaif_engine.update_component('nucleotides', sample)
    
    def _update_voxels(self):
        """Обновление вокселей"""
        if self.paused or self.voxel_world is None:
            return
        
        self.voxel_world.update(0.016)
        
        # Обновляем компонент кайфа
        if self.kaif_engine and self.voxel_world.voxels:
            # Собираем эмоциональные векторы
            import numpy as np
            emotions = []
            for voxel in list(self.voxel_world.voxels.values())[:50]:
                emotions.append(voxel.emotions.emotion_vector[:10])
            
            if emotions:
                combined = np.concatenate(emotions)
                self.kaif_engine.update_component('voxels', combined)
    
    def _update_kaif(self):
        """Обновление движка кайфа"""
        if self.kaif_engine:
            self.kaif_engine.update(0.05)
    
    def _auto_save(self):
        """Автосохранение"""
        self._on_save()
    
    def _on_start_stop(self, is_running: bool):
        """Обработка старта/паузы"""
        self.paused = not is_running
        state = "возобновлена" if is_running else "приостановлена"
        self.log(f"⏯️ Симуляция {state}")
    
    def _on_reset(self):
        """Сброс симуляции"""
        self.log("🔄 Сброс симуляции...")
        
        # Пересоздаём компоненты
        self.nucleotide_pool = NucleotidePool(size=self.config.nucleotide_pool_size)
        self.nucleotide_pool.initialize(random_init=True)
        
        self.voxel_world = VoxelWorld(max_voxels=1000)
        import random
        for _ in range(self.config.initial_voxels):
            pos = (random.uniform(-20, 20), random.uniform(-20, 20), 0)
            self.voxel_world.spawn_voxel(pos)
        
        self.log("✅ Сброс завершён")
    
    def _on_save(self):
        """Сохранение состояния"""
        self.log("💾 Сохранение состояния...")
        
        try:
            # Сохраняем концепты
            self.concept_searcher.save(f"{self.config.data_dir}/concepts.json")
            
            # Здесь можно добавить сохранение других компонентов
            
            self.log("✅ Сохранение завершено")
        except Exception as e:
            self.log(f"❌ Ошибка сохранения: {e}")
    
    def _on_load(self):
        """Загрузка состояния"""
        self.log("📂 Загрузка состояния...")
        
        try:
            concepts_path = f"{self.config.data_dir}/concepts.json"
            if os.path.exists(concepts_path):
                self.concept_searcher.load(concepts_path)
                self.log("✅ Загрузка завершена")
            else:
                self.log("⚠️ Файл сохранения не найден")
        except Exception as e:
            self.log(f"❌ Ошибка загрузки: {e}")
    
    def _on_search_concepts(self):
        """Поиск концептов"""
        self.log("🔍 Запуск поиска концептов...")
        
        # Запускаем в отдельном потоке
        def search_thread():
            concepts = self.concept_searcher.search_concepts()
            self.log(f"✅ Найдено {len(concepts)} концептов")
            
            # Интегрируем в воксели
            if self.voxel_world and concepts:
                for concept in concepts[:5]:
                    for voxel in list(self.voxel_world.voxels.values())[:10]:
                        self.concept_integrator.integrate_into_voxel(concept, voxel)
        
        thread = threading.Thread(target=search_thread, daemon=True)
        thread.start()
    
    def handle_events(self) -> bool:
        """Обработка событий pygame"""
        if not PYGAME_AVAILABLE:
            return True
        
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                return False
            
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_ESCAPE:
                    return False
                elif event.key == pygame.K_SPACE:
                    self.paused = not self.paused
                    self.control_panel.is_running = not self.paused
                    self.control_panel.buttons[0].text = "⏸ PAUSE" if not self.paused else "▶ START"
                elif event.key == pygame.K_PLUS or event.key == pygame.K_EQUALS:
                    self.voxel_viz.zoom_in()
                elif event.key == pygame.K_MINUS:
                    self.voxel_viz.zoom_out()
            
            elif event.type == pygame.MOUSEWHEEL:
                if event.y > 0:
                    self.voxel_viz.zoom_in()
                else:
                    self.voxel_viz.zoom_out()
            
            elif event.type == pygame.VIDEORESIZE:
                self.config.window_width = event.w
                self.config.window_height = event.h
                self._init_ui()
            
            # Обработка виджетов
            self.control_panel.handle_event(event)
        
        return True
    
    def update(self, dt: float):
        """Обновление логики"""
        # Обновляем UI
        if self.status_panel:
            self.status_panel.update(dt)
            self.status_panel.update_metrics(
                kaif=self.kaif_engine.get_kaif() if self.kaif_engine else 0,
                voxel_count=len(self.voxel_world.voxels) if self.voxel_world else 0,
                nucleotide_count=self.nucleotide_pool.size if self.nucleotide_pool else 0,
                fps=self.fps,
                concepts=len(self.concept_searcher.concepts) if self.concept_searcher else 0,
                avg_health=self.voxel_world.avg_health if self.voxel_world else 1.0,
                avg_energy=self.voxel_world.avg_energy if self.voxel_world else 1.0
            )
        
        if self.control_panel:
            self.control_panel.update(dt)
        
        if self.emotion_wheel and self.voxel_world:
            # Собираем эмоции
            emotion_dist = self.voxel_world._get_emotion_distribution()
            self.emotion_wheel.update_emotions(emotion_dist)
            self.emotion_wheel.update(dt)
        
        # Обновляем графики
        if self.kaif_graph and self.kaif_engine:
            self.kaif_graph.add_value(self.kaif_engine.get_kaif())
        
        if self.graphs and self.voxel_world:
            self.graphs[0].add_value(self.voxel_world.avg_health)
            self.graphs[1].add_value(self.voxel_world.avg_energy)
        
        if self.graphs and self.concept_searcher:
            self.graphs[2].add_value(len(self.concept_searcher.concepts) / 100)
        
        # Обновляем компоненты (если не на паузе)
        if not self.paused:
            self._update_nucleotides()
            self._update_voxels()
            self._update_kaif()
    
    def render(self):
        """Рендеринг"""
        if not PYGAME_AVAILABLE or self.screen is None:
            return
        
        # Очищаем экран
        self.screen.fill(COLORS['bg_primary'])
        
        w = self.config.window_width
        h = self.config.window_height
        panel_width = SIZES['panel_width']
        
        # Рендерим визуализатор вокселей
        if self.voxel_viz and self.voxel_world:
            self.voxel_viz.render(list(self.voxel_world.voxels.values()), 0.016)
            self.screen.blit(self.voxel_viz_surface, (panel_width + 10, 10))
        
        # Рамка визуализатора
        viz_rect = pygame.Rect(
            panel_width + 10, 10,
            w - panel_width * 2 - 30, h - 250
        )
        pygame.draw.rect(self.screen, COLORS['bg_highlight'], viz_rect, 2, border_radius=4)
        
        # Рендерим визуализатор нуклеотидов
        if self.nucleotide_viz and self.nucleotide_pool:
            nucleotides = self.nucleotide_pool.nucleotides[:100]
            self.nucleotide_viz.render(nucleotides, 0.016)
            self.screen.blit(self.nucleotide_viz_surface, (w - panel_width + 10, 10))
        
        # Рендерим панели
        if self.status_panel:
            self.status_panel.render(self.screen)
        
        if self.control_panel:
            self.control_panel.render(self.screen)
        
        # Рендерим график кайфа
        if self.kaif_graph:
            self.kaif_graph.render()
            self.screen.blit(self.kaif_graph_surface, (panel_width + 20, h - 240))
        
        # Рендерим колесо эмоций
        if self.emotion_wheel:
            self.emotion_wheel.render(self.screen)
        
        # Рендерим графики
        for graph in self.graphs:
            graph.render(self.screen)
        
        # Рендерим логи
        self._render_logs()
        
        # Обновляем экран
        pygame.display.flip()
    
    def _render_logs(self):
        """Рендеринг логов"""
        font = pygame.font.SysFont('monospace', 10)
        y = self.config.window_height - 30
        
        for log in reversed(self.logs[-5:]):
            text = font.render(log, True, COLORS['text_muted'])
            self.screen.blit(text, (10, y))
            y -= 14
    
    def run(self):
        """Главный цикл приложения"""
        self.initialize()
        self.running = True
        
        self.log("▶️ Запуск главного цикла...")
        
        try:
            while self.running:
                # Обработка событий
                if not self.handle_events():
                    break
                
                # Вычисляем dt
                dt = self.clock.tick(self.config.target_fps) / 1000.0 if self.clock else 0.016
                self.fps = self.clock.get_fps() if self.clock else 60
                self.frame_count += 1
                
                # Обновление
                self.update(dt)
                
                # Рендеринг
                self.render()
        
        except KeyboardInterrupt:
            self.log("⚠️ Прерывание пользователем")
        
        finally:
            self.shutdown()
    
    def shutdown(self):
        """Завершение работы"""
        self.log("🛑 Завершение работы...")
        
        # Сохраняем состояние
        self._on_save()
        
        # Останавливаем pygame
        if PYGAME_AVAILABLE:
            pygame.quit()
        
        self.log("👋 До свидания!")


def run_app(config: Optional[AppConfig] = None):
    """
    Запуск приложения
    
    Args:
        config: конфигурация
    """
    app = CrimeaAIApp(config)
    app.run()


if __name__ == "__main__":
    run_app()

"""
Модуль визуализации состояния экосистемы
"""

import pygame
import numpy as np
from typing import List, Dict, Tuple
import math


class EcosystemDisplay:
    """
    Визуализация состояния экосистемы с помощью Pygame.
    """
    
    def __init__(self, width: int = 1200, height: int = 800, title: str = "Cursor AI Ecosystem"):
        """
        Инициализация дисплея.
        
        Args:
            width: Ширина окна
            height: Высота окна
            title: Заголовок окна
        """
        pygame.init()
        
        self.width = width
        self.height = height
        self.screen = pygame.display.set_mode((width, height))
        pygame.display.set_caption(title)
        
        # Шрифты
        self.font_large = pygame.font.Font(None, 36)
        self.font_medium = pygame.font.Font(None, 24)
        self.font_small = pygame.font.Font(None, 18)
        
        # Цвета
        self.colors = {
            'background': (10, 10, 15),
            'text': (220, 220, 220),
            'accent': (100, 200, 255),
            'positive': (100, 255, 100),
            'negative': (255, 100, 100),
            'neutral': (200, 200, 100),
            'grid': (30, 30, 40),
            'voxel_alive': (50, 200, 50),
            'voxel_dead': (100, 50, 50),
        }
        
        # История данных для графиков
        self.kaif_history: List[float] = []
        self.entropy_history: List[float] = []
        self.energy_history: List[float] = []
        self.max_history_length = 200
        
        self.clock = pygame.time.Clock()
        self.running = True
        
    def handle_events(self) -> bool:
        """
        Обработка событий Pygame.
        
        Returns:
            True если окно должно продолжать работу, False для закрытия
        """
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                self.running = False
                return False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_ESCAPE:
                    self.running = False
                    return False
        
        return True
    
    def clear(self):
        """Очистка экрана."""
        self.screen.fill(self.colors['background'])
    
    def draw_text(self, text: str, x: int, y: int, font=None, color=None):
        """
        Отрисовка текста.
        
        Args:
            text: Текст для отрисовки
            x, y: Координаты
            font: Шрифт (по умолчанию medium)
            color: Цвет (по умолчанию text)
        """
        if font is None:
            font = self.font_medium
        if color is None:
            color = self.colors['text']
        
        text_surface = font.render(text, True, color)
        self.screen.blit(text_surface, (x, y))
    
    def draw_stats(self, stats: Dict, x: int = 20, y: int = 20):
        """
        Отрисовка статистики.
        
        Args:
            stats: Словарь со статистикой
            x, y: Координаты начала
        """
        self.draw_text("=== Cursor AI Ecosystem ===", x, y, self.font_large, self.colors['accent'])
        
        y_offset = y + 50
        line_height = 25
        
        for key, value in stats.items():
            if isinstance(value, float):
                text = f"{key}: {value:.3f}"
            elif isinstance(value, dict):
                # Рекурсивная отрисовка вложенных словарей
                self.draw_text(f"{key}:", x, y_offset, color=self.colors['accent'])
                y_offset += line_height
                for sub_key, sub_value in value.items():
                    if isinstance(sub_value, float):
                        text = f"  {sub_key}: {sub_value:.3f}"
                    else:
                        text = f"  {sub_key}: {sub_value}"
                    self.draw_text(text, x + 10, y_offset, self.font_small)
                    y_offset += line_height - 5
                continue
            else:
                text = f"{key}: {value}"
            
            self.draw_text(text, x, y_offset)
            y_offset += line_height
    
    def draw_voxel_grid(self, voxels, grid_x: int = 700, grid_y: int = 50, 
                       grid_size: int = 400, cols: int = 20):
        """
        Отрисовка сетки вокселей.
        
        Args:
            voxels: Список вокселей
            grid_x, grid_y: Координаты сетки
            grid_size: Размер сетки в пикселях
            cols: Количество столбцов
        """
        cell_size = grid_size // cols
        rows = (len(voxels) + cols - 1) // cols
        
        # Заголовок
        self.draw_text("Voxel Grid", grid_x, grid_y - 30, self.font_medium, self.colors['accent'])
        
        for i, voxel in enumerate(voxels[:cols * rows]):
            row = i // cols
            col = i % cols
            
            x = grid_x + col * cell_size
            y = grid_y + row * cell_size
            
            # Определение цвета на основе состояния
            if voxel.metadata['alive']:
                # Интенсивность зависит от "кайфа"
                intensity = min(255, int(voxel.kaif * 500))
                color = (50, 100 + intensity, 50)
            else:
                color = self.colors['voxel_dead']
            
            # Отрисовка ячейки
            pygame.draw.rect(self.screen, color, (x, y, cell_size - 2, cell_size - 2))
            
            # Рамка
            pygame.draw.rect(self.screen, self.colors['grid'], 
                           (x, y, cell_size - 2, cell_size - 2), 1)
    
    def draw_line_chart(self, data: List[float], x: int, y: int, 
                       width: int, height: int, title: str, 
                       color: Tuple[int, int, int] = None):
        """
        Отрисовка линейного графика.
        
        Args:
            data: Данные для графика
            x, y: Координаты графика
            width, height: Размеры графика
            title: Заголовок
            color: Цвет линии
        """
        if color is None:
            color = self.colors['accent']
        
        # Заголовок
        self.draw_text(title, x, y - 25, self.font_small, self.colors['accent'])
        
        # Рамка
        pygame.draw.rect(self.screen, self.colors['grid'], (x, y, width, height), 2)
        
        if len(data) < 2:
            return
        
        # Нормализация данных
        min_val = min(data)
        max_val = max(data)
        range_val = max_val - min_val if max_val != min_val else 1.0
        
        # Отрисовка линии
        points = []
        for i, value in enumerate(data):
            px = x + (i / len(data)) * width
            normalized = (value - min_val) / range_val
            py = y + height - (normalized * height)
            points.append((px, py))
        
        if len(points) >= 2:
            pygame.draw.lines(self.screen, color, False, points, 2)
        
        # Отрисовка значений min/max
        self.draw_text(f"Max: {max_val:.3f}", x + width - 100, y + 5, 
                      self.font_small, self.colors['text'])
        self.draw_text(f"Min: {min_val:.3f}", x + width - 100, y + height - 20, 
                      self.font_small, self.colors['text'])
    
    def update_history(self, kaif: float, entropy: float, energy: float):
        """
        Обновление истории данных.
        
        Args:
            kaif: Значение "кайфа"
            entropy: Энтропия
            energy: Энергия
        """
        self.kaif_history.append(kaif)
        self.entropy_history.append(entropy)
        self.energy_history.append(energy)
        
        # Ограничение длины истории
        if len(self.kaif_history) > self.max_history_length:
            self.kaif_history.pop(0)
        if len(self.entropy_history) > self.max_history_length:
            self.entropy_history.pop(0)
        if len(self.energy_history) > self.max_history_length:
            self.energy_history.pop(0)
    
    def draw_full_dashboard(self, ecosystem_state: Dict):
        """
        Отрисовка полной панели управления.
        
        Args:
            ecosystem_state: Состояние экосистемы
        """
        self.clear()
        
        # Левая панель: статистика
        stats = {
            'Uptime': ecosystem_state.get('uptime', 0.0),
            'FPS': ecosystem_state.get('fps', 0.0),
            'Voxels Alive': ecosystem_state.get('voxels_alive', 0),
            'Avg Kaif': ecosystem_state.get('avg_kaif', 0.0),
            'Avg Energy': ecosystem_state.get('avg_energy', 0.0),
            'Concepts Found': ecosystem_state.get('concepts_found', 0),
        }
        self.draw_stats(stats, 20, 20)
        
        # Правая панель: сетка вокселей
        voxels = ecosystem_state.get('voxels', [])
        if voxels:
            self.draw_voxel_grid(voxels, 700, 50, 450, 20)
        
        # Нижняя панель: графики
        if self.kaif_history:
            self.draw_line_chart(self.kaif_history, 20, 450, 300, 150, 
                               "Kaif History", self.colors['positive'])
        
        if self.entropy_history:
            self.draw_line_chart(self.entropy_history, 350, 450, 300, 150, 
                               "Entropy History", self.colors['neutral'])
        
        if self.energy_history:
            self.draw_line_chart(self.energy_history, 20, 630, 300, 150, 
                               "Energy History", self.colors['accent'])
        
        # Информация о концептах
        concepts = ecosystem_state.get('recent_concepts', [])
        if concepts:
            self.draw_text("Recent Concepts:", 350, 630, self.font_small, self.colors['accent'])
            for i, concept in enumerate(concepts[:10]):
                self.draw_text(f"• {concept}", 360, 655 + i * 18, self.font_small)
    
    def update(self):
        """Обновление дисплея."""
        pygame.display.flip()
        self.clock.tick(60)  # 60 FPS
    
    def close(self):
        """Закрытие дисплея."""
        pygame.quit()
    
    def __del__(self):
        """Деструктор."""
        try:
            pygame.quit()
        except:
            pass

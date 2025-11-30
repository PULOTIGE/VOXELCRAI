"""
Visualizers - Визуализация вокселей и нуклеотидов
=================================================
"""

import math
import time
from typing import List, Optional, Tuple, Dict
import numpy as np

try:
    import pygame
    PYGAME_AVAILABLE = True
except ImportError:
    PYGAME_AVAILABLE = False

from .theme import COLORS, get_kaif_color, get_emotion_color, lerp_color, add_glow


class VoxelVisualizer:
    """
    Визуализатор вокселей
    
    Отображает воксели как светящиеся точки с цветом,
    зависящим от их эмоционального состояния.
    """
    
    def __init__(self, width: int = 600, height: int = 400):
        """
        Создание визуализатора
        
        Args:
            width: ширина области
            height: высота области
        """
        self.width = width
        self.height = height
        
        # Параметры отображения
        self.zoom = 10.0
        self.offset_x = width // 2
        self.offset_y = height // 2
        
        # Анимация
        self.time = 0.0
        self.pulse_phase = 0.0
        
        # Следы (trails)
        self.trails: Dict[int, List[Tuple[float, float]]] = {}
        self.trail_length = 20
        
        # Поверхность для рендеринга
        self._surface: Optional[pygame.Surface] = None
    
    def set_surface(self, surface: pygame.Surface):
        """Установка поверхности для рендеринга"""
        self._surface = surface
        self.width = surface.get_width()
        self.height = surface.get_height()
        self.offset_x = self.width // 2
        self.offset_y = self.height // 2
    
    def world_to_screen(self, x: float, y: float) -> Tuple[int, int]:
        """Преобразование мировых координат в экранные"""
        screen_x = int(self.offset_x + x * self.zoom)
        screen_y = int(self.offset_y - y * self.zoom)  # Y инвертирован
        return screen_x, screen_y
    
    def render(self, voxels: List, dt: float = 0.016):
        """
        Рендеринг вокселей
        
        Args:
            voxels: список вокселей для отображения
            dt: delta time
        """
        if not PYGAME_AVAILABLE or self._surface is None:
            return
        
        self.time += dt
        self.pulse_phase = (self.pulse_phase + dt * 2) % (2 * math.pi)
        
        # Очищаем фон с эффектом затухания
        overlay = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        overlay.fill((*COLORS['bg_primary'], 40))
        self._surface.blit(overlay, (0, 0))
        
        # Рисуем сетку
        self._draw_grid()
        
        # Рисуем воксели
        for voxel in voxels:
            self._draw_voxel(voxel)
        
        # Рисуем информационный оверлей
        self._draw_info_overlay(len(voxels))
    
    def _draw_grid(self):
        """Рисование фоновой сетки"""
        grid_color = (*COLORS['bg_highlight'], 30)
        grid_spacing = int(50 * self.zoom / 10)
        
        # Вертикальные линии
        for x in range(0, self.width, grid_spacing):
            pygame.draw.line(
                self._surface, grid_color,
                (x, 0), (x, self.height), 1
            )
        
        # Горизонтальные линии
        for y in range(0, self.height, grid_spacing):
            pygame.draw.line(
                self._surface, grid_color,
                (0, y), (self.width, y), 1
            )
    
    def _draw_voxel(self, voxel):
        """Рисование одного вокселя"""
        # Позиция на экране
        pos = voxel.metadata.position
        screen_x, screen_y = self.world_to_screen(pos[0], pos[1])
        
        # Проверяем видимость
        if not (-50 < screen_x < self.width + 50 and -50 < screen_y < self.height + 50):
            return
        
        # Обновляем след
        vid = voxel.metadata.voxel_id
        if vid not in self.trails:
            self.trails[vid] = []
        self.trails[vid].append((screen_x, screen_y))
        if len(self.trails[vid]) > self.trail_length:
            self.trails[vid].pop(0)
        
        # Рисуем след
        self._draw_trail(self.trails[vid], voxel)
        
        # Получаем цвет на основе эмоций
        dom_emotion, dom_value = voxel.emotions.get_dominant_emotion()
        base_color = get_emotion_color(dom_emotion.value)
        
        # Модифицируем яркость на основе кайфа
        kaif = voxel.emotions.kaif
        brightness = 0.5 + kaif * 0.5
        color = tuple(int(c * brightness) for c in base_color)
        
        # Размер зависит от здоровья и энергии
        base_size = 8
        size = int(base_size * (0.5 + voxel.metadata.health * 0.3 + voxel.metadata.energy * 0.2))
        
        # Пульсация при высоком кайфе
        if kaif > 0.5:
            pulse = math.sin(self.pulse_phase * 3 + vid) * 0.2 * kaif
            size = int(size * (1 + pulse))
        
        # Рисуем свечение
        glow_color = add_glow(color, 0.3)
        for i in range(3):
            glow_size = size + (3 - i) * 4
            alpha = 50 - i * 15
            glow_surface = pygame.Surface((glow_size * 2, glow_size * 2), pygame.SRCALPHA)
            pygame.draw.circle(
                glow_surface,
                (*glow_color, alpha),
                (glow_size, glow_size),
                glow_size
            )
            self._surface.blit(
                glow_surface,
                (screen_x - glow_size, screen_y - glow_size)
            )
        
        # Рисуем основной круг
        pygame.draw.circle(self._surface, color, (screen_x, screen_y), size)
        
        # Внутренний круг (ядро)
        core_color = add_glow(color, 0.5)
        pygame.draw.circle(self._surface, core_color, (screen_x, screen_y), size // 2)
    
    def _draw_trail(self, trail: List[Tuple[float, float]], voxel):
        """Рисование следа вокселя"""
        if len(trail) < 2:
            return
        
        # Цвет следа
        dom_emotion, _ = voxel.emotions.get_dominant_emotion()
        trail_color = get_emotion_color(dom_emotion.value)
        
        # Рисуем сегменты с затуханием
        for i in range(1, len(trail)):
            alpha = int(100 * (i / len(trail)))
            thickness = max(1, int(3 * (i / len(trail))))
            
            segment_color = (*trail_color, alpha)
            
            start = (int(trail[i-1][0]), int(trail[i-1][1]))
            end = (int(trail[i][0]), int(trail[i][1]))
            
            # Используем anti-aliased линию
            pygame.draw.line(
                self._surface,
                trail_color[:3],
                start, end,
                thickness
            )
    
    def _draw_info_overlay(self, voxel_count: int):
        """Рисование информационного оверлея"""
        font = pygame.font.SysFont('monospace', 12)
        
        info_text = f"Voxels: {voxel_count} | Zoom: {self.zoom:.1f}x"
        text_surface = font.render(info_text, True, COLORS['text_muted'])
        
        self._surface.blit(text_surface, (10, 10))
    
    def zoom_in(self, factor: float = 1.2):
        """Увеличение масштаба"""
        self.zoom = min(50.0, self.zoom * factor)
    
    def zoom_out(self, factor: float = 1.2):
        """Уменьшение масштаба"""
        self.zoom = max(1.0, self.zoom / factor)
    
    def pan(self, dx: int, dy: int):
        """Смещение вида"""
        self.offset_x += dx
        self.offset_y += dy


class NucleotideVisualizer:
    """
    Визуализатор нуклеотидов
    
    Отображает нуклеотиды как ДНК-подобную структуру.
    """
    
    def __init__(self, width: int = 400, height: int = 600):
        """
        Создание визуализатора
        
        Args:
            width: ширина области
            height: высота области
        """
        self.width = width
        self.height = height
        
        # Параметры спирали
        self.helix_radius = 60
        self.helix_pitch = 30
        self.scroll_offset = 0.0
        
        # Цвета нуклеотидов
        self.nucleotide_colors = {
            'A': COLORS['accent_cyan'],     # Аденин
            'T': COLORS['accent_magenta'],  # Тимин
            'G': COLORS['accent_green'],    # Гуанин
            'C': COLORS['accent_yellow'],   # Цитозин
        }
        
        # Поверхность
        self._surface: Optional[pygame.Surface] = None
        
        # Анимация
        self.time = 0.0
        self.rotation = 0.0
    
    def set_surface(self, surface: pygame.Surface):
        """Установка поверхности"""
        self._surface = surface
        self.width = surface.get_width()
        self.height = surface.get_height()
    
    def render(self, nucleotides: List, dt: float = 0.016, display_range: int = 50):
        """
        Рендеринг нуклеотидов
        
        Args:
            nucleotides: список нуклеотидов
            dt: delta time
            display_range: количество отображаемых нуклеотидов
        """
        if not PYGAME_AVAILABLE or self._surface is None:
            return
        
        self.time += dt
        self.rotation += dt * 0.5
        
        # Очищаем
        self._surface.fill(COLORS['bg_secondary'])
        
        # Центр экрана
        center_x = self.width // 2
        center_y = self.height // 2
        
        # Отображаем спиральную структуру
        start_idx = max(0, int(self.scroll_offset))
        end_idx = min(len(nucleotides), start_idx + display_range)
        
        if end_idx <= start_idx:
            return
        
        # Рисуем связи (backbone)
        self._draw_backbone(nucleotides, start_idx, end_idx, center_x)
        
        # Рисуем нуклеотиды
        for i in range(start_idx, end_idx):
            local_idx = i - start_idx
            self._draw_nucleotide(
                nucleotides[i], local_idx,
                center_x, center_y, display_range
            )
        
        # Информация
        self._draw_info(len(nucleotides), start_idx, end_idx)
    
    def _draw_backbone(
        self,
        nucleotides: List,
        start: int,
        end: int,
        center_x: int
    ):
        """Рисование остова ДНК"""
        points_left = []
        points_right = []
        
        for i in range(start, end):
            local_idx = i - start
            t = local_idx / max(1, (end - start - 1))
            y = int(50 + t * (self.height - 100))
            
            # Позиции на спирали
            angle = self.rotation + local_idx * 0.3
            
            left_x = center_x - int(self.helix_radius * math.cos(angle))
            right_x = center_x + int(self.helix_radius * math.cos(angle))
            
            points_left.append((left_x, y))
            points_right.append((right_x, y))
        
        # Рисуем остовы
        if len(points_left) > 1:
            pygame.draw.lines(
                self._surface,
                COLORS['accent_cyan'],
                False,
                points_left,
                2
            )
            pygame.draw.lines(
                self._surface,
                COLORS['accent_magenta'],
                False,
                points_right,
                2
            )
    
    def _draw_nucleotide(
        self,
        nucleotide,
        local_idx: int,
        center_x: int,
        center_y: int,
        total: int
    ):
        """Рисование одного нуклеотида"""
        # Позиция на спирали
        t = local_idx / max(1, total - 1)
        y = int(50 + t * (self.height - 100))
        
        angle = self.rotation + local_idx * 0.3
        depth = math.sin(angle)  # Для 3D эффекта
        
        # Позиции пары оснований
        left_x = center_x - int(self.helix_radius * math.cos(angle))
        right_x = center_x + int(self.helix_radius * math.cos(angle))
        
        # Получаем цвет
        base = nucleotide.base.value
        color = self.nucleotide_colors.get(base, COLORS['text_secondary'])
        
        # Размер зависит от глубины (3D эффект)
        base_size = 8
        size = int(base_size * (0.6 + 0.4 * (depth + 1) / 2))
        
        # Модификация цвета на основе энергии
        energy = nucleotide.energy
        modified_color = tuple(int(c * (0.5 + energy * 0.5)) for c in color)
        
        # Рисуем соединительную линию (водородные связи)
        line_alpha = int(100 + 100 * (depth + 1) / 2)
        pygame.draw.line(
            self._surface,
            (*COLORS['text_muted'][:3],),
            (left_x, y),
            (right_x, y),
            1
        )
        
        # Рисуем основания
        # Левое основание (с глубиной)
        if depth > 0:
            pygame.draw.circle(self._surface, modified_color, (left_x, y), size)
            pygame.draw.circle(self._surface, add_glow(modified_color), (left_x, y), size // 2)
        
        # Правое основание (комплементарное)
        complement_map = {'A': 'T', 'T': 'A', 'G': 'C', 'C': 'G'}
        complement = complement_map.get(base, 'A')
        comp_color = self.nucleotide_colors.get(complement, COLORS['text_secondary'])
        comp_modified = tuple(int(c * (0.5 + energy * 0.5)) for c in comp_color)
        
        if depth < 0:
            pygame.draw.circle(self._surface, comp_modified, (right_x, y), size)
            pygame.draw.circle(self._surface, add_glow(comp_modified), (right_x, y), size // 2)
        
        # Эпигенетические метки (маленькие точки)
        if nucleotide.epigenetic_tags:
            tag_y = y - size - 3
            for i, (tag, strength) in enumerate(list(nucleotide.epigenetic_tags.items())[:3]):
                tag_x = left_x + i * 5 - 5
                tag_color = COLORS['accent_purple']
                tag_size = max(2, int(3 * strength))
                pygame.draw.circle(self._surface, tag_color, (tag_x, tag_y), tag_size)
    
    def _draw_info(self, total: int, start: int, end: int):
        """Рисование информации"""
        font = pygame.font.SysFont('monospace', 12)
        
        info_text = f"Nucleotides: {start}-{end} of {total}"
        text_surface = font.render(info_text, True, COLORS['text_muted'])
        
        self._surface.blit(text_surface, (10, self.height - 25))
        
        # Легенда
        legend_y = 10
        for base, color in self.nucleotide_colors.items():
            pygame.draw.circle(self._surface, color, (10, legend_y), 5)
            label = font.render(base, True, COLORS['text_secondary'])
            self._surface.blit(label, (20, legend_y - 6))
            legend_y += 20
    
    def scroll(self, delta: float):
        """Прокрутка"""
        self.scroll_offset = max(0, self.scroll_offset + delta)


class KaifGraph:
    """
    График кайфа в реальном времени
    """
    
    def __init__(self, width: int = 300, height: int = 150):
        self.width = width
        self.height = height
        
        self.history: List[float] = []
        self.max_history = 200
        
        self._surface: Optional[pygame.Surface] = None
    
    def set_surface(self, surface: pygame.Surface):
        self._surface = surface
        self.width = surface.get_width()
        self.height = surface.get_height()
    
    def add_value(self, kaif: float):
        """Добавление значения"""
        self.history.append(kaif)
        if len(self.history) > self.max_history:
            self.history.pop(0)
    
    def render(self):
        """Рендеринг графика"""
        if not PYGAME_AVAILABLE or self._surface is None:
            return
        
        # Фон
        self._surface.fill(COLORS['bg_tertiary'])
        
        # Рамка
        pygame.draw.rect(
            self._surface,
            COLORS['bg_highlight'],
            (0, 0, self.width, self.height),
            1
        )
        
        if len(self.history) < 2:
            return
        
        # Горизонтальные линии (уровни)
        for level in [0.25, 0.5, 0.75]:
            y = int(self.height * (1 - level))
            pygame.draw.line(
                self._surface,
                (*COLORS['bg_highlight'], 100),
                (0, y), (self.width, y),
                1
            )
        
        # Рисуем график
        points = []
        for i, value in enumerate(self.history):
            x = int(i / len(self.history) * self.width)
            y = int(self.height * (1 - value))
            y = max(0, min(self.height - 1, y))
            points.append((x, y))
        
        if len(points) > 1:
            # Заливка под графиком
            fill_points = [(0, self.height)] + points + [(self.width, self.height)]
            
            # Градиентная заливка (упрощённая)
            pygame.draw.polygon(
                self._surface,
                (*COLORS['accent_cyan'][:3], 50),
                fill_points
            )
            
            # Линия графика
            pygame.draw.lines(
                self._surface,
                COLORS['accent_cyan'],
                False,
                points,
                2
            )
        
        # Текущее значение
        if self.history:
            current = self.history[-1]
            font = pygame.font.SysFont('monospace', 14)
            
            # Цвет зависит от уровня
            color = get_kaif_color(current)
            
            text = font.render(f"KAIF: {current:.3f}", True, color)
            self._surface.blit(text, (5, 5))

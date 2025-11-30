"""
UI Widgets - Виджеты интерфейса
===============================

Переиспользуемые компоненты UI.
"""

import math
from typing import Optional, Callable, List, Dict, Any
from dataclasses import dataclass

try:
    import pygame
    PYGAME_AVAILABLE = True
except ImportError:
    PYGAME_AVAILABLE = False

from .theme import COLORS, SIZES, FONT_SIZES, get_kaif_color, lerp_color


class Button:
    """Стилизованная кнопка"""
    
    def __init__(
        self,
        x: int,
        y: int,
        width: int,
        height: int,
        text: str,
        on_click: Optional[Callable] = None,
        icon: Optional[str] = None
    ):
        self.rect = pygame.Rect(x, y, width, height) if PYGAME_AVAILABLE else None
        self.text = text
        self.on_click = on_click
        self.icon = icon
        
        self.hovered = False
        self.pressed = False
        self.enabled = True
        
        # Анимация
        self.hover_progress = 0.0
        self.press_progress = 0.0
    
    def update(self, dt: float):
        """Обновление анимаций"""
        target_hover = 1.0 if self.hovered else 0.0
        self.hover_progress += (target_hover - self.hover_progress) * 10 * dt
        
        target_press = 1.0 if self.pressed else 0.0
        self.press_progress += (target_press - self.press_progress) * 15 * dt
    
    def handle_event(self, event) -> bool:
        """Обработка событий"""
        if not PYGAME_AVAILABLE or not self.enabled:
            return False
        
        if event.type == pygame.MOUSEMOTION:
            self.hovered = self.rect.collidepoint(event.pos)
        
        elif event.type == pygame.MOUSEBUTTONDOWN:
            if event.button == 1 and self.hovered:
                self.pressed = True
                return True
        
        elif event.type == pygame.MOUSEBUTTONUP:
            if event.button == 1 and self.pressed:
                self.pressed = False
                if self.hovered and self.on_click:
                    self.on_click()
                    return True
        
        return False
    
    def render(self, surface: pygame.Surface):
        """Рендеринг кнопки"""
        if not PYGAME_AVAILABLE:
            return
        
        # Цвет фона
        if not self.enabled:
            bg_color = COLORS['bg_tertiary']
        else:
            bg_color = lerp_color(
                COLORS['bg_tertiary'],
                COLORS['bg_highlight'],
                self.hover_progress
            )
        
        # Эффект нажатия
        if self.press_progress > 0.1:
            bg_color = lerp_color(bg_color, COLORS['accent_cyan'], self.press_progress * 0.3)
        
        # Рисуем фон
        pygame.draw.rect(
            surface,
            bg_color,
            self.rect,
            border_radius=SIZES['border_radius']
        )
        
        # Рамка
        border_color = COLORS['accent_cyan'] if self.hovered else COLORS['bg_highlight']
        pygame.draw.rect(
            surface,
            border_color,
            self.rect,
            SIZES['border_width'],
            border_radius=SIZES['border_radius']
        )
        
        # Текст
        font = pygame.font.SysFont('monospace', FONT_SIZES['body'])
        text_color = COLORS['text_primary'] if self.enabled else COLORS['text_muted']
        text_surface = font.render(self.text, True, text_color)
        text_rect = text_surface.get_rect(center=self.rect.center)
        surface.blit(text_surface, text_rect)


class StatusPanel:
    """
    Панель статуса системы
    
    Отображает ключевые метрики экосистемы.
    """
    
    def __init__(self, x: int, y: int, width: int, height: int):
        self.x = x
        self.y = y
        self.width = width
        self.height = height
        
        self.metrics: Dict[str, Any] = {
            'kaif': 0.0,
            'voxel_count': 0,
            'nucleotide_count': 0,
            'fps': 0.0,
            'uptime': 0.0,
            'concepts': 0,
            'avg_health': 1.0,
            'avg_energy': 1.0
        }
        
        # Анимация значений
        self._animated_values: Dict[str, float] = {}
    
    def update_metrics(self, **kwargs):
        """Обновление метрик"""
        for key, value in kwargs.items():
            if key in self.metrics:
                self.metrics[key] = value
    
    def update(self, dt: float):
        """Обновление анимаций"""
        for key, target in self.metrics.items():
            if isinstance(target, (int, float)):
                current = self._animated_values.get(key, float(target))
                self._animated_values[key] = current + (float(target) - current) * 5 * dt
    
    def render(self, surface: pygame.Surface):
        """Рендеринг панели"""
        if not PYGAME_AVAILABLE:
            return
        
        # Фон панели
        panel_rect = pygame.Rect(self.x, self.y, self.width, self.height)
        pygame.draw.rect(
            surface,
            COLORS['bg_tertiary'],
            panel_rect,
            border_radius=SIZES['border_radius']
        )
        
        # Заголовок
        title_font = pygame.font.SysFont('monospace', FONT_SIZES['h3'], bold=True)
        title = title_font.render("⚡ SYSTEM STATUS", True, COLORS['accent_cyan'])
        surface.blit(title, (self.x + SIZES['padding_md'], self.y + SIZES['padding_md']))
        
        # Метрики
        y_offset = self.y + 45
        label_font = pygame.font.SysFont('monospace', FONT_SIZES['small'])
        value_font = pygame.font.SysFont('monospace', FONT_SIZES['body'], bold=True)
        
        metrics_display = [
            ('KAIF', self._animated_values.get('kaif', 0.0), '{:.3f}', get_kaif_color),
            ('Voxels', self._animated_values.get('voxel_count', 0), '{:.0f}', None),
            ('Nucleotides', self._animated_values.get('nucleotide_count', 0), '{:.0f}', None),
            ('FPS', self._animated_values.get('fps', 0.0), '{:.1f}', None),
            ('Health', self._animated_values.get('avg_health', 1.0), '{:.0%}', None),
            ('Energy', self._animated_values.get('avg_energy', 1.0), '{:.0%}', None),
            ('Concepts', self._animated_values.get('concepts', 0), '{:.0f}', None),
        ]
        
        for label, value, fmt, color_func in metrics_display:
            # Лейбл
            label_surface = label_font.render(label, True, COLORS['text_muted'])
            surface.blit(label_surface, (self.x + SIZES['padding_md'], y_offset))
            
            # Значение
            value_str = fmt.format(value) if isinstance(value, (int, float)) else str(value)
            value_color = color_func(value) if color_func else COLORS['text_primary']
            value_surface = value_font.render(value_str, True, value_color)
            surface.blit(
                value_surface,
                (self.x + self.width - SIZES['padding_md'] - value_surface.get_width(), y_offset)
            )
            
            y_offset += 25
        
        # Индикатор состояния
        self._draw_status_indicator(surface, y_offset + 10)
    
    def _draw_status_indicator(self, surface: pygame.Surface, y: int):
        """Рисование индикатора состояния"""
        kaif = self._animated_values.get('kaif', 0.0)
        
        # Бар кайфа
        bar_x = self.x + SIZES['padding_md']
        bar_width = self.width - 2 * SIZES['padding_md']
        bar_height = 8
        
        # Фон бара
        pygame.draw.rect(
            surface,
            COLORS['bg_highlight'],
            (bar_x, y, bar_width, bar_height),
            border_radius=4
        )
        
        # Заполнение
        fill_width = int(bar_width * min(1.0, kaif))
        if fill_width > 0:
            fill_color = get_kaif_color(kaif)
            pygame.draw.rect(
                surface,
                fill_color,
                (bar_x, y, fill_width, bar_height),
                border_radius=4
            )


class ControlPanel:
    """
    Панель управления
    
    Кнопки для управления симуляцией.
    """
    
    def __init__(self, x: int, y: int, width: int):
        self.x = x
        self.y = y
        self.width = width
        
        self.buttons: List[Button] = []
        self.is_running = True
        
        # Callbacks
        self._callbacks: Dict[str, Callable] = {}
    
    def setup_buttons(
        self,
        on_start_stop: Optional[Callable] = None,
        on_reset: Optional[Callable] = None,
        on_save: Optional[Callable] = None,
        on_load: Optional[Callable] = None,
        on_search: Optional[Callable] = None
    ):
        """Настройка кнопок"""
        if not PYGAME_AVAILABLE:
            return
        
        button_height = SIZES['button_height']
        padding = SIZES['padding_sm']
        btn_y = self.y
        
        # Кнопка старт/стоп
        self.buttons.append(Button(
            self.x, btn_y, self.width, button_height,
            "⏸ PAUSE" if self.is_running else "▶ START",
            on_click=self._toggle_run
        ))
        self._callbacks['start_stop'] = on_start_stop
        btn_y += button_height + padding
        
        # Кнопка сброса
        self.buttons.append(Button(
            self.x, btn_y, self.width, button_height,
            "🔄 RESET",
            on_click=on_reset
        ))
        btn_y += button_height + padding
        
        # Кнопка сохранения
        self.buttons.append(Button(
            self.x, btn_y, (self.width - padding) // 2, button_height,
            "💾 SAVE",
            on_click=on_save
        ))
        
        # Кнопка загрузки
        self.buttons.append(Button(
            self.x + (self.width + padding) // 2, btn_y, 
            (self.width - padding) // 2, button_height,
            "📂 LOAD",
            on_click=on_load
        ))
        btn_y += button_height + padding
        
        # Кнопка поиска концептов
        self.buttons.append(Button(
            self.x, btn_y, self.width, button_height,
            "🔍 SEARCH CONCEPTS",
            on_click=on_search
        ))
    
    def _toggle_run(self):
        """Переключение запуска/паузы"""
        self.is_running = not self.is_running
        
        # Обновляем текст кнопки
        if self.buttons:
            self.buttons[0].text = "⏸ PAUSE" if self.is_running else "▶ START"
        
        if self._callbacks.get('start_stop'):
            self._callbacks['start_stop'](self.is_running)
    
    def handle_event(self, event) -> bool:
        """Обработка событий"""
        for button in self.buttons:
            if button.handle_event(event):
                return True
        return False
    
    def update(self, dt: float):
        """Обновление"""
        for button in self.buttons:
            button.update(dt)
    
    def render(self, surface: pygame.Surface):
        """Рендеринг панели"""
        if not PYGAME_AVAILABLE:
            return
        
        for button in self.buttons:
            button.render(surface)


class GraphWidget:
    """
    Виджет графика с историей
    """
    
    def __init__(
        self,
        x: int,
        y: int,
        width: int,
        height: int,
        title: str = "Graph",
        max_points: int = 200
    ):
        self.x = x
        self.y = y
        self.width = width
        self.height = height
        self.title = title
        self.max_points = max_points
        
        self.values: List[float] = []
        self.min_val = 0.0
        self.max_val = 1.0
        self.auto_scale = True
        
        # Цвет линии
        self.line_color = COLORS['accent_cyan']
        self.fill_color = (*COLORS['accent_cyan'][:3], 50)
    
    def add_value(self, value: float):
        """Добавление значения"""
        self.values.append(value)
        if len(self.values) > self.max_points:
            self.values.pop(0)
        
        # Автомасштабирование
        if self.auto_scale and self.values:
            self.min_val = min(0, min(self.values))
            self.max_val = max(1, max(self.values))
    
    def render(self, surface: pygame.Surface):
        """Рендеринг графика"""
        if not PYGAME_AVAILABLE:
            return
        
        rect = pygame.Rect(self.x, self.y, self.width, self.height)
        
        # Фон
        pygame.draw.rect(
            surface,
            COLORS['bg_tertiary'],
            rect,
            border_radius=SIZES['border_radius']
        )
        
        # Заголовок
        font = pygame.font.SysFont('monospace', FONT_SIZES['small'])
        title_surface = font.render(self.title, True, COLORS['text_muted'])
        surface.blit(title_surface, (self.x + SIZES['padding_sm'], self.y + SIZES['padding_sm']))
        
        # Область графика
        graph_y = self.y + 25
        graph_height = self.height - 35
        graph_width = self.width - 2 * SIZES['padding_sm']
        graph_x = self.x + SIZES['padding_sm']
        
        if len(self.values) < 2:
            return
        
        # Строим точки
        val_range = self.max_val - self.min_val
        if val_range < 0.001:
            val_range = 1.0
        
        points = []
        for i, val in enumerate(self.values):
            px = graph_x + int(i / (len(self.values) - 1) * graph_width)
            py = graph_y + graph_height - int((val - self.min_val) / val_range * graph_height)
            py = max(graph_y, min(graph_y + graph_height, py))
            points.append((px, py))
        
        # Заливка
        fill_points = [
            (graph_x, graph_y + graph_height)
        ] + points + [
            (graph_x + graph_width, graph_y + graph_height)
        ]
        pygame.draw.polygon(surface, self.fill_color, fill_points)
        
        # Линия
        pygame.draw.lines(surface, self.line_color, False, points, 2)
        
        # Текущее значение
        if self.values:
            current = self.values[-1]
            val_text = font.render(f"{current:.3f}", True, self.line_color)
            surface.blit(
                val_text,
                (self.x + self.width - SIZES['padding_sm'] - val_text.get_width(), 
                 self.y + SIZES['padding_sm'])
            )


class EmotionWheel:
    """
    Колесо эмоций
    
    Визуализация эмоционального состояния вокселей.
    """
    
    EMOTIONS = [
        ('joy', COLORS['emotion_joy']),
        ('curiosity', COLORS['emotion_curiosity']),
        ('surprise', COLORS['emotion_surprise']),
        ('anger', COLORS['emotion_anger']),
        ('disgust', COLORS['emotion_disgust']),
        ('sadness', COLORS['emotion_sadness']),
        ('fear', COLORS['emotion_fear']),
        ('peace', COLORS['emotion_peace']),
    ]
    
    def __init__(self, x: int, y: int, radius: int):
        self.x = x
        self.y = y
        self.radius = radius
        
        self.values: Dict[str, float] = {e[0]: 0.5 for e in self.EMOTIONS}
        self._animated_values: Dict[str, float] = self.values.copy()
        
        # Анимация вращения
        self.rotation = 0.0
    
    def update_emotions(self, emotions: Dict[str, float]):
        """Обновление значений эмоций"""
        for key, value in emotions.items():
            if key in self.values:
                self.values[key] = value
    
    def update(self, dt: float):
        """Обновление анимаций"""
        self.rotation += dt * 0.2
        
        for key in self.values:
            target = self.values[key]
            current = self._animated_values.get(key, target)
            self._animated_values[key] = current + (target - current) * 3 * dt
    
    def render(self, surface: pygame.Surface):
        """Рендеринг колеса"""
        if not PYGAME_AVAILABLE:
            return
        
        # Фоновый круг
        pygame.draw.circle(
            surface,
            COLORS['bg_tertiary'],
            (self.x, self.y),
            self.radius
        )
        
        # Сегменты эмоций
        num_emotions = len(self.EMOTIONS)
        angle_step = 2 * math.pi / num_emotions
        
        for i, (emotion, color) in enumerate(self.EMOTIONS):
            angle = self.rotation + i * angle_step
            value = self._animated_values.get(emotion, 0.5)
            
            # Радиус сегмента зависит от значения
            segment_radius = int(self.radius * 0.3 + self.radius * 0.6 * value)
            
            # Координаты
            x = self.x + int(segment_radius * 0.6 * math.cos(angle))
            y = self.y + int(segment_radius * 0.6 * math.sin(angle))
            
            # Размер точки
            dot_size = int(8 + 12 * value)
            
            # Свечение
            glow_surface = pygame.Surface((dot_size * 4, dot_size * 4), pygame.SRCALPHA)
            pygame.draw.circle(
                glow_surface,
                (*color[:3], 50),
                (dot_size * 2, dot_size * 2),
                dot_size * 2
            )
            surface.blit(glow_surface, (x - dot_size * 2, y - dot_size * 2))
            
            # Основная точка
            pygame.draw.circle(surface, color, (x, y), dot_size)
            
            # Метка
            font = pygame.font.SysFont('monospace', 10)
            label_x = self.x + int((self.radius + 20) * math.cos(angle))
            label_y = self.y + int((self.radius + 20) * math.sin(angle))
            
            label = font.render(emotion[:3].upper(), True, COLORS['text_muted'])
            label_rect = label.get_rect(center=(label_x, label_y))
            surface.blit(label, label_rect)
        
        # Центр
        pygame.draw.circle(surface, COLORS['bg_highlight'], (self.x, self.y), 15)
        pygame.draw.circle(surface, COLORS['accent_cyan'], (self.x, self.y), 8)

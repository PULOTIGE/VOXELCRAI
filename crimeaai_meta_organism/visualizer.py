"""
CrimeaAI Meta Organism - Real-time Visualization
═══════════════════════════════════════════════════════════════════════════════

Визуализация в реальном времени через Open3D и Plotly.
Оптимизировано для миллиона точек без лагов.

LightPattern 1KB - освещение зависит от energy/emotion.
"""

import numpy as np
from typing import Optional, Dict, Callable, Tuple
import threading
import time

# Попытка импорта Open3D
try:
    import open3d as o3d
    HAS_OPEN3D = True
except ImportError:
    HAS_OPEN3D = False
    print("⚠️ Open3D не найден, будет использован Plotly")

# Попытка импорта Plotly
try:
    import plotly.graph_objects as go
    from plotly.subplots import make_subplots
    HAS_PLOTLY = True
except ImportError:
    HAS_PLOTLY = False
    print("⚠️ Plotly не найден")

# Matplotlib как fallback
try:
    import matplotlib.pyplot as plt
    from mpl_toolkits.mplot3d import Axes3D
    HAS_MATPLOTLIB = True
except ImportError:
    HAS_MATPLOTLIB = False


class LightPattern1KB:
    """
    LightPattern 1KB - компактная система освещения.
    
    Освещение зависит от:
    - energy вокселей
    - emotion (радость = яркость, страх = мерцание, гнев = красное свечение)
    - trauma (затемнение + красный оттенок)
    """
    
    def __init__(self):
        # Параметры освещения (помещаются в 1KB)
        self.ambient = 0.3
        self.diffuse = 0.6
        self.specular = 0.2
        self.light_pos = np.array([100.0, 100.0, 100.0])
        
        # Эмоциональные модификаторы
        self.joy_brightness = 1.2
        self.fear_flicker_rate = 5.0
        self.anger_red_boost = 0.4
        self.peace_blue_tint = 0.2
        
        # Время для анимации
        self.time = 0.0
    
    def compute_lighting(self, positions: np.ndarray, colors: np.ndarray,
                        emotions: np.ndarray, energies: np.ndarray,
                        traumas: np.ndarray) -> np.ndarray:
        """
        Вычислить освещённые цвета для всех вокселей.
        
        Args:
            positions: (N, 3) позиции вокселей
            colors: (N, 3) базовые цвета
            emotions: (N, 4) эмоции [joy, fear, anger, peace]
            energies: (N,) энергия вокселей
            traumas: (N,) травмы вокселей
        
        Returns:
            (N, 3) освещённые цвета
        """
        if len(positions) == 0:
            return colors
        
        N = len(positions)
        
        # Направление к свету
        light_dirs = self.light_pos - positions
        light_dists = np.linalg.norm(light_dirs, axis=1, keepdims=True) + 1e-10
        light_dirs = light_dirs / light_dists
        
        # Простое диффузное освещение
        # Нормаль = направление от центра (для сферы)
        center = np.mean(positions, axis=0)
        normals = positions - center
        normal_lens = np.linalg.norm(normals, axis=1, keepdims=True) + 1e-10
        normals = normals / normal_lens
        
        diffuse = np.maximum(0, np.sum(normals * light_dirs, axis=1))
        
        # Базовое освещение
        lighting = self.ambient + self.diffuse * diffuse
        
        # Эмоциональные модификации
        joy = emotions[:, 0]
        fear = emotions[:, 1]
        anger = emotions[:, 2]
        peace = emotions[:, 3]
        
        # Радость увеличивает яркость
        lighting = lighting * (1.0 + joy * (self.joy_brightness - 1.0))
        
        # Страх вызывает мерцание
        flicker = 0.5 + 0.5 * np.sin(self.time * self.fear_flicker_rate + 
                                      np.arange(N) * 0.1)
        lighting = lighting * (1.0 - fear * 0.3 * flicker)
        
        # Применяем освещение к цветам
        lit_colors = colors * lighting[:, np.newaxis]
        
        # Гнев добавляет красный
        lit_colors[:, 0] = np.minimum(1.0, lit_colors[:, 0] + anger * self.anger_red_boost)
        
        # Покой добавляет синий оттенок
        lit_colors[:, 2] = np.minimum(1.0, lit_colors[:, 2] + peace * self.peace_blue_tint)
        
        # Травма затемняет и добавляет красный
        trauma_factor = 1.0 - traumas * 0.5
        lit_colors = lit_colors * trauma_factor[:, np.newaxis]
        lit_colors[:, 0] = np.minimum(1.0, lit_colors[:, 0] + traumas * 0.3)
        
        # Энергия влияет на общую яркость
        lit_colors = lit_colors * energies[:, np.newaxis]
        
        return np.clip(lit_colors, 0, 1)
    
    def update(self, dt: float):
        """Обновить время для анимации"""
        self.time += dt


class Open3DVisualizer:
    """
    Визуализатор на Open3D - оптимизирован для миллиона точек.
    """
    
    def __init__(self, window_name: str = "CrimeaAI Meta Organism"):
        if not HAS_OPEN3D:
            raise ImportError("Open3D не установлен")
        
        self.window_name = window_name
        self.vis = None
        self.pcd = None
        self.running = False
        self.light_pattern = LightPattern1KB()
        
    def initialize(self):
        """Инициализация визуализатора"""
        self.vis = o3d.visualization.Visualizer()
        self.vis.create_window(window_name=self.window_name, width=1280, height=720)
        
        # Настройка рендеринга
        render_opt = self.vis.get_render_option()
        render_opt.background_color = np.array([0.02, 0.02, 0.05])  # Тёмный космос
        render_opt.point_size = 2.0
        
        # Создание облака точек
        self.pcd = o3d.geometry.PointCloud()
        self.vis.add_geometry(self.pcd)
        
        self.running = True
    
    def update(self, positions: np.ndarray, colors: np.ndarray,
               emotions: Optional[np.ndarray] = None,
               energies: Optional[np.ndarray] = None,
               traumas: Optional[np.ndarray] = None,
               dt: float = 0.016):
        """Обновить визуализацию"""
        if not self.running or self.pcd is None:
            return False
        
        if len(positions) == 0:
            return True
        
        # Применить освещение если есть данные
        if emotions is not None and energies is not None and traumas is not None:
            self.light_pattern.update(dt)
            colors = self.light_pattern.compute_lighting(
                positions, colors, emotions, energies, traumas
            )
        
        # Обновить облако точек
        self.pcd.points = o3d.utility.Vector3dVector(positions)
        self.pcd.colors = o3d.utility.Vector3dVector(colors)
        
        # Обновить геометрию
        self.vis.update_geometry(self.pcd)
        self.vis.poll_events()
        self.vis.update_renderer()
        
        return True
    
    def close(self):
        """Закрыть визуализатор"""
        self.running = False
        if self.vis:
            self.vis.destroy_window()


class PlotlyVisualizer:
    """
    Визуализатор на Plotly - для интерактивного просмотра в браузере.
    Оптимизирован через downsampling для больших датасетов.
    """
    
    def __init__(self, max_points: int = 50000):
        if not HAS_PLOTLY:
            raise ImportError("Plotly не установлен")
        
        self.max_points = max_points
        self.fig = None
        self.light_pattern = LightPattern1KB()
    
    def create_figure(self, positions: np.ndarray, colors: np.ndarray,
                     title: str = "CrimeaAI Meta Organism",
                     emotions: Optional[np.ndarray] = None,
                     energies: Optional[np.ndarray] = None,
                     traumas: Optional[np.ndarray] = None) -> go.Figure:
        """Создать фигуру Plotly"""
        
        # Downsampling если слишком много точек
        if len(positions) > self.max_points:
            indices = np.random.choice(len(positions), self.max_points, replace=False)
            positions = positions[indices]
            colors = colors[indices]
            if emotions is not None:
                emotions = emotions[indices]
            if energies is not None:
                energies = energies[indices]
            if traumas is not None:
                traumas = traumas[indices]
        
        # Применить освещение
        if emotions is not None and energies is not None and traumas is not None:
            colors = self.light_pattern.compute_lighting(
                positions, colors, emotions, energies, traumas
            )
        
        # Конвертация цветов в формат Plotly
        color_strings = [f'rgb({int(r*255)},{int(g*255)},{int(b*255)})' 
                        for r, g, b in colors]
        
        # Создание scatter3d
        scatter = go.Scatter3d(
            x=positions[:, 0],
            y=positions[:, 1],
            z=positions[:, 2],
            mode='markers',
            marker=dict(
                size=2,
                color=color_strings,
                opacity=0.8
            ),
            hoverinfo='none'
        )
        
        # Создание фигуры
        self.fig = go.Figure(data=[scatter])
        
        self.fig.update_layout(
            title=dict(
                text=title,
                font=dict(size=24, color='white')
            ),
            scene=dict(
                xaxis=dict(showbackground=False, showgrid=False, zeroline=False, visible=False),
                yaxis=dict(showbackground=False, showgrid=False, zeroline=False, visible=False),
                zaxis=dict(showbackground=False, showgrid=False, zeroline=False, visible=False),
                bgcolor='rgb(5, 5, 15)'
            ),
            paper_bgcolor='rgb(5, 5, 15)',
            plot_bgcolor='rgb(5, 5, 15)',
            margin=dict(l=0, r=0, t=50, b=0)
        )
        
        return self.fig
    
    def show(self):
        """Показать фигуру в браузере"""
        if self.fig:
            self.fig.show()
    
    def save_html(self, path: str):
        """Сохранить как HTML файл"""
        if self.fig:
            self.fig.write_html(path)


class MatplotlibVisualizer:
    """
    Визуализатор на Matplotlib - fallback вариант.
    """
    
    def __init__(self, max_points: int = 20000):
        if not HAS_MATPLOTLIB:
            raise ImportError("Matplotlib не установлен")
        
        self.max_points = max_points
        self.fig = None
        self.ax = None
        self.scatter = None
        self.light_pattern = LightPattern1KB()
        
    def initialize(self):
        """Инициализация"""
        plt.ion()
        self.fig = plt.figure(figsize=(12, 8), facecolor='#050510')
        self.ax = self.fig.add_subplot(111, projection='3d', facecolor='#050510')
        
        # Убрать оси
        self.ax.set_axis_off()
        
        plt.tight_layout()
        plt.show(block=False)
    
    def update(self, positions: np.ndarray, colors: np.ndarray,
               title: str = "",
               emotions: Optional[np.ndarray] = None,
               energies: Optional[np.ndarray] = None,
               traumas: Optional[np.ndarray] = None,
               dt: float = 0.016):
        """Обновить визуализацию"""
        
        # Downsampling
        if len(positions) > self.max_points:
            indices = np.random.choice(len(positions), self.max_points, replace=False)
            positions = positions[indices]
            colors = colors[indices]
            if emotions is not None:
                emotions = emotions[indices]
            if energies is not None:
                energies = energies[indices]
            if traumas is not None:
                traumas = traumas[indices]
        
        # Применить освещение
        if emotions is not None and energies is not None and traumas is not None:
            self.light_pattern.update(dt)
            colors = self.light_pattern.compute_lighting(
                positions, colors, emotions, energies, traumas
            )
        
        self.ax.clear()
        self.ax.set_axis_off()
        self.ax.set_facecolor('#050510')
        
        if len(positions) > 0:
            self.ax.scatter(
                positions[:, 0],
                positions[:, 1],
                positions[:, 2],
                c=colors,
                s=1,
                alpha=0.8
            )
        
        if title:
            self.ax.set_title(title, color='white', fontsize=14)
        
        self.fig.canvas.draw()
        self.fig.canvas.flush_events()
    
    def close(self):
        """Закрыть"""
        plt.close(self.fig)


def create_visualizer(backend: str = 'auto'):
    """
    Создать визуализатор с автоматическим выбором бэкенда.
    
    Args:
        backend: 'open3d', 'plotly', 'matplotlib', или 'auto'
    
    Returns:
        Экземпляр визуализатора
    """
    if backend == 'auto':
        if HAS_OPEN3D:
            return Open3DVisualizer()
        elif HAS_PLOTLY:
            return PlotlyVisualizer()
        elif HAS_MATPLOTLIB:
            return MatplotlibVisualizer()
        else:
            raise ImportError("Ни один из визуализаторов не доступен")
    
    elif backend == 'open3d':
        return Open3DVisualizer()
    elif backend == 'plotly':
        return PlotlyVisualizer()
    elif backend == 'matplotlib':
        return MatplotlibVisualizer()
    else:
        raise ValueError(f"Неизвестный бэкенд: {backend}")

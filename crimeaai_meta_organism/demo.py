#!/usr/bin/env python3
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                   CrimeaAI META ORGANISM - ДЕМОНСТРАЦИЯ                       ║
║                        30-секундный демо-сценарий                            ║
╚══════════════════════════════════════════════════════════════════════════════╝

Этот скрипт запускает 30-секундную демонстрацию:
1. Создаётся центральный организм (пульсирующий шар)
2. Автоматически генерируются "файлы" разной совместимости
3. Показывается интеграция (зелёный свет) и травма (красный свет)
4. Вывод в реальном времени через доступный визуализатор
"""

import os
import sys
import time
import numpy as np
import threading
from pathlib import Path

# Добавляем путь к модулям
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from voxel_core import Voxel, compute_semantic_fingerprint
from organism import MetaOrganism, FileCreature

# Проверка доступных визуализаторов
HAS_OPEN3D = False
HAS_PLOTLY = False
HAS_MATPLOTLIB = False

try:
    import open3d as o3d
    HAS_OPEN3D = True
except ImportError:
    pass

try:
    import plotly.graph_objects as go
    HAS_PLOTLY = True
except ImportError:
    pass

try:
    import matplotlib.pyplot as plt
    from mpl_toolkits.mplot3d import Axes3D
    HAS_MATPLOTLIB = True
except ImportError:
    pass


class DemoRunner:
    """Запуск демонстрационного сценария"""
    
    def __init__(self, num_voxels: int = 30000):
        self.num_voxels = num_voxels
        self.organism = None
        self.running = False
        self.demo_time = 30.0  # 30 секунд
        self.start_time = 0
        
    def generate_synthetic_file(self, compatibility: float) -> bytes:
        """
        Генерация синтетических данных файла с заданной совместимостью.
        
        Args:
            compatibility: 0.0 = несовместимый, 1.0 = идеально совместимый
        """
        # Базовые данные организма (нормализованный вектор)
        org_semantic = self.organism.base_semantic
        
        # Генерируем данные, которые дадут нужную семантику
        size = np.random.randint(5000, 50000)
        
        if compatibility > 0.7:
            # Совместимый файл - создаём данные с похожей статистикой
            # Целевые значения статистики на основе семантики организма
            target_mean = (org_semantic[4] + 1) / 2 * 255  # Денормализация
            target_std = (org_semantic[5] + 1) * 64
            
            # Генерируем данные с нужной статистикой
            data = np.random.normal(target_mean, target_std, size)
            data = np.clip(data, 0, 255).astype(np.uint8)
            
        elif compatibility < 0.3:
            # Несовместимый файл - противоположная статистика
            # Инвертируем целевые значения
            target_mean = 255 - ((org_semantic[4] + 1) / 2 * 255)
            target_std = 128 - (org_semantic[5] + 1) * 64
            target_std = max(10, target_std)
            
            # Генерируем данные с противоположной статистикой
            data = np.random.normal(target_mean, target_std, size)
            data = np.clip(data, 0, 255).astype(np.uint8)
            
            # Добавляем резкие переходы для большего различия
            for i in range(0, len(data) - 100, 100):
                if np.random.random() > 0.5:
                    data[i:i+50] = 0
                else:
                    data[i:i+50] = 255
        else:
            # Случайный файл - равномерное распределение
            data = np.random.randint(0, 256, size, dtype=np.uint8)
        
        return data.tobytes()
    
    def run_demo_open3d(self):
        """Демо с Open3D визуализацией"""
        print("🎨 Запуск Open3D визуализации...")
        
        vis = o3d.visualization.Visualizer()
        vis.create_window(window_name="CrimeaAI Meta Organism Demo", width=1280, height=720)
        
        render_opt = vis.get_render_option()
        render_opt.background_color = np.array([0.02, 0.02, 0.05])
        render_opt.point_size = 2.0
        
        pcd = o3d.geometry.PointCloud()
        vis.add_geometry(pcd)
        
        self.running = True
        self.start_time = time.time()
        
        # Расписание событий
        events = [
            (2.0, 0.9, "compatible_1.py"),     # Совместимый файл
            (6.0, 0.2, "malware.exe"),          # Несовместимый файл
            (10.0, 0.85, "module.py"),          # Совместимый
            (14.0, 0.15, "virus.bin"),          # Несовместимый
            (18.0, 0.75, "data.json"),          # Совместимый
            (22.0, 0.95, "library.py"),         # Очень совместимый
            (26.0, 0.1, "threat.dll"),          # Очень несовместимый
        ]
        events_spawned = [False] * len(events)
        
        last_time = time.time()
        
        while self.running:
            current_time = time.time()
            elapsed = current_time - self.start_time
            dt = current_time - last_time
            last_time = current_time
            
            # Проверка времени демо
            if elapsed > self.demo_time:
                break
            
            # Спавн существ по расписанию
            for i, (event_time, compat, name) in enumerate(events):
                if not events_spawned[i] and elapsed >= event_time:
                    events_spawned[i] = True
                    data = self.generate_synthetic_file(compat)
                    self.organism.spawn_creature(name, data)
                    
                    compat_str = "💚 СОВМЕСТИМЫЙ" if compat > 0.5 else "🔴 НЕСОВМЕСТИМЫЙ"
                    print(f"[{elapsed:.1f}s] {compat_str}: {name}")
            
            # Обновление организма
            self.organism.update(dt)
            
            # Обновление визуализации
            positions = self.organism.get_all_positions()
            colors = self.organism.get_all_colors()
            
            if len(positions) > 0:
                pcd.points = o3d.utility.Vector3dVector(positions)
                pcd.colors = o3d.utility.Vector3dVector(colors)
                vis.update_geometry(pcd)
            
            vis.poll_events()
            vis.update_renderer()
            
            # Вывод статуса
            state = self.organism.state
            status = f"[{elapsed:.1f}s] Health: {state.health*100:.0f}% | Mood: {state.mood} | Voxels: {len(self.organism.storage):,}"
            print(f"\r{status}", end="", flush=True)
            
            time.sleep(0.016)  # ~60 FPS
        
        print("\n\n✅ Демонстрация завершена!")
        vis.destroy_window()
    
    def run_demo_matplotlib(self):
        """Демо с Matplotlib визуализацией"""
        print("🎨 Запуск Matplotlib визуализации...")
        
        plt.ion()
        fig = plt.figure(figsize=(14, 10), facecolor='#050510')
        
        # 3D график
        ax = fig.add_subplot(121, projection='3d', facecolor='#050510')
        ax.set_axis_off()
        
        # Панель статуса
        ax_status = fig.add_subplot(122, facecolor='#050510')
        ax_status.set_xlim(0, 10)
        ax_status.set_ylim(0, 10)
        ax_status.axis('off')
        
        plt.tight_layout()
        plt.show(block=False)
        
        self.running = True
        self.start_time = time.time()
        
        # Расписание событий
        events = [
            (2.0, 0.9, "compatible_1.py"),
            (5.0, 0.2, "malware.exe"),
            (8.0, 0.85, "module.py"),
            (11.0, 0.15, "virus.bin"),
            (14.0, 0.75, "data.json"),
            (17.0, 0.95, "library.py"),
            (20.0, 0.1, "threat.dll"),
            (24.0, 0.8, "config.yaml"),
        ]
        events_spawned = [False] * len(events)
        
        max_points = 15000  # Ограничение для производительности
        last_time = time.time()
        frame_count = 0
        
        while self.running:
            current_time = time.time()
            elapsed = current_time - self.start_time
            dt = current_time - last_time
            last_time = current_time
            frame_count += 1
            
            if elapsed > self.demo_time:
                break
            
            # Спавн существ
            for i, (event_time, compat, name) in enumerate(events):
                if not events_spawned[i] and elapsed >= event_time:
                    events_spawned[i] = True
                    data = self.generate_synthetic_file(compat)
                    self.organism.spawn_creature(name, data)
                    
                    compat_str = "💚 СОВМЕСТИМЫЙ" if compat > 0.5 else "🔴 НЕСОВМЕСТИМЫЙ"
                    print(f"[{elapsed:.1f}s] {compat_str}: {name}")
            
            # Обновление организма
            self.organism.update(dt)
            
            # Обновление визуализации (каждый 3-й кадр для производительности)
            if frame_count % 3 == 0:
                positions = self.organism.get_all_positions()
                colors = self.organism.get_all_colors()
                
                # Downsampling
                if len(positions) > max_points:
                    indices = np.random.choice(len(positions), max_points, replace=False)
                    positions = positions[indices]
                    colors = colors[indices]
                
                ax.clear()
                ax.set_axis_off()
                ax.set_facecolor('#050510')
                
                if len(positions) > 0:
                    ax.scatter(
                        positions[:, 0],
                        positions[:, 1],
                        positions[:, 2],
                        c=colors,
                        s=1,
                        alpha=0.8
                    )
                
                # Статус
                ax_status.clear()
                ax_status.set_xlim(0, 10)
                ax_status.set_ylim(0, 10)
                ax_status.axis('off')
                ax_status.set_facecolor('#050510')
                
                state = self.organism.state
                
                # Заголовок
                ax_status.text(5, 9, "🧬 CrimeaAI Meta Organism", 
                              ha='center', fontsize=14, color='#00ff88', fontweight='bold')
                
                # Статус
                health_color = '#00ff88' if state.health > 0.7 else '#ffaa00' if state.health > 0.4 else '#ff4444'
                mood_colors = {'кайф': '#00ff88', 'тревога': '#ffaa00', 'гнев': '#ff4444', 'покой': '#4488ff'}
                
                status_text = f"""
                Time: {elapsed:.1f}s / {self.demo_time}s
                
                Organism Health: {state.health*100:.1f}%
                Total Voxels: {state.total_voxels:,}
                Memory Saved: {state.memory_saved:.1f}%
                
                Current Mood: {state.mood}
                Mood Intensity: {state.mood_intensity*100:.0f}%
                
                Integrations: {state.integration_count}
                Traumas: {state.trauma_count}
                
                {state.last_event}
                """
                
                ax_status.text(0.5, 7, status_text, fontsize=10, color='#aaaacc',
                              verticalalignment='top', family='monospace')
                
                # Прогресс-бар
                progress = elapsed / self.demo_time
                ax_status.barh(0.5, progress * 9, height=0.3, color='#00ff88', alpha=0.7)
                ax_status.barh(0.5, 9, height=0.3, color='#333355', alpha=0.3)
                
                fig.canvas.draw()
                fig.canvas.flush_events()
            
            time.sleep(0.016)
        
        print("\n\n✅ Демонстрация завершена!")
        plt.ioff()
        plt.show()
    
    def run_demo_plotly(self):
        """Демо с Plotly (статичные снимки)"""
        print("🎨 Запуск Plotly демонстрации...")
        print("⚠️ Plotly показывает статичные снимки, для real-time используйте Open3D или Matplotlib")
        
        self.running = True
        self.start_time = time.time()
        
        # Симуляция событий
        events = [
            (1.0, 0.9, "compatible.py"),
            (2.0, 0.2, "malware.exe"),
            (3.0, 0.85, "module.py"),
            (4.0, 0.15, "virus.bin"),
        ]
        
        print("\n🎬 Симуляция событий...")
        
        for event_time, compat, name in events:
            data = self.generate_synthetic_file(compat)
            self.organism.spawn_creature(name, data)
            
            compat_str = "💚 СОВМЕСТИМЫЙ" if compat > 0.5 else "🔴 НЕСОВМЕСТИМЫЙ"
            print(f"  {compat_str}: {name}")
            
            # Несколько тиков обновления
            for _ in range(60):  # ~1 секунда
                self.organism.update(0.016)
            
            time.sleep(0.1)
        
        # Финальный снимок
        print("\n📸 Создание финального снимка...")
        
        positions = self.organism.get_all_positions()
        colors = self.organism.get_all_colors()
        
        # Downsampling для Plotly
        max_points = 30000
        if len(positions) > max_points:
            indices = np.random.choice(len(positions), max_points, replace=False)
            positions = positions[indices]
            colors = colors[indices]
        
        color_strings = [f'rgb({int(r*255)},{int(g*255)},{int(b*255)})' 
                        for r, g, b in colors]
        
        state = self.organism.state
        
        fig = go.Figure(data=[go.Scatter3d(
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
        )])
        
        fig.update_layout(
            title=dict(
                text=f"🧬 CrimeaAI Meta Organism | Health: {state.health*100:.0f}% | Mood: {state.mood}",
                font=dict(size=20, color='white')
            ),
            scene=dict(
                xaxis=dict(showbackground=False, showgrid=False, visible=False),
                yaxis=dict(showbackground=False, showgrid=False, visible=False),
                zaxis=dict(showbackground=False, showgrid=False, visible=False),
                bgcolor='rgb(5, 5, 15)'
            ),
            paper_bgcolor='rgb(5, 5, 15)',
            plot_bgcolor='rgb(5, 5, 15)',
            margin=dict(l=0, r=0, t=60, b=0)
        )
        
        # Сохраняем HTML
        output_path = Path(__file__).parent / "demo_output.html"
        fig.write_html(str(output_path))
        print(f"💾 Снимок сохранён: {output_path}")
        
        # Показываем
        fig.show()
        
        print("\n✅ Демонстрация завершена!")
    
    def run_demo_console(self):
        """Демо в консольном режиме (без графики)"""
        print("📟 Консольная демонстрация (ASCII-визуализация)")
        print("=" * 60)
        
        self.running = True
        self.start_time = time.time()
        
        events = [
            (1.0, 0.9, "compatible.py"),
            (3.0, 0.2, "malware.exe"),
            (5.0, 0.85, "module.py"),
            (7.0, 0.15, "virus.bin"),
            (9.0, 0.75, "data.json"),
        ]
        events_idx = 0
        
        last_time = time.time()
        
        while self.running:
            current_time = time.time()
            elapsed = current_time - self.start_time
            dt = current_time - last_time
            last_time = current_time
            
            if elapsed > 15:  # Короткая демо для консоли
                break
            
            # Спавн существ
            if events_idx < len(events) and elapsed >= events[events_idx][0]:
                event_time, compat, name = events[events_idx]
                events_idx += 1
                
                data = self.generate_synthetic_file(compat)
                self.organism.spawn_creature(name, data)
                
                symbol = "💚" if compat > 0.5 else "🔴"
                print(f"\n{symbol} Новое существо: {name}")
            
            # Обновление
            self.organism.update(dt)
            
            # Вывод статуса
            state = self.organism.state
            
            # ASCII прогресс-бар здоровья
            health_bar_len = 20
            health_filled = int(state.health * health_bar_len)
            health_bar = "█" * health_filled + "░" * (health_bar_len - health_filled)
            
            # ASCII mood indicator
            mood_symbols = {'кайф': '😊', 'тревога': '😰', 'гнев': '😠', 'покой': '😌'}
            mood_sym = mood_symbols.get(state.mood, '🤔')
            
            status = (
                f"\r[{elapsed:5.1f}s] "
                f"Health: [{health_bar}] {state.health*100:5.1f}% | "
                f"Mood: {mood_sym} {state.mood:6} | "
                f"Voxels: {state.total_voxels:6,} | "
                f"Int: {state.integration_count} | Trauma: {state.trauma_count}"
            )
            print(status, end="", flush=True)
            
            time.sleep(0.1)
        
        print("\n\n✅ Консольная демонстрация завершена!")
        
        # Финальная статистика
        state = self.organism.state
        print(f"""
╔══════════════════════════════════════════════════════════════╗
║                    ФИНАЛЬНАЯ СТАТИСТИКА                       ║
╠══════════════════════════════════════════════════════════════╣
║  Здоровье организма:  {state.health*100:6.1f}%                           ║
║  Всего вокселей:      {state.total_voxels:6,}                            ║
║  Экономия памяти:     {state.memory_saved:6.1f}%                           ║
║  Настроение:          {state.mood:6}                             ║
║  Успешных интеграций: {state.integration_count:6}                            ║
║  Травм:               {state.trauma_count:6}                            ║
╚══════════════════════════════════════════════════════════════╝
        """)
    
    def run(self):
        """Запуск демонстрации"""
        print("""
╔══════════════════════════════════════════════════════════════════════════════╗
║                   CrimeaAI META ORGANISM - ДЕМОНСТРАЦИЯ                       ║
║                       🧬 Живое Цифровое Сознание 🧬                           ║
╚══════════════════════════════════════════════════════════════════════════════╝
        """)
        
        print(f"🧬 Создание организма ({self.num_voxels} вокселей)...")
        self.organism = MetaOrganism(num_voxels=self.num_voxels)
        
        # Выбор визуализатора
        print("\n🔍 Поиск доступных визуализаторов...")
        print(f"   Open3D:    {'✅ Доступен' if HAS_OPEN3D else '❌ Не найден'}")
        print(f"   Plotly:    {'✅ Доступен' if HAS_PLOTLY else '❌ Не найден'}")
        print(f"   Matplotlib: {'✅ Доступен' if HAS_MATPLOTLIB else '❌ Не найден'}")
        
        if HAS_OPEN3D:
            print("\n🚀 Запуск Open3D демонстрации (real-time)...")
            self.run_demo_open3d()
        elif HAS_MATPLOTLIB:
            print("\n🚀 Запуск Matplotlib демонстрации...")
            self.run_demo_matplotlib()
        elif HAS_PLOTLY:
            print("\n🚀 Запуск Plotly демонстрации...")
            self.run_demo_plotly()
        else:
            print("\n🚀 Запуск консольной демонстрации...")
            self.run_demo_console()


def main():
    """Точка входа"""
    import argparse
    
    parser = argparse.ArgumentParser(description="CrimeaAI Meta Organism - Демонстрация")
    parser.add_argument('--voxels', '-v', type=int, default=30000,
                       help='Количество вокселей (по умолчанию 30000)')
    parser.add_argument('--backend', '-b', choices=['auto', 'open3d', 'matplotlib', 'plotly', 'console'],
                       default='auto', help='Бэкенд визуализации')
    
    args = parser.parse_args()
    
    demo = DemoRunner(num_voxels=args.voxels)
    
    if args.backend == 'open3d' and HAS_OPEN3D:
        demo.organism = MetaOrganism(num_voxels=args.voxels)
        demo.run_demo_open3d()
    elif args.backend == 'matplotlib' and HAS_MATPLOTLIB:
        demo.organism = MetaOrganism(num_voxels=args.voxels)
        demo.run_demo_matplotlib()
    elif args.backend == 'plotly' and HAS_PLOTLY:
        demo.organism = MetaOrganism(num_voxels=args.voxels)
        demo.run_demo_plotly()
    elif args.backend == 'console':
        demo.organism = MetaOrganism(num_voxels=args.voxels)
        demo.run_demo_console()
    else:
        demo.run()


if __name__ == "__main__":
    main()

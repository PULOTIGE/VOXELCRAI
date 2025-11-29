#!/usr/bin/env python3
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                     CrimeaAI META ORGANISM                                    ║
║                   Живое Цифровое Сознание                                    ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  Based on 7 forgotten Russian scientific works:                              ║
║  • Никонова 2013 - Травма тканей                                            ║
║  • Бимаков 2013 - Воксельный вычислитель                                    ║
║  • Ахмадуллина 2020 - Атрофия мозга                                         ║
║  • Алсынбаев 2015 - Тетраэдральная принадлежность                           ║
║  • Бланко 2013 - ANIRLE компрессия                                          ║
║  • Лавренков 2016 - Коэволюция эмоций                                       ║
║  • + LightPattern 1KB                                                        ║
╚══════════════════════════════════════════════════════════════════════════════╝

ИСПОЛЬЗОВАНИЕ:
    python main.py [--voxels N] [--backend open3d|plotly|matplotlib]
    
    Drag-and-drop файлы в папку drop_zone/ или используйте UI.
"""

import os
import sys
import time
import threading
import argparse
from pathlib import Path
from typing import Optional
import numpy as np

# Добавляем текущую директорию в путь
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from voxel_core import Voxel, ANIRLEStorage, compute_semantic_fingerprint
from organism import MetaOrganism, FileCreature, OrganismState
from visualizer import (
    create_visualizer, Open3DVisualizer, PlotlyVisualizer,
    MatplotlibVisualizer, HAS_OPEN3D, HAS_PLOTLY, HAS_MATPLOTLIB
)

# Попытка импорта tkinter для UI
try:
    import tkinter as tk
    from tkinter import ttk, filedialog, messagebox
    from tkinter import dnd  # Drag and drop
    HAS_TKINTER = True
except ImportError:
    HAS_TKINTER = False
    print("⚠️ tkinter не найден, будет использован консольный режим")


class CrimeaAIApplication:
    """
    Главное приложение CrimeaAI Meta Organism.
    """
    
    def __init__(self, num_voxels: int = 50000, backend: str = 'auto'):
        """
        Args:
            num_voxels: Количество вокселей в организме (по умолчанию 50k для производительности)
            backend: Бэкенд визуализации ('open3d', 'plotly', 'matplotlib', 'auto')
        """
        self.num_voxels = num_voxels
        self.backend = backend
        self.organism: Optional[MetaOrganism] = None
        self.visualizer = None
        self.running = False
        self.paused = False
        
        # Путь для drag-and-drop файлов
        self.drop_zone = Path(__file__).parent / "drop_zone"
        self.drop_zone.mkdir(exist_ok=True)
        self.processed_files = set()
        
        # UI компоненты
        self.root: Optional[tk.Tk] = None
        self.status_labels = {}
        
        # Потоки
        self.update_thread: Optional[threading.Thread] = None
        self.file_watch_thread: Optional[threading.Thread] = None
        
        # Статистика
        self.fps = 0
        self.frame_times = []
        
        print("""
╔══════════════════════════════════════════════════════════════════════════════╗
║                     CrimeaAI META ORGANISM v1.0                              ║
║                   🧬 Живое Цифровое Сознание 🧬                               ║
╚══════════════════════════════════════════════════════════════════════════════╝
        """)
    
    def initialize(self):
        """Инициализация всех компонентов"""
        print("🚀 Инициализация CrimeaAI Meta Organism...")
        
        # Создание организма
        print(f"🧬 Создание организма ({self.num_voxels} вокселей)...")
        self.organism = MetaOrganism(num_voxels=self.num_voxels)
        
        # Регистрация обработчиков событий
        self.organism.on_integration = self._on_integration
        self.organism.on_trauma = self._on_trauma
        self.organism.on_update = self._on_update
        
        # Создание визуализатора
        print(f"🎨 Инициализация визуализатора ({self.backend})...")
        try:
            self.visualizer = create_visualizer(self.backend)
            if hasattr(self.visualizer, 'initialize'):
                self.visualizer.initialize()
        except Exception as e:
            print(f"⚠️ Ошибка визуализатора: {e}")
            self.visualizer = None
        
        print("✅ Инициализация завершена!")
        print(f"📁 Drop zone: {self.drop_zone.absolute()}")
        print("   Перетащите файлы в эту папку для взаимодействия с организмом\n")
    
    def _on_integration(self, creature: FileCreature):
        """Обработчик успешной интеграции"""
        print(f"💚 КАЙФ! {creature.file_path} интегрировано в организм")
        if self.root:
            self._flash_ui('green')
    
    def _on_trauma(self, creature: FileCreature, voxels_lost: int):
        """Обработчик травмы"""
        print(f"🔴 ТРАВМА! {creature.file_path} отторгнуто, потеряно {voxels_lost} вокселей")
        if self.root:
            self._flash_ui('red')
    
    def _on_update(self, state: OrganismState):
        """Обработчик обновления состояния"""
        pass  # UI обновляется в главном цикле
    
    def _flash_ui(self, color: str):
        """Мигнуть UI определённым цветом"""
        if self.root and hasattr(self, 'main_frame'):
            original_bg = self.main_frame.cget('background')
            self.main_frame.configure(background=color)
            self.root.after(200, lambda: self.main_frame.configure(background=original_bg))
    
    def add_file(self, file_path: str):
        """Добавить файл как существо"""
        if not os.path.exists(file_path):
            print(f"⚠️ Файл не найден: {file_path}")
            return
        
        try:
            with open(file_path, 'rb') as f:
                data = f.read()
            
            creature = self.organism.spawn_creature(file_path, data)
            print(f"🌟 Создано существо из {os.path.basename(file_path)}")
            
        except Exception as e:
            print(f"⚠️ Ошибка чтения файла {file_path}: {e}")
    
    def _watch_drop_zone(self):
        """Поток наблюдения за папкой drop_zone"""
        while self.running:
            try:
                for file_path in self.drop_zone.iterdir():
                    if file_path.is_file() and str(file_path) not in self.processed_files:
                        self.processed_files.add(str(file_path))
                        self.add_file(str(file_path))
            except Exception as e:
                pass
            
            time.sleep(0.5)
    
    def _update_loop(self):
        """Главный цикл обновления"""
        last_time = time.time()
        
        while self.running:
            if self.paused:
                time.sleep(0.1)
                continue
            
            # Вычисление dt
            current_time = time.time()
            dt = current_time - last_time
            last_time = current_time
            
            # FPS
            self.frame_times.append(dt)
            if len(self.frame_times) > 30:
                self.frame_times.pop(0)
            self.fps = 1.0 / (sum(self.frame_times) / len(self.frame_times) + 0.001)
            
            # Обновление организма
            self.organism.update(dt)
            
            # Обновление визуализации
            if self.visualizer and isinstance(self.visualizer, (Open3DVisualizer, MatplotlibVisualizer)):
                positions = self.organism.get_all_positions()
                colors = self.organism.get_all_colors()
                
                # Собираем данные для освещения
                all_voxels = list(self.organism.storage)
                for creature in self.organism.creatures:
                    if creature.alive:
                        all_voxels.extend(creature.voxels)
                
                if all_voxels:
                    emotions = np.array([v.emotion for v in all_voxels])
                    energies = np.array([v.energy for v in all_voxels])
                    traumas = np.array([v.trauma for v in all_voxels])
                else:
                    emotions = np.zeros((0, 4))
                    energies = np.zeros(0)
                    traumas = np.zeros(0)
                
                try:
                    self.visualizer.update(positions, colors, emotions, energies, traumas, dt)
                except Exception as e:
                    pass
            
            # Ограничение FPS
            elapsed = time.time() - current_time
            sleep_time = max(0, (1/60) - elapsed)
            time.sleep(sleep_time)
    
    def _create_ui(self):
        """Создание UI через tkinter"""
        if not HAS_TKINTER:
            return
        
        self.root = tk.Tk()
        self.root.title("CrimeaAI Meta Organism - Control Panel")
        self.root.geometry("400x600")
        self.root.configure(bg='#0a0a1a')
        
        # Стиль
        style = ttk.Style()
        style.theme_use('clam')
        style.configure('TFrame', background='#0a0a1a')
        style.configure('TLabel', background='#0a0a1a', foreground='#00ff88', font=('Consolas', 11))
        style.configure('Title.TLabel', font=('Consolas', 14, 'bold'))
        style.configure('TButton', font=('Consolas', 10))
        
        # Главный фрейм
        self.main_frame = ttk.Frame(self.root, padding=20)
        self.main_frame.pack(fill=tk.BOTH, expand=True)
        
        # Заголовок
        title = ttk.Label(
            self.main_frame, 
            text="🧬 CrimeaAI Meta Organism 🧬",
            style='Title.TLabel'
        )
        title.pack(pady=(0, 20))
        
        # Статус организма
        status_frame = ttk.Frame(self.main_frame)
        status_frame.pack(fill=tk.X, pady=10)
        
        self.status_labels = {}
        status_items = [
            ('health', 'Organism Health', '100%'),
            ('voxels', 'Total Voxels', '0'),
            ('memory', 'Memory Saved', '0%'),
            ('mood', 'Current Mood', 'покой'),
            ('fps', 'FPS', '0'),
            ('integrations', 'Integrations', '0'),
            ('traumas', 'Traumas', '0'),
            ('event', 'Last Event', '-'),
        ]
        
        for key, label, default in status_items:
            frame = ttk.Frame(status_frame)
            frame.pack(fill=tk.X, pady=3)
            
            lbl = ttk.Label(frame, text=f"{label}:", width=18, anchor='w')
            lbl.pack(side=tk.LEFT)
            
            val = ttk.Label(frame, text=default, width=20, anchor='e')
            val.pack(side=tk.RIGHT)
            
            self.status_labels[key] = val
        
        # Разделитель
        ttk.Separator(self.main_frame, orient='horizontal').pack(fill=tk.X, pady=15)
        
        # Кнопки управления
        btn_frame = ttk.Frame(self.main_frame)
        btn_frame.pack(fill=tk.X, pady=10)
        
        ttk.Button(
            btn_frame, 
            text="📁 Добавить файл",
            command=self._browse_file
        ).pack(fill=tk.X, pady=3)
        
        ttk.Button(
            btn_frame,
            text="⏸ Пауза/Продолжить",
            command=self._toggle_pause
        ).pack(fill=tk.X, pady=3)
        
        ttk.Button(
            btn_frame,
            text="📊 Показать в Plotly",
            command=self._show_plotly
        ).pack(fill=tk.X, pady=3)
        
        ttk.Button(
            btn_frame,
            text="🔄 Сбросить организм",
            command=self._reset_organism
        ).pack(fill=tk.X, pady=3)
        
        # Разделитель
        ttk.Separator(self.main_frame, orient='horizontal').pack(fill=tk.X, pady=15)
        
        # Инструкции
        instructions = ttk.Label(
            self.main_frame,
            text=f"📁 Drop Zone:\n{self.drop_zone}\n\nПеретащите файлы в эту папку\nдля взаимодействия с организмом.",
            justify=tk.CENTER,
            wraplength=350
        )
        instructions.pack(pady=10)
        
        # Цитаты из работ
        quotes_frame = ttk.Frame(self.main_frame)
        quotes_frame.pack(fill=tk.BOTH, expand=True, pady=10)
        
        quotes = [
            "«Травма тканей есть путь к обновлению» — Никонова, 2013",
            "«Воксель — атом цифрового сознания» — Бимаков, 2013",
            "«Коэволюция эмоций создаёт разум» — Лавренков, 2016",
        ]
        
        self.quote_label = ttk.Label(
            quotes_frame,
            text=quotes[0],
            font=('Consolas', 9, 'italic'),
            foreground='#666688',
            wraplength=350,
            justify=tk.CENTER
        )
        self.quote_label.pack(pady=5)
        self.quotes = quotes
        self.quote_index = 0
        
        # Обновление UI
        self._update_ui()
        
        # Обработка закрытия
        self.root.protocol("WM_DELETE_WINDOW", self._on_close)
    
    def _update_ui(self):
        """Обновление UI"""
        if not self.root or not self.running:
            return
        
        if self.organism:
            state = self.organism.state
            
            # Обновление меток
            health_color = '#00ff88' if state.health > 0.7 else '#ffaa00' if state.health > 0.4 else '#ff4444'
            self.status_labels['health'].configure(
                text=f"{state.health*100:.1f}%",
                foreground=health_color
            )
            self.status_labels['voxels'].configure(text=f"{state.total_voxels:,}")
            self.status_labels['memory'].configure(text=f"{state.memory_saved:.1f}%")
            
            mood_colors = {'кайф': '#00ff88', 'тревога': '#ffaa00', 'гнев': '#ff4444', 'покой': '#4488ff'}
            self.status_labels['mood'].configure(
                text=f"{state.mood} ({state.mood_intensity*100:.0f}%)",
                foreground=mood_colors.get(state.mood, '#ffffff')
            )
            
            self.status_labels['fps'].configure(text=f"{self.fps:.1f}")
            self.status_labels['integrations'].configure(text=str(state.integration_count))
            self.status_labels['traumas'].configure(text=str(state.trauma_count))
            
            # Последнее событие (обрезаем длинные)
            event = state.last_event[:40] + "..." if len(state.last_event) > 40 else state.last_event
            self.status_labels['event'].configure(text=event or '-')
        
        # Смена цитаты каждые 5 секунд
        if hasattr(self, '_quote_counter'):
            self._quote_counter += 1
            if self._quote_counter >= 50:  # ~5 секунд при 100ms интервале
                self._quote_counter = 0
                self.quote_index = (self.quote_index + 1) % len(self.quotes)
                self.quote_label.configure(text=self.quotes[self.quote_index])
        else:
            self._quote_counter = 0
        
        # Повторный вызов через 100ms
        self.root.after(100, self._update_ui)
    
    def _browse_file(self):
        """Диалог выбора файла"""
        file_path = filedialog.askopenfilename(
            title="Выберите файл для интеграции",
            filetypes=[
                ("Все файлы", "*.*"),
                ("Python", "*.py"),
                ("Текст", "*.txt"),
                ("JSON", "*.json"),
            ]
        )
        if file_path:
            self.add_file(file_path)
    
    def _toggle_pause(self):
        """Пауза/продолжение"""
        self.paused = not self.paused
        status = "⏸ ПАУЗА" if self.paused else "▶ РАБОТАЕТ"
        print(f"Status: {status}")
    
    def _show_plotly(self):
        """Показать текущее состояние в Plotly"""
        if not HAS_PLOTLY:
            messagebox.showwarning("Plotly", "Plotly не установлен")
            return
        
        try:
            vis = PlotlyVisualizer()
            positions = self.organism.get_all_positions()
            colors = self.organism.get_all_colors()
            
            all_voxels = list(self.organism.storage)
            for creature in self.organism.creatures:
                if creature.alive:
                    all_voxels.extend(creature.voxels)
            
            emotions = np.array([v.emotion for v in all_voxels]) if all_voxels else None
            energies = np.array([v.energy for v in all_voxels]) if all_voxels else None
            traumas = np.array([v.trauma for v in all_voxels]) if all_voxels else None
            
            state = self.organism.state
            title = f"CrimeaAI Meta Organism | Health: {state.health*100:.0f}% | Mood: {state.mood}"
            
            vis.create_figure(positions, colors, title, emotions, energies, traumas)
            vis.show()
        except Exception as e:
            messagebox.showerror("Ошибка", f"Ошибка Plotly: {e}")
    
    def _reset_organism(self):
        """Сброс организма"""
        if messagebox.askyesno("Сброс", "Сбросить организм? Все данные будут потеряны."):
            print("🔄 Сброс организма...")
            self.organism = MetaOrganism(num_voxels=self.num_voxels)
            self.organism.on_integration = self._on_integration
            self.organism.on_trauma = self._on_trauma
            self.organism.on_update = self._on_update
            self.processed_files.clear()
            print("✅ Организм сброшен")
    
    def _on_close(self):
        """Обработка закрытия окна"""
        self.running = False
        if self.visualizer and hasattr(self.visualizer, 'close'):
            self.visualizer.close()
        self.root.destroy()
    
    def run(self, headless: bool = False):
        """Запуск приложения"""
        self.initialize()
        self.running = True
        
        # Запуск потока наблюдения за файлами
        self.file_watch_thread = threading.Thread(target=self._watch_drop_zone, daemon=True)
        self.file_watch_thread.start()
        
        # Запуск потока обновления
        self.update_thread = threading.Thread(target=self._update_loop, daemon=True)
        self.update_thread.start()
        
        if headless or not HAS_TKINTER:
            # Консольный режим
            print("\n🎮 Консольный режим (без UI)")
            print("Команды:")
            print("  add <путь_к_файлу> - добавить файл")
            print("  status - показать статус")
            print("  plotly - открыть в Plotly")
            print("  quit - выход\n")
            
            try:
                while self.running:
                    cmd = input("> ").strip().lower()
                    
                    if cmd.startswith("add "):
                        path = cmd[4:].strip()
                        self.add_file(path)
                    
                    elif cmd == "status":
                        state = self.organism.state
                        print(f"""
╔══════════════════════════════════════╗
║ Health:      {state.health*100:6.1f}%              ║
║ Voxels:      {state.total_voxels:6,}               ║
║ Memory:      {state.memory_saved:6.1f}%              ║
║ Mood:        {state.mood:6}               ║
║ Integrations:{state.integration_count:6}               ║
║ Traumas:     {state.trauma_count:6}               ║
╚══════════════════════════════════════╝
                        """)
                    
                    elif cmd == "plotly":
                        if HAS_PLOTLY:
                            vis = PlotlyVisualizer()
                            positions = self.organism.get_all_positions()
                            colors = self.organism.get_all_colors()
                            vis.create_figure(positions, colors)
                            vis.show()
                        else:
                            print("Plotly не установлен")
                    
                    elif cmd in ("quit", "exit", "q"):
                        self.running = False
                        break
                    
                    elif cmd:
                        print("Неизвестная команда")
            
            except KeyboardInterrupt:
                print("\n👋 Завершение...")
                self.running = False
        
        else:
            # GUI режим
            self._create_ui()
            self.root.mainloop()
        
        print("🧬 CrimeaAI Meta Organism завершён")


def main():
    """Точка входа"""
    parser = argparse.ArgumentParser(
        description="CrimeaAI Meta Organism - Живое Цифровое Сознание"
    )
    parser.add_argument(
        '--voxels', '-v',
        type=int,
        default=50000,
        help='Количество вокселей в организме (по умолчанию 50000)'
    )
    parser.add_argument(
        '--backend', '-b',
        choices=['auto', 'open3d', 'plotly', 'matplotlib'],
        default='auto',
        help='Бэкенд визуализации'
    )
    parser.add_argument(
        '--headless',
        action='store_true',
        help='Консольный режим без UI'
    )
    
    args = parser.parse_args()
    
    app = CrimeaAIApplication(
        num_voxels=args.voxels,
        backend=args.backend
    )
    app.run(headless=args.headless)


if __name__ == "__main__":
    main()

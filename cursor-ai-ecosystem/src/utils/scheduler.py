"""
Планировщик задач на основе asyncio
"""

import asyncio
import time
from typing import Callable, Dict, List
from dataclasses import dataclass


@dataclass
class Task:
    """Задача для планировщика."""
    name: str
    function: Callable
    interval: float  # интервал в секундах
    last_run: float = 0.0
    run_count: int = 0
    enabled: bool = True


class Scheduler:
    """
    Асинхронный планировщик задач.
    Управляет периодическими задачами с разными интервалами.
    """
    
    def __init__(self):
        """Инициализация планировщика."""
        self.tasks: Dict[str, Task] = {}
        self.running = False
        self.start_time = time.time()
        
    def add_task(self, name: str, function: Callable, interval: float):
        """
        Добавление задачи в планировщик.
        
        Args:
            name: Имя задачи
            function: Функция для выполнения (может быть sync или async)
            interval: Интервал выполнения в секундах
        """
        task = Task(
            name=name,
            function=function,
            interval=interval,
            last_run=time.time()
        )
        self.tasks[name] = task
        print(f"[Scheduler] Добавлена задача '{name}' с интервалом {interval}s")
    
    def remove_task(self, name: str):
        """
        Удаление задачи из планировщика.
        
        Args:
            name: Имя задачи
        """
        if name in self.tasks:
            del self.tasks[name]
            print(f"[Scheduler] Удалена задача '{name}'")
    
    def enable_task(self, name: str):
        """Включение задачи."""
        if name in self.tasks:
            self.tasks[name].enabled = True
    
    def disable_task(self, name: str):
        """Отключение задачи."""
        if name in self.tasks:
            self.tasks[name].enabled = False
    
    async def _run_task(self, task: Task):
        """
        Выполнение задачи.
        
        Args:
            task: Задача для выполнения
        """
        try:
            # Проверка, является ли функция асинхронной
            if asyncio.iscoroutinefunction(task.function):
                await task.function()
            else:
                # Запуск синхронной функции в executor
                loop = asyncio.get_event_loop()
                await loop.run_in_executor(None, task.function)
            
            task.run_count += 1
            task.last_run = time.time()
            
        except Exception as e:
            print(f"[Scheduler] Ошибка в задаче '{task.name}': {e}")
    
    async def run(self):
        """
        Основной цикл планировщика.
        Непрерывно проверяет задачи и выполняет их по расписанию.
        """
        self.running = True
        print(f"[Scheduler] Запуск планировщика с {len(self.tasks)} задачами")
        
        while self.running:
            current_time = time.time()
            
            # Проверка каждой задачи
            tasks_to_run = []
            for task in self.tasks.values():
                if not task.enabled:
                    continue
                
                # Проверка, нужно ли выполнить задачу
                time_since_last_run = current_time - task.last_run
                if time_since_last_run >= task.interval:
                    tasks_to_run.append(task)
            
            # Выполнение задач параллельно
            if tasks_to_run:
                await asyncio.gather(*[self._run_task(task) for task in tasks_to_run])
            
            # Короткий сон для предотвращения busy-waiting
            await asyncio.sleep(0.01)
    
    def stop(self):
        """Остановка планировщика."""
        self.running = False
        print("[Scheduler] Остановка планировщика")
    
    def get_stats(self) -> Dict:
        """Получение статистики планировщика."""
        uptime = time.time() - self.start_time
        
        task_stats = []
        for name, task in self.tasks.items():
            task_stats.append({
                'name': name,
                'interval': task.interval,
                'run_count': task.run_count,
                'last_run_ago': time.time() - task.last_run,
                'enabled': task.enabled
            })
        
        return {
            'uptime': uptime,
            'total_tasks': len(self.tasks),
            'running': self.running,
            'tasks': task_stats
        }
    
    def print_stats(self):
        """Вывод статистики в консоль."""
        stats = self.get_stats()
        print(f"\n[Scheduler Stats]")
        print(f"  Uptime: {stats['uptime']:.1f}s")
        print(f"  Total tasks: {stats['total_tasks']}")
        print(f"  Running: {stats['running']}")
        print(f"\n  Tasks:")
        for task in stats['tasks']:
            status = "✓" if task['enabled'] else "✗"
            print(f"    {status} {task['name']}: {task['run_count']} runs, "
                  f"last {task['last_run_ago']:.1f}s ago (interval: {task['interval']}s)")


class TickerScheduler(Scheduler):
    """
    Специализированный планировщик с фиксированным тикером (например, 60 FPS).
    """
    
    def __init__(self, fps: int = 60):
        """
        Инициализация тикера.
        
        Args:
            fps: Частота обновления (кадров в секунду)
        """
        super().__init__()
        self.fps = fps
        self.dt = 1.0 / fps
        self.frame_count = 0
        self.actual_fps = 0.0
        self.last_fps_update = time.time()
        
    async def run(self):
        """
        Основной цикл тикера с фиксированной частотой.
        """
        self.running = True
        print(f"[TickerScheduler] Запуск с {self.fps} FPS")
        
        last_frame_time = time.time()
        
        while self.running:
            frame_start = time.time()
            
            # Вычисление фактического dt
            actual_dt = frame_start - last_frame_time
            last_frame_time = frame_start
            
            # Выполнение всех задач с передачей dt
            tasks_to_run = [task for task in self.tasks.values() if task.enabled]
            
            for task in tasks_to_run:
                try:
                    # Передача dt в функцию, если она его принимает
                    if asyncio.iscoroutinefunction(task.function):
                        # Попытка передать dt
                        try:
                            await task.function(actual_dt)
                        except TypeError:
                            # Функция не принимает аргументы
                            await task.function()
                    else:
                        loop = asyncio.get_event_loop()
                        try:
                            await loop.run_in_executor(None, task.function, actual_dt)
                        except TypeError:
                            await loop.run_in_executor(None, task.function)
                    
                    task.run_count += 1
                    task.last_run = frame_start
                    
                except Exception as e:
                    print(f"[TickerScheduler] Ошибка в задаче '{task.name}': {e}")
            
            self.frame_count += 1
            
            # Обновление FPS каждую секунду
            if frame_start - self.last_fps_update >= 1.0:
                self.actual_fps = self.frame_count / (frame_start - self.last_fps_update)
                self.frame_count = 0
                self.last_fps_update = frame_start
            
            # Расчет времени сна для поддержания FPS
            frame_time = time.time() - frame_start
            sleep_time = max(0.0, self.dt - frame_time)
            
            if sleep_time > 0:
                await asyncio.sleep(sleep_time)
    
    def get_stats(self) -> Dict:
        """Получение статистики тикера."""
        base_stats = super().get_stats()
        base_stats.update({
            'target_fps': self.fps,
            'actual_fps': self.actual_fps,
            'total_frames': sum(task.run_count for task in self.tasks.values())
        })
        return base_stats

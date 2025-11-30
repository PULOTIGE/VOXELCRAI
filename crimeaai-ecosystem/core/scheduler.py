"""
CrimeaAI Scheduler - Планировщик задач
======================================

Асинхронный планировщик для управления:
- Обновлениями нуклеотидов (60 FPS)
- Обновлениями вокселей (60 FPS)
- Поиском концептов (каждые 19 минут)
- Визуализацией (30-60 FPS)
- Сохранением состояния (каждые 5 минут)
"""

import asyncio
import time
from dataclasses import dataclass, field
from typing import Callable, Optional, List, Dict, Any, Coroutine
from enum import Enum
import threading
import queue


class TaskPriority(Enum):
    """Приоритеты задач"""
    CRITICAL = 0    # Критические (рендеринг)
    HIGH = 1        # Высокий (физика, эмоции)
    NORMAL = 2      # Нормальный (память, обучение)
    LOW = 3         # Низкий (поиск, сохранение)
    BACKGROUND = 4  # Фоновый (очистка, оптимизация)


@dataclass
class ScheduledTask:
    """Запланированная задача"""
    name: str
    callback: Callable
    interval: float  # Интервал в секундах
    priority: TaskPriority = TaskPriority.NORMAL
    enabled: bool = True
    
    # Статистика
    last_run: float = 0.0
    run_count: int = 0
    total_time: float = 0.0
    avg_time: float = 0.0
    
    # Ограничения
    max_duration: Optional[float] = None  # Макс. время выполнения
    
    def should_run(self, current_time: float) -> bool:
        """Проверка необходимости запуска"""
        if not self.enabled:
            return False
        return (current_time - self.last_run) >= self.interval
    
    def record_run(self, duration: float, current_time: float):
        """Запись статистики запуска"""
        self.last_run = current_time
        self.run_count += 1
        self.total_time += duration
        self.avg_time = self.total_time / self.run_count


class CrimeaScheduler:
    """
    Планировщик задач CrimeaAI
    
    Управляет асинхронным выполнением всех компонентов экосистемы.
    """
    
    def __init__(self):
        """Инициализация планировщика"""
        self.tasks: Dict[str, ScheduledTask] = {}
        self.running = False
        self.paused = False
        
        # Очередь событий
        self._event_queue: asyncio.Queue = None
        
        # Статистика
        self.start_time = 0.0
        self.total_ticks = 0
        self.fps = 0.0
        self._fps_samples: List[float] = []
        
        # Callbacks
        self._tick_callbacks: List[Callable[[int], None]] = []
        self._error_callbacks: List[Callable[[str, Exception], None]] = []
        
        # Основной цикл
        self._main_loop: Optional[asyncio.Task] = None
        self._task_runners: Dict[str, asyncio.Task] = {}
    
    def add_task(
        self,
        name: str,
        callback: Callable,
        interval: float,
        priority: TaskPriority = TaskPriority.NORMAL,
        max_duration: Optional[float] = None
    ) -> ScheduledTask:
        """
        Добавление задачи в планировщик
        
        Args:
            name: имя задачи
            callback: функция для выполнения (sync или async)
            interval: интервал в секундах
            priority: приоритет
            max_duration: макс. время выполнения
        
        Returns:
            Созданная задача
        """
        task = ScheduledTask(
            name=name,
            callback=callback,
            interval=interval,
            priority=priority,
            max_duration=max_duration
        )
        self.tasks[name] = task
        return task
    
    def remove_task(self, name: str):
        """Удаление задачи"""
        if name in self.tasks:
            del self.tasks[name]
            if name in self._task_runners:
                self._task_runners[name].cancel()
                del self._task_runners[name]
    
    def enable_task(self, name: str):
        """Включение задачи"""
        if name in self.tasks:
            self.tasks[name].enabled = True
    
    def disable_task(self, name: str):
        """Отключение задачи"""
        if name in self.tasks:
            self.tasks[name].enabled = False
    
    async def _run_task(self, task: ScheduledTask):
        """Запуск отдельной задачи в цикле"""
        while self.running:
            if self.paused or not task.enabled:
                await asyncio.sleep(0.1)
                continue
            
            current_time = time.time()
            
            if task.should_run(current_time):
                start = time.perf_counter()
                
                try:
                    # Вызываем callback
                    result = task.callback()
                    
                    # Если это корутина, ждём её
                    if asyncio.iscoroutine(result):
                        if task.max_duration:
                            result = await asyncio.wait_for(
                                result,
                                timeout=task.max_duration
                            )
                        else:
                            result = await result
                
                except asyncio.TimeoutError:
                    self._handle_error(task.name, TimeoutError(f"Task {task.name} timed out"))
                
                except Exception as e:
                    self._handle_error(task.name, e)
                
                finally:
                    duration = time.perf_counter() - start
                    task.record_run(duration, current_time)
            
            # Ждём до следующего запуска
            sleep_time = max(0.001, task.interval - (time.time() - task.last_run))
            await asyncio.sleep(sleep_time)
    
    async def _main_tick(self):
        """Главный тик планировщика"""
        last_time = time.time()
        
        while self.running:
            current_time = time.time()
            dt = current_time - last_time
            last_time = current_time
            
            if not self.paused:
                self.total_ticks += 1
                
                # Вычисляем FPS
                if dt > 0:
                    self._fps_samples.append(1.0 / dt)
                    if len(self._fps_samples) > 60:
                        self._fps_samples.pop(0)
                    self.fps = sum(self._fps_samples) / len(self._fps_samples)
                
                # Вызываем tick callbacks
                for callback in self._tick_callbacks:
                    try:
                        callback(self.total_ticks)
                    except Exception as e:
                        self._handle_error("tick_callback", e)
            
            # Ждём ~16мс (60 FPS)
            await asyncio.sleep(0.016)
    
    def _handle_error(self, task_name: str, error: Exception):
        """Обработка ошибки"""
        print(f"❌ Error in {task_name}: {error}")
        for callback in self._error_callbacks:
            try:
                callback(task_name, error)
            except:
                pass
    
    def on_tick(self, callback: Callable[[int], None]):
        """Регистрация callback на каждый тик"""
        self._tick_callbacks.append(callback)
    
    def on_error(self, callback: Callable[[str, Exception], None]):
        """Регистрация callback на ошибку"""
        self._error_callbacks.append(callback)
    
    async def start_async(self):
        """Асинхронный запуск планировщика"""
        if self.running:
            return
        
        self.running = True
        self.start_time = time.time()
        self._event_queue = asyncio.Queue()
        
        print("🚀 Запуск CrimeaAI Scheduler...")
        
        # Запускаем главный тик
        self._main_loop = asyncio.create_task(self._main_tick())
        
        # Запускаем все задачи
        for name, task in self.tasks.items():
            self._task_runners[name] = asyncio.create_task(self._run_task(task))
        
        print(f"✅ Запущено {len(self.tasks)} задач")
    
    async def stop_async(self):
        """Асинхронная остановка планировщика"""
        if not self.running:
            return
        
        print("🛑 Остановка CrimeaAI Scheduler...")
        self.running = False
        
        # Останавливаем все задачи
        for runner in self._task_runners.values():
            runner.cancel()
        
        if self._main_loop:
            self._main_loop.cancel()
        
        # Ждём завершения
        await asyncio.sleep(0.1)
        
        print("✅ Планировщик остановлен")
    
    def start(self):
        """Синхронный запуск (создаёт event loop)"""
        asyncio.run(self._run_forever())
    
    async def _run_forever(self):
        """Запуск навсегда"""
        await self.start_async()
        
        try:
            while self.running:
                await asyncio.sleep(1)
        except KeyboardInterrupt:
            pass
        finally:
            await self.stop_async()
    
    def pause(self):
        """Пауза планировщика"""
        self.paused = True
        print("⏸️ Планировщик на паузе")
    
    def resume(self):
        """Возобновление работы"""
        self.paused = False
        print("▶️ Планировщик возобновлён")
    
    def get_statistics(self) -> dict:
        """Получение статистики планировщика"""
        uptime = time.time() - self.start_time if self.start_time else 0
        
        task_stats = {}
        for name, task in self.tasks.items():
            task_stats[name] = {
                'enabled': task.enabled,
                'interval': task.interval,
                'priority': task.priority.name,
                'run_count': task.run_count,
                'avg_time_ms': task.avg_time * 1000,
                'total_time_s': task.total_time
            }
        
        return {
            'running': self.running,
            'paused': self.paused,
            'uptime_seconds': uptime,
            'total_ticks': self.total_ticks,
            'fps': self.fps,
            'task_count': len(self.tasks),
            'tasks': task_stats
        }


class ThreadedScheduler:
    """
    Многопоточный планировщик для CPU-интенсивных задач
    
    Использует отдельные потоки для тяжёлых вычислений,
    не блокируя основной event loop.
    """
    
    def __init__(self, num_workers: int = 4):
        """
        Создание планировщика
        
        Args:
            num_workers: количество рабочих потоков
        """
        self.num_workers = num_workers
        self.running = False
        
        # Очереди задач и результатов
        self._task_queue: queue.Queue = queue.Queue()
        self._result_queue: queue.Queue = queue.Queue()
        
        # Рабочие потоки
        self._workers: List[threading.Thread] = []
        
        # Статистика
        self.tasks_completed = 0
        self.tasks_pending = 0
    
    def _worker_loop(self):
        """Цикл рабочего потока"""
        while self.running:
            try:
                # Ждём задачу (с таймаутом для проверки running)
                task_data = self._task_queue.get(timeout=0.1)
                task_id, callback, args, kwargs = task_data
                
                try:
                    result = callback(*args, **kwargs)
                    self._result_queue.put((task_id, result, None))
                except Exception as e:
                    self._result_queue.put((task_id, None, e))
                
                self._task_queue.task_done()
                self.tasks_completed += 1
                self.tasks_pending -= 1
                
            except queue.Empty:
                continue
    
    def start(self):
        """Запуск планировщика"""
        if self.running:
            return
        
        self.running = True
        
        # Создаём рабочие потоки
        for i in range(self.num_workers):
            worker = threading.Thread(
                target=self._worker_loop,
                name=f"CrimeaWorker-{i}",
                daemon=True
            )
            worker.start()
            self._workers.append(worker)
        
        print(f"🔧 Запущен ThreadedScheduler с {self.num_workers} потоками")
    
    def stop(self):
        """Остановка планировщика"""
        self.running = False
        
        # Ждём завершения потоков
        for worker in self._workers:
            worker.join(timeout=1.0)
        
        self._workers.clear()
        print("🔧 ThreadedScheduler остановлен")
    
    def submit(
        self,
        callback: Callable,
        *args,
        task_id: Optional[str] = None,
        **kwargs
    ) -> str:
        """
        Отправка задачи на выполнение
        
        Args:
            callback: функция для выполнения
            *args: позиционные аргументы
            task_id: ID задачи
            **kwargs: именованные аргументы
        
        Returns:
            ID задачи
        """
        if task_id is None:
            task_id = f"task_{self.tasks_completed + self.tasks_pending}"
        
        self._task_queue.put((task_id, callback, args, kwargs))
        self.tasks_pending += 1
        
        return task_id
    
    def get_results(self) -> List[tuple]:
        """
        Получение завершённых результатов
        
        Returns:
            Список (task_id, result, error)
        """
        results = []
        while not self._result_queue.empty():
            try:
                results.append(self._result_queue.get_nowait())
            except queue.Empty:
                break
        return results
    
    def wait_all(self, timeout: Optional[float] = None):
        """Ожидание завершения всех задач"""
        self._task_queue.join()


def create_standard_scheduler() -> CrimeaScheduler:
    """
    Создание стандартного планировщика с предустановленными задачами
    
    Returns:
        Настроенный планировщик
    """
    scheduler = CrimeaScheduler()
    
    # Placeholder задачи (будут заменены реальными)
    scheduler.add_task(
        name="nucleotide_update",
        callback=lambda: None,
        interval=0.016,  # 60 FPS
        priority=TaskPriority.HIGH
    )
    
    scheduler.add_task(
        name="voxel_update",
        callback=lambda: None,
        interval=0.016,  # 60 FPS
        priority=TaskPriority.HIGH
    )
    
    scheduler.add_task(
        name="concept_search",
        callback=lambda: None,
        interval=19 * 60,  # 19 минут
        priority=TaskPriority.LOW
    )
    
    scheduler.add_task(
        name="auto_save",
        callback=lambda: None,
        interval=5 * 60,  # 5 минут
        priority=TaskPriority.BACKGROUND
    )
    
    return scheduler

"""
Kaif Engine - Движок эмоционального состояния
=============================================

Кайф (|dS/dt|) - это абсолютная величина производной энтропии,
характеризующая интенсивность переживаний системы.

Высокий кайф = система активно меняется, обучается, испытывает эмоции.
Низкий кайф = система в состоянии покоя или стагнации.
"""

import numpy as np
from dataclasses import dataclass, field
from typing import List, Optional, Callable, Dict, Tuple
from enum import Enum


class KaifState(Enum):
    """Состояния кайфа"""
    DORMANT = 'dormant'       # Покой (кайф < 0.1)
    CALM = 'calm'             # Спокойствие (0.1 - 0.3)
    ACTIVE = 'active'         # Активность (0.3 - 0.6)
    EXCITED = 'excited'       # Возбуждение (0.6 - 0.8)
    ECSTATIC = 'ecstatic'     # Экстаз (> 0.8)


def compute_entropy(distribution: np.ndarray) -> float:
    """
    Вычисление энтропии Шеннона
    
    Args:
        distribution: распределение вероятностей или значений
    
    Returns:
        Значение энтропии
    """
    # Нормализуем до вероятностей
    probs = np.abs(distribution)
    total = np.sum(probs)
    
    if total < 1e-10:
        return 0.0
    
    probs = probs / total
    
    # Избегаем log(0)
    probs = probs[probs > 1e-10]
    
    # Энтропия Шеннона
    entropy = -np.sum(probs * np.log2(probs))
    
    return float(entropy)


def compute_entropy_derivative(
    current: np.ndarray,
    previous: np.ndarray,
    dt: float
) -> float:
    """
    Вычисление производной энтропии (кайфа)
    
    Args:
        current: текущее состояние
        previous: предыдущее состояние
        dt: временной шаг
    
    Returns:
        |dS/dt| - абсолютная величина производной энтропии
    """
    if dt <= 0:
        return 0.0
    
    S_current = compute_entropy(current)
    S_previous = compute_entropy(previous)
    
    dS_dt = (S_current - S_previous) / dt
    
    return abs(dS_dt)


@dataclass
class KaifMetrics:
    """Метрики кайфа системы"""
    instant_kaif: float = 0.0          # Мгновенный кайф
    smoothed_kaif: float = 0.0         # Сглаженный кайф (экспоненциальный)
    max_kaif: float = 0.0              # Максимальный кайф за всё время
    avg_kaif: float = 0.0              # Средний кайф за окно
    kaif_variance: float = 0.0         # Вариация кайфа
    state: KaifState = KaifState.CALM  # Текущее состояние
    
    # История
    history: List[float] = field(default_factory=list)
    history_size: int = 100
    
    # Пороги для состояний
    thresholds: Dict[KaifState, float] = field(default_factory=lambda: {
        KaifState.DORMANT: 0.1,
        KaifState.CALM: 0.3,
        KaifState.ACTIVE: 0.6,
        KaifState.EXCITED: 0.8,
        KaifState.ECSTATIC: float('inf')
    })
    
    def update(self, new_kaif: float, smoothing: float = 0.1):
        """Обновление метрик"""
        self.instant_kaif = new_kaif
        
        # Экспоненциальное сглаживание
        self.smoothed_kaif = (1 - smoothing) * self.smoothed_kaif + smoothing * new_kaif
        
        # Обновляем максимум
        self.max_kaif = max(self.max_kaif, new_kaif)
        
        # Добавляем в историю
        self.history.append(new_kaif)
        if len(self.history) > self.history_size:
            self.history.pop(0)
        
        # Вычисляем статистику по истории
        if self.history:
            self.avg_kaif = np.mean(self.history)
            self.kaif_variance = np.var(self.history)
        
        # Определяем состояние
        self._update_state()
    
    def _update_state(self):
        """Обновление состояния на основе сглаженного кайфа"""
        kaif = self.smoothed_kaif
        
        if kaif < self.thresholds[KaifState.DORMANT]:
            self.state = KaifState.DORMANT
        elif kaif < self.thresholds[KaifState.CALM]:
            self.state = KaifState.CALM
        elif kaif < self.thresholds[KaifState.ACTIVE]:
            self.state = KaifState.ACTIVE
        elif kaif < self.thresholds[KaifState.EXCITED]:
            self.state = KaifState.EXCITED
        else:
            self.state = KaifState.ECSTATIC
    
    def get_trend(self) -> str:
        """Определение тренда кайфа"""
        if len(self.history) < 10:
            return "stable"
        
        recent = np.mean(self.history[-10:])
        older = np.mean(self.history[-20:-10]) if len(self.history) >= 20 else self.avg_kaif
        
        diff = recent - older
        
        if diff > 0.1:
            return "rising"
        elif diff < -0.1:
            return "falling"
        else:
            return "stable"


class KaifEngine:
    """
    Движок кайфа - управляет эмоциональным состоянием системы
    
    Отслеживает энтропию различных компонентов и вычисляет общий кайф.
    """
    
    def __init__(self):
        """Инициализация движка"""
        self.metrics = KaifMetrics()
        
        # Компоненты для отслеживания
        self._components: Dict[str, np.ndarray] = {}
        self._prev_components: Dict[str, np.ndarray] = {}
        
        # Веса компонентов
        self._weights: Dict[str, float] = {}
        
        # Callbacks на изменение состояния
        self._state_callbacks: List[Callable[[KaifState, KaifState], None]] = []
        
        # Последнее состояние для callbacks
        self._last_state = KaifState.CALM
        
        # Счётчик обновлений
        self.update_count = 0
    
    def register_component(
        self,
        name: str,
        initial_state: Optional[np.ndarray] = None,
        weight: float = 1.0
    ):
        """
        Регистрация компонента для отслеживания
        
        Args:
            name: имя компонента
            initial_state: начальное состояние
            weight: вес в общем кайфе
        """
        if initial_state is None:
            initial_state = np.zeros(64)
        
        self._components[name] = initial_state.copy()
        self._prev_components[name] = initial_state.copy()
        self._weights[name] = weight
    
    def update_component(self, name: str, new_state: np.ndarray):
        """
        Обновление состояния компонента
        
        Args:
            name: имя компонента
            new_state: новое состояние
        """
        if name not in self._components:
            self.register_component(name, new_state)
            return
        
        self._prev_components[name] = self._components[name].copy()
        self._components[name] = new_state.copy()
    
    def compute_total_kaif(self, dt: float) -> float:
        """
        Вычисление общего кайфа системы
        
        Args:
            dt: временной шаг
        
        Returns:
            Общий кайф (взвешенная сумма)
        """
        if dt <= 0 or not self._components:
            return 0.0
        
        total_kaif = 0.0
        total_weight = 0.0
        
        for name, current in self._components.items():
            prev = self._prev_components.get(name, current)
            weight = self._weights.get(name, 1.0)
            
            component_kaif = compute_entropy_derivative(current, prev, dt)
            total_kaif += component_kaif * weight
            total_weight += weight
        
        if total_weight > 0:
            total_kaif /= total_weight
        
        return total_kaif
    
    def update(self, dt: float):
        """
        Главное обновление движка
        
        Args:
            dt: временной шаг
        """
        self.update_count += 1
        
        # Вычисляем общий кайф
        kaif = self.compute_total_kaif(dt)
        
        # Обновляем метрики
        self.metrics.update(kaif)
        
        # Проверяем изменение состояния
        if self.metrics.state != self._last_state:
            for callback in self._state_callbacks:
                callback(self._last_state, self.metrics.state)
            self._last_state = self.metrics.state
    
    def on_state_change(self, callback: Callable[[KaifState, KaifState], None]):
        """
        Регистрация callback на изменение состояния
        
        Args:
            callback: функция (old_state, new_state)
        """
        self._state_callbacks.append(callback)
    
    def get_state(self) -> KaifState:
        """Получение текущего состояния кайфа"""
        return self.metrics.state
    
    def get_kaif(self) -> float:
        """Получение текущего значения кайфа"""
        return self.metrics.smoothed_kaif
    
    def get_component_kaifs(self, dt: float = 0.016) -> Dict[str, float]:
        """
        Получение кайфа по компонентам
        
        Args:
            dt: временной шаг
        
        Returns:
            Словарь {компонент: кайф}
        """
        result = {}
        
        for name, current in self._components.items():
            prev = self._prev_components.get(name, current)
            result[name] = compute_entropy_derivative(current, prev, dt)
        
        return result
    
    def get_statistics(self) -> dict:
        """Получение полной статистики"""
        return {
            'instant_kaif': self.metrics.instant_kaif,
            'smoothed_kaif': self.metrics.smoothed_kaif,
            'max_kaif': self.metrics.max_kaif,
            'avg_kaif': self.metrics.avg_kaif,
            'variance': self.metrics.kaif_variance,
            'state': self.metrics.state.value,
            'trend': self.metrics.get_trend(),
            'update_count': self.update_count,
            'components': list(self._components.keys())
        }
    
    def inject_stimulus(self, intensity: float = 1.0):
        """
        Инъекция стимула для увеличения кайфа
        
        Args:
            intensity: интенсивность стимула [0, 1]
        """
        for name in self._components:
            # Добавляем шум к компоненту
            noise = np.random.randn(*self._components[name].shape) * intensity * 0.5
            self._components[name] += noise.astype(self._components[name].dtype)
    
    def calm_down(self, rate: float = 0.1):
        """
        Успокоение системы
        
        Args:
            rate: скорость успокоения [0, 1]
        """
        for name in self._components:
            # Сглаживаем к среднему
            mean = np.mean(self._components[name])
            self._components[name] = (
                (1 - rate) * self._components[name] +
                rate * mean
            ).astype(self._components[name].dtype)


class AdaptiveKaifController:
    """
    Адаптивный контроллер кайфа
    
    Автоматически регулирует параметры системы для поддержания
    оптимального уровня кайфа.
    """
    
    def __init__(
        self,
        engine: KaifEngine,
        target_kaif: float = 0.5,
        tolerance: float = 0.1
    ):
        """
        Создание контроллера
        
        Args:
            engine: движок кайфа
            target_kaif: целевой уровень кайфа
            tolerance: допустимое отклонение
        """
        self.engine = engine
        self.target_kaif = target_kaif
        self.tolerance = tolerance
        
        # PID-параметры
        self.kp = 0.5  # Пропорциональный коэффициент
        self.ki = 0.1  # Интегральный коэффициент
        self.kd = 0.2  # Дифференциальный коэффициент
        
        # Внутреннее состояние
        self._integral = 0.0
        self._prev_error = 0.0
        
        # Действия
        self._actions: Dict[str, Callable[[float], None]] = {}
    
    def register_action(self, name: str, action: Callable[[float], None]):
        """
        Регистрация действия для регулирования
        
        Args:
            name: имя действия
            action: функция, принимающая силу регулирования [-1, 1]
        """
        self._actions[name] = action
    
    def update(self, dt: float) -> float:
        """
        Обновление контроллера
        
        Args:
            dt: временной шаг
        
        Returns:
            Сила регулирования [-1, 1]
        """
        current_kaif = self.engine.get_kaif()
        
        # Вычисляем ошибку
        error = self.target_kaif - current_kaif
        
        # PID-регулятор
        self._integral += error * dt
        derivative = (error - self._prev_error) / dt if dt > 0 else 0
        
        control = (
            self.kp * error +
            self.ki * self._integral +
            self.kd * derivative
        )
        
        # Ограничиваем
        control = np.clip(control, -1.0, 1.0)
        
        self._prev_error = error
        
        # Применяем действия если отклонение больше допустимого
        if abs(error) > self.tolerance:
            for action in self._actions.values():
                action(control)
        
        return float(control)
    
    def set_target(self, target: float):
        """Установка целевого кайфа"""
        self.target_kaif = np.clip(target, 0, 1)
        self._integral = 0.0  # Сбрасываем интегратор

"""
Модуль вокселя - микро-организма с памятью, эмоциями, мыслями и физикой (9 КБ)
"""

import numpy as np
import random
from typing import Dict, List
from scipy.stats import entropy


class Voxel:
    """
    Воксель - микро-организм (9 КБ) с внутренней структурой.
    
    Структура:
    - metadata: 512 байт (ID, позиция, состояние)
    - sensors: 1536 байт (сенсорные данные)
    - physics: 1024 байт (физическое состояние)
    - thoughts: 2048 байт (мысли и планы)
    - emotions: 2048 байт (эмоциональное состояние)
    - memory: 2048 байт (краткосрочная память)
    """
    
    def __init__(self, voxel_id: int = 0):
        """
        Инициализация вокселя.
        
        Args:
            voxel_id: Уникальный идентификатор
        """
        # Метаданные (512 байт)
        self.metadata = {
            'id': voxel_id,
            'position': np.array([0.0, 0.0, 0.0], dtype=np.float32),
            'age': 0.0,
            'alive': True,
            'generation': 0
        }
        
        # Сенсоры (1536 байт ~ 384 float32)
        self.sensors = np.random.randn(384).astype(np.float32) * 0.1
        
        # Физика (1024 байт ~ 256 float32)
        self.physics = {
            'velocity': np.zeros(3, dtype=np.float32),
            'acceleration': np.zeros(3, dtype=np.float32),
            'mass': 1.0,
            'energy': 100.0,
            'temperature': 37.0,
            'state_vector': np.random.randn(250).astype(np.float32) * 0.1
        }
        
        # Мысли (2048 байт ~ 512 float32)
        self.thoughts = np.random.randn(512).astype(np.float32) * 0.1
        
        # Эмоции (2048 байт ~ 512 float32)
        self.emotions = np.random.randn(512).astype(np.float32) * 0.1
        
        # Память (2048 байт ~ 512 float32)
        self.memory = np.random.randn(512).astype(np.float32) * 0.1
        
        # История для вычисления энтропии
        self.emotion_history = []
        self.max_history = 10
        
        # Система "кайфа"
        self.kaif = 0.0
        self.prev_entropy = 0.0
        
    def update(self, dt: float):
        """
        Обновление состояния вокселя.
        
        Args:
            dt: Временной шаг
        """
        self.metadata['age'] += dt
        
        if not self.metadata['alive']:
            return
        
        # 1. Обновление сенсоров (шум + внешние воздействия)
        self.sensors += np.random.randn(384).astype(np.float32) * 0.01 * dt
        self.sensors = np.clip(self.sensors, -1.0, 1.0)
        
        # 2. Обновление физики
        self._update_physics(dt)
        
        # 3. Мысли: интеграция сенсоров и памяти
        sensor_contribution = np.mean(self.sensors.reshape(-1, 384)[:1], axis=0)
        memory_contribution = self.memory[:384]
        self.thoughts[:384] = 0.7 * self.thoughts[:384] + 0.2 * sensor_contribution + 0.1 * memory_contribution
        self.thoughts[384:] += np.random.randn(128).astype(np.float32) * 0.01 * dt
        
        # 4. Эмоции: зависят от мыслей и физики
        thought_influence = np.mean(self.thoughts[:256])
        energy_influence = self.physics['energy'] / 100.0 - 1.0
        self.emotions[:256] = 0.8 * self.emotions[:256] + 0.15 * thought_influence + 0.05 * energy_influence
        self.emotions[256:] += np.random.randn(256).astype(np.float32) * 0.01 * dt
        
        # 5. Обновление памяти: сохранение мыслей
        self.memory = 0.95 * self.memory + 0.05 * self.thoughts[:512]
        
        # 6. Вычисление "кайфа" (производная энтропии)
        self._update_kaif(dt)
        
        # 7. Адаптация связей на основе "кайфа"
        if self.kaif > 0.1:
            # Усиление связей при высоком кайфе
            self.memory += 0.001 * self.kaif * self.thoughts[:512]
        elif self.kaif < -0.1:
            # Ослабление при низком кайфе
            self.memory *= 0.999
        
        # 8. Проверка энергии
        self.physics['energy'] -= 0.1 * dt
        if self.physics['energy'] <= 0:
            self.metadata['alive'] = False
    
    def _update_physics(self, dt: float):
        """Обновление физического состояния."""
        # Простая физика
        self.physics['velocity'] += self.physics['acceleration'] * dt
        self.metadata['position'] += self.physics['velocity'] * dt
        
        # Демпфирование
        self.physics['velocity'] *= 0.99
        self.physics['acceleration'] *= 0.95
        
        # Случайные силы
        self.physics['acceleration'] += np.random.randn(3).astype(np.float32) * 0.1 * dt
        
        # Температура
        self.physics['temperature'] += (37.0 - self.physics['temperature']) * 0.1 * dt
    
    def _update_kaif(self, dt: float):
        """
        Вычисление "кайфа" как |dS/dt| (абсолютная производная энтропии).
        """
        # Дискретизация эмоций для вычисления энтропии
        emotion_bins = np.histogram(self.emotions, bins=20, range=(-1, 1))[0]
        emotion_bins = emotion_bins + 1  # избежать log(0)
        emotion_bins = emotion_bins / np.sum(emotion_bins)  # нормализация
        
        current_entropy = entropy(emotion_bins)
        
        # Производная энтропии
        if self.prev_entropy > 0:
            d_entropy = (current_entropy - self.prev_entropy) / dt if dt > 0 else 0.0
            self.kaif = abs(d_entropy)
        
        self.prev_entropy = current_entropy
        
        # Сохранение истории
        self.emotion_history.append(current_entropy)
        if len(self.emotion_history) > self.max_history:
            self.emotion_history.pop(0)
    
    def sense(self, external_input: np.ndarray):
        """
        Получение внешних сенсорных данных.
        
        Args:
            external_input: Внешние данные
        """
        if len(external_input) == len(self.sensors):
            self.sensors = 0.7 * self.sensors + 0.3 * external_input
    
    def get_emotional_state(self) -> Dict[str, float]:
        """
        Получение упрощенного эмоционального состояния.
        
        Returns:
            Словарь с основными эмоциями
        """
        # Упрощенная модель: выделение основных эмоций
        joy = float(np.mean(self.emotions[:128]))
        sadness = float(np.mean(self.emotions[128:256]))
        anger = float(np.mean(self.emotions[256:384]))
        fear = float(np.mean(self.emotions[384:512]))
        
        return {
            'joy': joy,
            'sadness': sadness,
            'anger': anger,
            'fear': fear,
            'kaif': self.kaif,
            'entropy': self.prev_entropy
        }
    
    def get_state_summary(self) -> Dict:
        """Получение краткой сводки состояния."""
        return {
            'id': self.metadata['id'],
            'age': self.metadata['age'],
            'alive': self.metadata['alive'],
            'position': self.metadata['position'].tolist(),
            'energy': self.physics['energy'],
            'kaif': self.kaif,
            'emotions': self.get_emotional_state()
        }
    
    def to_bytes(self) -> bytes:
        """
        Сериализация в байты (9 КБ).
        
        Returns:
            Байтовое представление
        """
        # Упрощенная версия
        data = bytearray(9 * 1024)
        # В реальности нужна правильная упаковка всех полей
        return bytes(data)
    
    def __repr__(self) -> str:
        alive_str = "alive" if self.metadata['alive'] else "dead"
        return f"Voxel(id={self.metadata['id']}, age={self.metadata['age']:.1f}s, {alive_str}, kaif={self.kaif:.3f})"


class VoxelGrid:
    """Сетка вокселей для управления множеством организмов."""
    
    def __init__(self, num_voxels: int = 1000):
        """
        Инициализация сетки вокселей.
        
        Args:
            num_voxels: Количество вокселей
        """
        self.voxels: List[Voxel] = [Voxel(i) for i in range(num_voxels)]
        self.num_voxels = num_voxels
        self.total_updates = 0
        
    def update_all(self, dt: float):
        """
        Обновление всех вокселей.
        
        Args:
            dt: Временной шаг
        """
        for voxel in self.voxels:
            voxel.update(dt)
        self.total_updates += 1
    
    def get_alive_count(self) -> int:
        """Подсчет живых вокселей."""
        return sum(1 for v in self.voxels if v.metadata['alive'])
    
    def get_average_kaif(self) -> float:
        """Средний "кайф" по всем живым вокселям."""
        alive = [v for v in self.voxels if v.metadata['alive']]
        if not alive:
            return 0.0
        return sum(v.kaif for v in alive) / len(alive)
    
    def get_stats(self) -> Dict:
        """Получение статистики сетки."""
        alive = [v for v in self.voxels if v.metadata['alive']]
        if not alive:
            return {
                'alive_count': 0,
                'dead_count': self.num_voxels,
                'avg_kaif': 0.0,
                'avg_energy': 0.0,
                'avg_age': 0.0
            }
        
        return {
            'alive_count': len(alive),
            'dead_count': self.num_voxels - len(alive),
            'avg_kaif': sum(v.kaif for v in alive) / len(alive),
            'avg_energy': sum(v.physics['energy'] for v in alive) / len(alive),
            'avg_age': sum(v.metadata['age'] for v in alive) / len(alive),
            'total_updates': self.total_updates
        }

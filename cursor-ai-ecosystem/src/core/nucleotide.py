"""
Модуль нуклеотида - базовой ячейки памяти (256 байт)
"""

import numpy as np
import random
from typing import Dict, List


class Nucleotide:
    """
    Нуклеотид - базовая ячейка памяти (256 байт).
    
    Структура:
    - base: 1 байт (A, T, G, C)
    - epigenetic_tags: ~50 байт (метки метилирования, ацетилирования)
    - quantum_noise: 8 байт (float64)
    - histone_state: ~40 байт
    - semantic_vector: 156 байт (39 float32)
    """
    
    BASES = ['A', 'T', 'G', 'C']
    VECTOR_SIZE = 39  # 39 * 4 = 156 байт
    
    def __init__(self, 
                 base: str = None, 
                 epigenetic_tags: Dict[str, float] = None,
                 quantum_noise: float = None,
                 histone_state: Dict[str, float] = None,
                 semantic_vector: np.ndarray = None):
        """
        Инициализация нуклеотида.
        
        Args:
            base: Базовый символ (A, T, G, C)
            epigenetic_tags: Эпигенетические метки
            quantum_noise: Квантовый шум (псевдорандом)
            histone_state: Состояние гистонов
            semantic_vector: Семантический вектор
        """
        self.base = base if base else random.choice(self.BASES)
        
        # Эпигенетические метки (метилирование, ацетилирование и т.д.)
        self.epigenetic_tags = epigenetic_tags if epigenetic_tags else {
            'methylation': random.random(),
            'acetylation': random.random(),
            'phosphorylation': random.random(),
            'ubiquitination': random.random(),
        }
        
        # Квантовый шум для случайности
        self.quantum_noise = quantum_noise if quantum_noise is not None else random.random()
        
        # Состояние хроматина
        self.histone_state = histone_state if histone_state else {
            'h3k4me3': random.random(),  # активный промотор
            'h3k27ac': random.random(),  # активный энхансер
            'h3k27me3': random.random(), # репрессия Polycomb
            'h3k9me3': random.random(),  # гетерохроматин
        }
        
        # Семантический вектор (представление смысла)
        if semantic_vector is not None:
            self.semantic_vector = semantic_vector
        else:
            self.semantic_vector = np.random.randn(self.VECTOR_SIZE).astype(np.float32)
            self.semantic_vector /= np.linalg.norm(self.semantic_vector)  # нормализация
        
        # Время последнего обновления
        self.last_update = 0.0
        
    def update(self, dt: float, experience_vector: np.ndarray = None):
        """
        Обновление состояния нуклеотида.
        
        Args:
            dt: Временной шаг
            experience_vector: Вектор опыта для обновления семантики
        """
        self.last_update += dt
        
        # Обновление квантового шума
        self.quantum_noise = random.random()
        
        # Обновление эпигенетических меток (деградация и регенерация)
        for key in self.epigenetic_tags:
            # Деградация
            self.epigenetic_tags[key] *= 0.999
            # Случайная флуктуация
            self.epigenetic_tags[key] += (random.random() - 0.5) * 0.01 * dt
            # Ограничение [0, 1]
            self.epigenetic_tags[key] = max(0.0, min(1.0, self.epigenetic_tags[key]))
        
        # Обновление состояния гистонов
        for key in self.histone_state:
            self.histone_state[key] *= 0.998
            self.histone_state[key] += (random.random() - 0.5) * 0.01 * dt
            self.histone_state[key] = max(0.0, min(1.0, self.histone_state[key]))
        
        # Обновление семантического вектора
        if experience_vector is not None and len(experience_vector) == self.VECTOR_SIZE:
            # Сдвиг в сторону опыта (обучение)
            learning_rate = 0.001 * dt
            self.semantic_vector += learning_rate * experience_vector
            # Нормализация
            norm = np.linalg.norm(self.semantic_vector)
            if norm > 0:
                self.semantic_vector /= norm
    
    def get_expression_level(self) -> float:
        """
        Вычисление уровня экспрессии на основе эпигенетических меток.
        
        Returns:
            Уровень экспрессии [0, 1]
        """
        # Активные метки увеличивают экспрессию
        active = self.epigenetic_tags['acetylation'] + self.histone_state['h3k4me3']
        # Репрессивные метки уменьшают экспрессию
        repressive = self.epigenetic_tags['methylation'] + self.histone_state['h3k27me3']
        
        expression = (active - repressive + 1.0) / 2.0
        return max(0.0, min(1.0, expression))
    
    def similarity(self, other: 'Nucleotide') -> float:
        """
        Вычисление схожести с другим нуклеотидом.
        
        Args:
            other: Другой нуклеотид
            
        Returns:
            Косинусное сходство семантических векторов
        """
        return float(np.dot(self.semantic_vector, other.semantic_vector))
    
    def to_bytes(self) -> bytes:
        """
        Сериализация в байты (256 байт).
        
        Returns:
            Байтовое представление
        """
        # Это упрощенная версия, в реальности нужна точная упаковка
        data = bytearray(256)
        data[0] = ord(self.base)
        # Остальные данные можно упаковать через struct
        return bytes(data)
    
    def __repr__(self) -> str:
        return f"Nucleotide(base={self.base}, expr={self.get_expression_level():.2f})"


class NucleotidePool:
    """Пул нуклеотидов для эффективного управления."""
    
    def __init__(self, size: int = 1000000):
        """
        Инициализация пула нуклеотидов.
        
        Args:
            size: Количество нуклеотидов
        """
        self.size = size
        self.nucleotides: List[Nucleotide] = [Nucleotide() for _ in range(size)]
        self.total_updates = 0
        
    def update_all(self, dt: float):
        """
        Обновление всех нуклеотидов.
        
        Args:
            dt: Временной шаг
        """
        # Можно оптимизировать через multiprocessing
        for nuc in self.nucleotides:
            nuc.update(dt)
        self.total_updates += 1
    
    def get_random_nucleotide(self) -> Nucleotide:
        """Получить случайный нуклеотид."""
        return random.choice(self.nucleotides)
    
    def get_average_expression(self) -> float:
        """Вычислить среднюю экспрессию."""
        return sum(n.get_expression_level() for n in self.nucleotides) / self.size
    
    def get_stats(self) -> Dict[str, float]:
        """Получить статистику пула."""
        expressions = [n.get_expression_level() for n in self.nucleotides]
        return {
            'mean_expression': np.mean(expressions),
            'std_expression': np.std(expressions),
            'min_expression': np.min(expressions),
            'max_expression': np.max(expressions),
            'total_updates': self.total_updates
        }

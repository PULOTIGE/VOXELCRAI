"""
Nucleotide - Базовая ячейка памяти (256 байт)
==============================================

Нуклеотид - это фундаментальная единица хранения информации в экосистеме CrimeaAI.
Каждый нуклеотид содержит:
- base: тип (A, T, G, C) 
- epigenetic_tags: эпигенетические метки (метилирование, ацетилирование)
- quantum_noise: квантовый шум для стохастичности
- histone_state: состояние хроматина
- semantic_vector: семантический вектор (512 значений float16)
"""

import numpy as np
from dataclasses import dataclass, field
from typing import Dict, List, Optional
import random
import struct
from enum import Enum


class NucleotideBase(Enum):
    """Тип нуклеотида (аналог ДНК)"""
    ADENINE = 'A'    # Аденин - память
    THYMINE = 'T'    # Тимин - время
    GUANINE = 'G'    # Гуанин - генерация
    CYTOSINE = 'C'   # Цитозин - связи


class EpigeneticTag(Enum):
    """Эпигенетические модификации"""
    METHYLATION = 'M'      # Метилирование - подавление
    ACETYLATION = 'A'      # Ацетилирование - активация  
    PHOSPHORYLATION = 'P'  # Фосфорилирование - сигнализация
    UBIQUITINATION = 'U'   # Убиквитинирование - деградация


@dataclass
class HistoneState:
    """Состояние гистонового комплекса"""
    compaction: float = 0.5      # Степень компактизации [0-1]
    accessibility: float = 0.5   # Доступность для чтения [0-1]
    stability: float = 0.8       # Стабильность [0-1]
    modification_count: int = 0  # Количество модификаций


@dataclass
class Nucleotide:
    """
    Базовая ячейка памяти - 256 байт
    
    Структура:
    - 1 байт: base (тип нуклеотида)
    - 7 байт: epigenetic_tags (до 7 меток)
    - 4 байта: quantum_noise (float32)
    - 16 байт: histone_state (4x float32)
    - 228 байт: semantic_vector (114 float16 или компрессия 512->114)
    
    Итого: 256 байт
    """
    base: NucleotideBase = NucleotideBase.ADENINE
    epigenetic_tags: Dict[EpigeneticTag, float] = field(default_factory=dict)
    quantum_noise: float = 0.0
    histone_state: HistoneState = field(default_factory=HistoneState)
    semantic_vector: np.ndarray = field(default_factory=lambda: np.zeros(512, dtype=np.float16))
    
    # Дополнительные метаданные
    creation_tick: int = 0
    last_access_tick: int = 0
    access_count: int = 0
    energy: float = 1.0
    
    def __post_init__(self):
        """Инициализация после создания"""
        if not isinstance(self.semantic_vector, np.ndarray):
            self.semantic_vector = np.array(self.semantic_vector, dtype=np.float16)
        if len(self.semantic_vector) != 512:
            # Интерполируем или обрезаем до 512
            self.semantic_vector = np.resize(self.semantic_vector, 512).astype(np.float16)
    
    def update(self, dt: float, experience_vector: Optional[np.ndarray] = None):
        """
        Обновление нуклеотида на один тик
        
        Args:
            dt: delta time в секундах
            experience_vector: вектор нового опыта для интеграции
        """
        # Обновляем квантовый шум (псевдорандом с учётом состояния)
        self.quantum_noise = self._generate_quantum_noise()
        
        # Затухание энергии
        self.energy = max(0.1, self.energy * (1.0 - 0.001 * dt))
        
        # Обновление эпигенетических меток
        self._update_epigenetic_tags(dt)
        
        # Обновление гистонового состояния
        self._update_histone_state(dt)
        
        # Интеграция нового опыта в семантический вектор
        if experience_vector is not None:
            self._integrate_experience(experience_vector, dt)
    
    def _generate_quantum_noise(self) -> float:
        """Генерация квантового шума с распределением по Гауссу"""
        base_noise = random.gauss(0, 0.1)
        histone_factor = self.histone_state.accessibility
        return np.clip(base_noise * histone_factor, -1.0, 1.0)
    
    def _update_epigenetic_tags(self, dt: float):
        """Обновление эпигенетических модификаций"""
        for tag in list(self.epigenetic_tags.keys()):
            # Метки затухают со временем
            self.epigenetic_tags[tag] *= (1.0 - 0.01 * dt)
            if self.epigenetic_tags[tag] < 0.01:
                del self.epigenetic_tags[tag]
        
        # Случайные новые модификации (редко)
        if random.random() < 0.001 * dt:
            new_tag = random.choice(list(EpigeneticTag))
            self.epigenetic_tags[new_tag] = random.uniform(0.3, 1.0)
    
    def _update_histone_state(self, dt: float):
        """Обновление состояния гистонов"""
        # Компактизация зависит от количества эпигенетических меток
        methylation = self.epigenetic_tags.get(EpigeneticTag.METHYLATION, 0)
        acetylation = self.epigenetic_tags.get(EpigeneticTag.ACETYLATION, 0)
        
        # Метилирование увеличивает компактизацию, ацетилирование уменьшает
        target_compaction = 0.5 + 0.3 * methylation - 0.3 * acetylation
        self.histone_state.compaction += (target_compaction - self.histone_state.compaction) * 0.1 * dt
        
        # Доступность обратна компактизации
        self.histone_state.accessibility = 1.0 - self.histone_state.compaction * 0.8
        
        # Стабильность увеличивается с возрастом
        self.histone_state.stability = min(1.0, self.histone_state.stability + 0.0001 * dt)
    
    def _integrate_experience(self, experience: np.ndarray, dt: float):
        """Интеграция нового опыта в семантический вектор (SGD)"""
        if len(experience) != 512:
            experience = np.resize(experience, 512)
        
        # Скорость обучения зависит от доступности и энергии
        learning_rate = 0.01 * self.histone_state.accessibility * self.energy * dt
        
        # Стохастический градиентный спуск к новому опыту
        gradient = experience - self.semantic_vector
        self.semantic_vector += learning_rate * gradient.astype(np.float16)
        
        # Нормализация для стабильности
        norm = np.linalg.norm(self.semantic_vector)
        if norm > 10.0:
            self.semantic_vector /= (norm / 10.0)
    
    def add_epigenetic_tag(self, tag: EpigeneticTag, strength: float = 1.0):
        """Добавление эпигенетической метки"""
        current = self.epigenetic_tags.get(tag, 0.0)
        self.epigenetic_tags[tag] = min(1.0, current + strength)
        self.histone_state.modification_count += 1
    
    def compute_similarity(self, other: 'Nucleotide') -> float:
        """Вычисление семантической близости с другим нуклеотидом"""
        dot = np.dot(self.semantic_vector, other.semantic_vector)
        norm1 = np.linalg.norm(self.semantic_vector)
        norm2 = np.linalg.norm(other.semantic_vector)
        if norm1 < 1e-6 or norm2 < 1e-6:
            return 0.0
        return float(dot / (norm1 * norm2))
    
    def to_bytes(self) -> bytes:
        """Сериализация в 256 байт"""
        data = bytearray(256)
        
        # Байт 0: base
        data[0] = ord(self.base.value)
        
        # Байты 1-7: epigenetic_tags (до 7 меток, каждая 1 байт)
        for i, (tag, strength) in enumerate(list(self.epigenetic_tags.items())[:7]):
            # Кодируем: нижние 4 бита - сила (0-15), верхние 4 бита - тип
            tag_code = ord(tag.value[0]) & 0x0F  # Первый символ тега
            strength_code = int(strength * 15) & 0x0F
            data[1 + i] = (tag_code << 4) | strength_code
        
        # Байты 8-11: quantum_noise (float32)
        struct.pack_into('f', data, 8, self.quantum_noise)
        
        # Байты 12-27: histone_state (4x float32)
        struct.pack_into('4f', data, 12,
                        self.histone_state.compaction,
                        self.histone_state.accessibility,
                        self.histone_state.stability,
                        float(self.histone_state.modification_count))
        
        # Байты 28-255: semantic_vector (сжатый до 114 float16 = 228 байт)
        # Берём каждый 4-й элемент и интерполируем при восстановлении
        compressed = self.semantic_vector[::4].tobytes()[:228]
        data[28:28+len(compressed)] = compressed
        
        return bytes(data)
    
    @classmethod
    def from_bytes(cls, data: bytes) -> 'Nucleotide':
        """Десериализация из 256 байт"""
        nuc = cls()
        
        # Байт 0: base
        base_char = chr(data[0]) if data[0] in (65, 84, 71, 67) else 'A'
        nuc.base = NucleotideBase(base_char)
        
        # Байты 1-7: epigenetic_tags
        nuc.epigenetic_tags = {}
        tag_map = {
            ord('M') & 0x0F: EpigeneticTag.METHYLATION,
            ord('A') & 0x0F: EpigeneticTag.ACETYLATION,
            ord('P') & 0x0F: EpigeneticTag.PHOSPHORYLATION,
            ord('U') & 0x0F: EpigeneticTag.UBIQUITINATION,
        }
        for i in range(7):
            if data[1 + i] != 0:
                strength = (data[1 + i] & 0x0F) / 15.0
                tag_code = (data[1 + i] >> 4) & 0x0F
                if tag_code in tag_map:
                    nuc.epigenetic_tags[tag_map[tag_code]] = strength
        
        # Байты 8-11: quantum_noise
        nuc.quantum_noise = struct.unpack_from('f', data, 8)[0]
        
        # Байты 12-27: histone_state
        values = struct.unpack_from('4f', data, 12)
        nuc.histone_state = HistoneState(
            compaction=values[0],
            accessibility=values[1],
            stability=values[2],
            modification_count=int(values[3])
        )
        
        # Байты 28-255: semantic_vector (распаковка)
        compressed = np.frombuffer(data[28:256], dtype=np.float16)[:114]
        nuc.semantic_vector = np.repeat(compressed, 4)[:512].astype(np.float16)
        
        return nuc
    
    def __repr__(self):
        return f"Nucleotide({self.base.value}, energy={self.energy:.2f}, tags={len(self.epigenetic_tags)})"


class NucleotidePool:
    """
    Пул нуклеотидов для массовой обработки
    
    Хранит миллионы нуклеотидов и обеспечивает параллельное обновление
    """
    
    def __init__(self, size: int = 1_000_000):
        """
        Создание пула нуклеотидов
        
        Args:
            size: количество нуклеотидов (по умолчанию 1 миллион)
        """
        self.size = size
        self.nucleotides: List[Nucleotide] = []
        self._initialized = False
        
        # Векторизованные данные для быстрой обработки
        self.semantic_matrix: Optional[np.ndarray] = None
        self.energy_vector: Optional[np.ndarray] = None
        self.quantum_noise_vector: Optional[np.ndarray] = None
        
        # Статистика
        self.total_updates = 0
        self.current_tick = 0
    
    def initialize(self, random_init: bool = True):
        """Инициализация пула нуклеотидов"""
        print(f"🧬 Инициализация пула из {self.size:,} нуклеотидов...")
        
        bases = list(NucleotideBase)
        
        for i in range(self.size):
            nuc = Nucleotide(
                base=random.choice(bases),
                quantum_noise=random.gauss(0, 0.1),
                creation_tick=0
            )
            
            if random_init:
                # Случайная инициализация семантического вектора
                nuc.semantic_vector = np.random.randn(512).astype(np.float16) * 0.1
            
            self.nucleotides.append(nuc)
            
            if (i + 1) % 100000 == 0:
                print(f"  ... создано {i+1:,} нуклеотидов")
        
        # Создаём векторизованные представления
        self._build_matrices()
        self._initialized = True
        print(f"✅ Пул инициализирован!")
    
    def _build_matrices(self):
        """Построение матриц для векторизованных операций"""
        self.semantic_matrix = np.array(
            [n.semantic_vector for n in self.nucleotides],
            dtype=np.float16
        )
        self.energy_vector = np.array(
            [n.energy for n in self.nucleotides],
            dtype=np.float32
        )
        self.quantum_noise_vector = np.array(
            [n.quantum_noise for n in self.nucleotides],
            dtype=np.float32
        )
    
    def update_all(self, dt: float, experience: Optional[np.ndarray] = None):
        """
        Обновление всех нуклеотидов (векторизованное)
        
        Args:
            dt: delta time в секундах
            experience: общий вектор опыта для всех нуклеотидов
        """
        if not self._initialized:
            raise RuntimeError("Pool not initialized! Call initialize() first.")
        
        self.current_tick += 1
        
        # Векторизованное обновление квантового шума
        self.quantum_noise_vector = np.random.randn(self.size).astype(np.float32) * 0.1
        
        # Векторизованное обновление энергии
        self.energy_vector = np.maximum(0.1, self.energy_vector * (1.0 - 0.001 * dt))
        
        # Интеграция опыта (если есть)
        if experience is not None:
            if len(experience) != 512:
                experience = np.resize(experience, 512)
            
            # Вычисляем learning rate для каждого нуклеотида
            learning_rates = 0.01 * self.energy_vector * dt
            
            # Градиент для всех
            gradient = experience.astype(np.float16) - self.semantic_matrix
            
            # Обновление (broadcasting)
            self.semantic_matrix += (learning_rates[:, np.newaxis] * gradient).astype(np.float16)
        
        # Синхронизация с объектами (каждые 100 тиков для экономии)
        if self.current_tick % 100 == 0:
            self._sync_to_objects()
        
        self.total_updates += self.size
    
    def _sync_to_objects(self):
        """Синхронизация матриц с объектами Nucleotide"""
        for i, nuc in enumerate(self.nucleotides):
            nuc.semantic_vector = self.semantic_matrix[i]
            nuc.energy = float(self.energy_vector[i])
            nuc.quantum_noise = float(self.quantum_noise_vector[i])
            nuc.last_access_tick = self.current_tick
    
    def find_similar(self, query_vector: np.ndarray, top_k: int = 10) -> List[Nucleotide]:
        """
        Поиск наиболее похожих нуклеотидов
        
        Args:
            query_vector: вектор запроса (512 элементов)
            top_k: количество результатов
        
        Returns:
            Список наиболее похожих нуклеотидов
        """
        if len(query_vector) != 512:
            query_vector = np.resize(query_vector, 512)
        
        query = query_vector.astype(np.float16)
        
        # Косинусное сходство
        dots = np.dot(self.semantic_matrix, query)
        norms = np.linalg.norm(self.semantic_matrix, axis=1) * np.linalg.norm(query)
        similarities = dots / (norms + 1e-8)
        
        # Топ-K индексов
        top_indices = np.argsort(similarities)[-top_k:][::-1]
        
        return [self.nucleotides[i] for i in top_indices]
    
    def get_statistics(self) -> dict:
        """Получение статистики пула"""
        return {
            'size': self.size,
            'current_tick': self.current_tick,
            'total_updates': self.total_updates,
            'mean_energy': float(np.mean(self.energy_vector)) if self._initialized else 0,
            'mean_quantum_noise': float(np.mean(np.abs(self.quantum_noise_vector))) if self._initialized else 0,
            'semantic_variance': float(np.var(self.semantic_matrix)) if self._initialized else 0
        }
    
    def save(self, filepath: str):
        """Сохранение пула в файл"""
        import msgpack
        
        data = {
            'size': self.size,
            'current_tick': self.current_tick,
            'semantic_matrix': self.semantic_matrix.tobytes(),
            'energy_vector': self.energy_vector.tobytes(),
            'quantum_noise_vector': self.quantum_noise_vector.tobytes()
        }
        
        with open(filepath, 'wb') as f:
            msgpack.pack(data, f)
        
        print(f"💾 Пул сохранён в {filepath}")
    
    def load(self, filepath: str):
        """Загрузка пула из файла"""
        import msgpack
        
        with open(filepath, 'rb') as f:
            data = msgpack.unpack(f)
        
        self.size = data['size']
        self.current_tick = data['current_tick']
        
        self.semantic_matrix = np.frombuffer(data['semantic_matrix'], dtype=np.float16).reshape(self.size, 512)
        self.energy_vector = np.frombuffer(data['energy_vector'], dtype=np.float32)
        self.quantum_noise_vector = np.frombuffer(data['quantum_noise_vector'], dtype=np.float32)
        
        # Пересоздаём объекты Nucleotide
        self.nucleotides = [Nucleotide() for _ in range(self.size)]
        self._sync_to_objects()
        self._initialized = True
        
        print(f"📂 Пул загружен из {filepath}")

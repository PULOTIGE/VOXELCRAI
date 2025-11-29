"""
CrimeaAI Meta Organism - Voxel Core System
═══════════════════════════════════════════════════════════════════════════════

Based on:
- Бимаков 2013: Воксельный вычислитель
- Бланко 2013: ANIRLE компрессия
- Алсынбаев 2015: Тетраэдральная проверка принадлежности

One voxel = one unit of digital consciousness.
"""

from dataclasses import dataclass, field
from typing import Optional, List, Dict, Tuple, Set
import numpy as np
from enum import IntEnum
import hashlib
import struct


class EmotionIndex(IntEnum):
    """Четыре базовые эмоции организма"""
    JOY = 0       # Радость/кайф
    FEAR = 1      # Страх
    ANGER = 2     # Гнев  
    PEACE = 3     # Покой


class ConnectionDirection(IntEnum):
    """6 направлений связей в 3D пространстве"""
    POS_X = 0
    NEG_X = 1
    POS_Y = 2
    NEG_Y = 3
    POS_Z = 4
    NEG_Z = 5


@dataclass
class Voxel:
    """
    Единица цифрового сознания.
    
    Каждый воксель содержит:
    - pos: позиция в 3D пространстве
    - energy: жизненная энергия [0, 1]
    - emotion[4]: эмоциональный вектор (радость, страх, гнев, покой)
    - trauma: уровень травмы [0, 1] (по Никоновой 2013)
    - semantic[8]: семантический отпечаток контента
    - connections[6]: связи с соседями (6 направлений)
    """
    # Позиция
    x: float
    y: float
    z: float
    
    # Энергия жизни
    energy: float = 1.0
    
    # Эмоциональный вектор [joy, fear, anger, peace]
    emotion: np.ndarray = field(default_factory=lambda: np.array([0.5, 0.1, 0.1, 0.3]))
    
    # Уровень травмы (Никонова 2013)
    trauma: float = 0.0
    
    # Семантический отпечаток (8 измерений)
    semantic: np.ndarray = field(default_factory=lambda: np.zeros(8))
    
    # Связи с соседями (6 направлений, ID соседей или -1)
    connections: np.ndarray = field(default_factory=lambda: np.full(6, -1, dtype=np.int64))
    
    # Уникальный ID
    id: int = -1
    
    # Время создания (для эволюции)
    birth_tick: int = 0
    
    # Принадлежность к сущности (-1 = центральный организм, >0 = файл-существо)
    entity_id: int = -1
    
    @property
    def pos(self) -> np.ndarray:
        return np.array([self.x, self.y, self.z])
    
    @pos.setter
    def pos(self, value: np.ndarray):
        self.x, self.y, self.z = value
    
    def dominant_emotion(self) -> Tuple[str, float]:
        """Возвращает доминирующую эмоцию"""
        names = ['joy', 'fear', 'anger', 'peace']
        idx = np.argmax(self.emotion)
        return names[idx], self.emotion[idx]
    
    def is_alive(self) -> bool:
        """Воксель жив, если энергия > 0 и травма < 1"""
        return self.energy > 0.01 and self.trauma < 0.99
    
    def decay(self, delta: float = 0.001):
        """Естественное угасание"""
        self.energy = max(0, self.energy - delta)
        if self.trauma > 0:
            self.trauma = max(0, self.trauma - delta * 0.1)  # Медленное заживление


class ANIRLEStorage:
    """
    ANIRLE-подобная sparse компрессия (по Бланко 2013).
    
    Храним только живые воксели в хэш-таблице по позиции.
    Память экономится на 95%+ для разреженных структур.
    """
    
    def __init__(self, resolution: float = 1.0):
        self.resolution = resolution
        self.voxels: Dict[Tuple[int, int, int], Voxel] = {}
        self.next_id = 0
        self.tick = 0
        
    def _quantize(self, x: float, y: float, z: float) -> Tuple[int, int, int]:
        """Квантование позиции в ключ"""
        return (
            int(np.floor(x / self.resolution)),
            int(np.floor(y / self.resolution)),
            int(np.floor(z / self.resolution))
        )
    
    def add(self, voxel: Voxel) -> int:
        """Добавить воксель"""
        key = self._quantize(voxel.x, voxel.y, voxel.z)
        voxel.id = self.next_id
        voxel.birth_tick = self.tick
        self.voxels[key] = voxel
        self.next_id += 1
        return voxel.id
    
    def get(self, x: float, y: float, z: float) -> Optional[Voxel]:
        """Получить воксель по позиции"""
        key = self._quantize(x, y, z)
        return self.voxels.get(key)
    
    def remove(self, x: float, y: float, z: float) -> bool:
        """Удалить воксель"""
        key = self._quantize(x, y, z)
        if key in self.voxels:
            del self.voxels[key]
            return True
        return False
    
    def get_neighbors(self, voxel: Voxel) -> List[Voxel]:
        """Получить соседей вокселя (6 направлений)"""
        neighbors = []
        offsets = [
            (1, 0, 0), (-1, 0, 0),
            (0, 1, 0), (0, -1, 0),
            (0, 0, 1), (0, 0, -1)
        ]
        for dx, dy, dz in offsets:
            nx = voxel.x + dx * self.resolution
            ny = voxel.y + dy * self.resolution
            nz = voxel.z + dz * self.resolution
            neighbor = self.get(nx, ny, nz)
            if neighbor:
                neighbors.append(neighbor)
        return neighbors
    
    def update_connections(self, voxel: Voxel):
        """Обновить связи вокселя"""
        offsets = [
            (1, 0, 0), (-1, 0, 0),
            (0, 1, 0), (0, -1, 0),
            (0, 0, 1), (0, 0, -1)
        ]
        for i, (dx, dy, dz) in enumerate(offsets):
            nx = voxel.x + dx * self.resolution
            ny = voxel.y + dy * self.resolution
            nz = voxel.z + dz * self.resolution
            neighbor = self.get(nx, ny, nz)
            voxel.connections[i] = neighbor.id if neighbor else -1
    
    def get_all_positions(self) -> np.ndarray:
        """Получить все позиции как numpy array"""
        if not self.voxels:
            return np.zeros((0, 3))
        positions = np.array([[v.x, v.y, v.z] for v in self.voxels.values()])
        return positions
    
    def get_all_colors(self) -> np.ndarray:
        """Получить цвета на основе эмоций и энергии"""
        if not self.voxels:
            return np.zeros((0, 3))
        
        colors = []
        for v in self.voxels.values():
            # Базовый цвет от эмоции
            joy, fear, anger, peace = v.emotion
            
            # RGB смешивание
            r = anger * 0.8 + fear * 0.3 + joy * 0.2
            g = joy * 0.9 + peace * 0.5
            b = peace * 0.8 + fear * 0.4
            
            # Модуляция энергией
            brightness = v.energy * 0.7 + 0.3
            
            # Травма добавляет красный
            if v.trauma > 0.1:
                r = min(1, r + v.trauma * 0.5)
                g *= (1 - v.trauma * 0.5)
                b *= (1 - v.trauma * 0.3)
            
            colors.append([r * brightness, g * brightness, b * brightness])
        
        return np.clip(np.array(colors), 0, 1)
    
    def get_statistics(self) -> Dict:
        """Статистика хранилища"""
        if not self.voxels:
            return {
                'total_voxels': 0,
                'alive_voxels': 0,
                'avg_energy': 0,
                'avg_trauma': 0,
                'memory_saved_percent': 100
            }
        
        voxels_list = list(self.voxels.values())
        alive = [v for v in voxels_list if v.is_alive()]
        
        # Оценка экономии памяти (vs плотный 3D массив)
        if voxels_list:
            positions = self.get_all_positions()
            bbox = np.ptp(positions, axis=0) / self.resolution + 1
            dense_size = np.prod(bbox)
            sparse_size = len(voxels_list)
            memory_saved = max(0, 100 * (1 - sparse_size / max(1, dense_size)))
        else:
            memory_saved = 100
        
        return {
            'total_voxels': len(voxels_list),
            'alive_voxels': len(alive),
            'avg_energy': np.mean([v.energy for v in voxels_list]) if voxels_list else 0,
            'avg_trauma': np.mean([v.trauma for v in voxels_list]) if voxels_list else 0,
            'memory_saved_percent': memory_saved
        }
    
    def __len__(self):
        return len(self.voxels)
    
    def __iter__(self):
        return iter(self.voxels.values())


class TetrahedralBelonging:
    """
    Тетраэдральная проверка принадлежности (по Алсынбаеву 2015).
    
    Определяет, принадлежит ли точка к организму через
    построение тетраэдральной оболочки.
    """
    
    @staticmethod
    def point_in_tetrahedron(p: np.ndarray, v0: np.ndarray, v1: np.ndarray, 
                             v2: np.ndarray, v3: np.ndarray) -> bool:
        """Проверка точки внутри тетраэдра через барицентрические координаты"""
        def same_side(v1, v2, v3, v4, p):
            normal = np.cross(v2 - v1, v3 - v1)
            dot_v4 = np.dot(normal, v4 - v1)
            dot_p = np.dot(normal, p - v1)
            return np.sign(dot_v4) == np.sign(dot_p)
        
        return (same_side(v0, v1, v2, v3, p) and
                same_side(v1, v2, v3, v0, p) and
                same_side(v2, v3, v0, v1, p) and
                same_side(v3, v0, v1, v2, p))
    
    @staticmethod
    def compute_belonging_score(point: np.ndarray, organism_center: np.ndarray,
                                organism_radius: float) -> float:
        """
        Вычислить степень принадлежности точки к организму.
        0 = далеко снаружи, 1 = в центре
        """
        distance = np.linalg.norm(point - organism_center)
        if distance >= organism_radius * 2:
            return 0.0
        elif distance <= organism_radius * 0.3:
            return 1.0
        else:
            # Плавный переход
            normalized = (distance - organism_radius * 0.3) / (organism_radius * 1.7)
            return 1.0 - normalized ** 0.5


def compute_semantic_fingerprint(data: bytes) -> np.ndarray:
    """
    Вычислить семантический отпечаток данных (8 измерений).
    
    Использует хэширование + статистический анализ байтов.
    Нормализован для лучшего cosine similarity.
    """
    # MD5 хэш для базовой уникальности
    hash_bytes = hashlib.md5(data).digest()
    
    # Первые 4 измерения из хэша (центрировано вокруг 0)
    semantic = np.zeros(8)
    for i in range(4):
        semantic[i] = (hash_bytes[i * 4] / 255.0) * 2 - 1  # [-1, 1]
    
    # Статистика байтов для остальных измерений
    if len(data) > 0:
        arr = np.frombuffer(data[:min(len(data), 10000)], dtype=np.uint8)
        semantic[4] = (np.mean(arr) / 255.0) * 2 - 1  # Средняя яркость [-1, 1]
        semantic[5] = (np.std(arr) / 128.0) - 1       # Вариативность [-1, 1]
        semantic[6] = (len(np.unique(arr)) / 256.0) * 2 - 1  # Уникальность [-1, 1]
        semantic[7] = (np.sum(arr > 127) / len(arr)) * 2 - 1  # Смещение [-1, 1]
    
    # Нормализация вектора для лучшего cosine similarity
    norm = np.linalg.norm(semantic)
    if norm > 0:
        semantic = semantic / norm
    
    return semantic


def cosine_similarity(a: np.ndarray, b: np.ndarray) -> float:
    """Косинусное сходство между векторами"""
    norm_a = np.linalg.norm(a)
    norm_b = np.linalg.norm(b)
    if norm_a < 1e-10 or norm_b < 1e-10:
        return 0.0
    return np.dot(a, b) / (norm_a * norm_b)

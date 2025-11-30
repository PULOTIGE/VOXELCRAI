"""
LightPattern - Паттерн освещения (1 КБ)
=======================================

Паттерн освещения содержит:
- ID паттерна
- Прямое освещение (32 источника x 3 RGB)
- Непрямое освещение (32 источника x 3 RGB)
- Сферические гармоники (9 коэффициентов x 3 RGB)
- Свойства материала

Всего: 1024 байта (1 КБ)
"""

import numpy as np
from dataclasses import dataclass, field
from typing import List, Optional, Tuple, Dict
import struct
import math


@dataclass
class MaterialProperties:
    """Свойства материала"""
    roughness: float = 0.5
    metalness: float = 0.0
    albedo: Tuple[float, float, float] = (0.8, 0.8, 0.8)
    emission: Tuple[float, float, float] = (0.0, 0.0, 0.0)
    subsurface: float = 0.0
    ior: float = 1.5  # Index of refraction
    
    def to_array(self) -> np.ndarray:
        """Преобразование в массив"""
        return np.array([
            self.roughness, self.metalness,
            *self.albedo, *self.emission,
            self.subsurface, self.ior
        ], dtype=np.float32)
    
    @classmethod
    def from_array(cls, arr: np.ndarray) -> 'MaterialProperties':
        """Создание из массива"""
        return cls(
            roughness=float(arr[0]),
            metalness=float(arr[1]),
            albedo=(float(arr[2]), float(arr[3]), float(arr[4])),
            emission=(float(arr[5]), float(arr[6]), float(arr[7])),
            subsurface=float(arr[8]),
            ior=float(arr[9])
        )


@dataclass
class LightPattern:
    """
    Паттерн освещения - 1024 байта
    
    Структура:
    - 4 байта: pattern_id (uint32)
    - 4 байта: flags и метаданные
    - 384 байта: direct_lighting (32 x 3 float32)
    - 384 байта: indirect_lighting (32 x 3 float32)
    - 108 байт: sh_coeffs (9 x 3 float32)
    - 40 байт: material_props (10 float32)
    - 100 байт: дополнительные данные
    
    Итого: 1024 байта
    """
    
    pattern_id: int = 0
    flags: int = 0
    
    # Прямое освещение: 32 направленных источника света
    direct_lighting: np.ndarray = field(
        default_factory=lambda: np.zeros((32, 3), dtype=np.float32)
    )
    
    # Непрямое (отражённое) освещение
    indirect_lighting: np.ndarray = field(
        default_factory=lambda: np.zeros((32, 3), dtype=np.float32)
    )
    
    # Сферические гармоники для ambient occlusion
    sh_coeffs: np.ndarray = field(
        default_factory=lambda: np.zeros((9, 3), dtype=np.float32)
    )
    
    # Свойства материала
    material_props: MaterialProperties = field(default_factory=MaterialProperties)
    
    # Метаданные паттерна
    importance: float = 1.0
    last_used_tick: int = 0
    use_count: int = 0
    
    def get_feature_vector(self) -> np.ndarray:
        """
        Получение вектора признаков для сравнения паттернов
        
        Returns:
            Вектор признаков (float32)
        """
        return np.concatenate([
            self.direct_lighting.flatten(),
            self.indirect_lighting.flatten(),
            self.sh_coeffs.flatten(),
            self.material_props.to_array()
        ]).astype(np.float32)
    
    def apply_to_scene(self, scene_position: np.ndarray) -> Tuple[np.ndarray, np.ndarray]:
        """
        Применение паттерна к точке сцены
        
        Args:
            scene_position: позиция в сцене (x, y, z)
        
        Returns:
            (direct_color, indirect_color) - RGB цвета
        """
        # Вычисляем индекс направления на основе позиции
        direction = scene_position / (np.linalg.norm(scene_position) + 1e-8)
        
        # Выбираем ближайшее направление
        angles = []
        for i in range(32):
            theta = (i / 32) * 2 * math.pi
            dir_i = np.array([math.cos(theta), math.sin(theta), 0])
            angle = np.dot(direction[:2], dir_i[:2])
            angles.append(angle)
        
        idx = int(np.argmax(angles))
        
        # Интерполируем соседние значения
        idx_prev = (idx - 1) % 32
        idx_next = (idx + 1) % 32
        
        direct = 0.5 * self.direct_lighting[idx] + 0.25 * (
            self.direct_lighting[idx_prev] + self.direct_lighting[idx_next]
        )
        indirect = 0.5 * self.indirect_lighting[idx] + 0.25 * (
            self.indirect_lighting[idx_prev] + self.indirect_lighting[idx_next]
        )
        
        # Добавляем сферические гармоники
        sh_contribution = self._evaluate_sh(direction)
        indirect += sh_contribution
        
        return direct, indirect
    
    def _evaluate_sh(self, direction: np.ndarray) -> np.ndarray:
        """
        Вычисление сферических гармоник
        
        Args:
            direction: нормализованное направление
        
        Returns:
            RGB вклад от SH
        """
        x, y, z = direction[:3] if len(direction) >= 3 else (direction[0], direction[1], 0)
        
        # SH базисные функции (упрощённые)
        sh_basis = np.array([
            1.0,                           # Y00
            y, z, x,                       # Y1
            x*y, y*z, 3*z*z - 1, x*z, x*x - y*y  # Y2
        ], dtype=np.float32)
        
        # Свёртка с коэффициентами
        result = np.zeros(3, dtype=np.float32)
        for i in range(min(9, len(sh_basis))):
            result += self.sh_coeffs[i] * sh_basis[i]
        
        return np.maximum(0, result)
    
    def blend_with(self, other: 'LightPattern', weight: float) -> 'LightPattern':
        """
        Смешивание с другим паттерном
        
        Args:
            other: другой паттерн
            weight: вес другого паттерна [0, 1]
        
        Returns:
            Новый смешанный паттерн
        """
        w1 = 1.0 - weight
        w2 = weight
        
        result = LightPattern(pattern_id=-1)
        result.direct_lighting = w1 * self.direct_lighting + w2 * other.direct_lighting
        result.indirect_lighting = w1 * self.indirect_lighting + w2 * other.indirect_lighting
        result.sh_coeffs = w1 * self.sh_coeffs + w2 * other.sh_coeffs
        
        # Смешиваем свойства материала
        result.material_props = MaterialProperties(
            roughness=w1 * self.material_props.roughness + w2 * other.material_props.roughness,
            metalness=w1 * self.material_props.metalness + w2 * other.material_props.metalness,
            albedo=tuple(
                w1 * self.material_props.albedo[i] + w2 * other.material_props.albedo[i]
                for i in range(3)
            ),
            emission=tuple(
                w1 * self.material_props.emission[i] + w2 * other.material_props.emission[i]
                for i in range(3)
            ),
            subsurface=w1 * self.material_props.subsurface + w2 * other.material_props.subsurface,
            ior=w1 * self.material_props.ior + w2 * other.material_props.ior
        )
        
        return result
    
    def to_bytes(self) -> bytes:
        """Сериализация в 1024 байта"""
        data = bytearray(1024)
        
        # ID и флаги
        struct.pack_into('I', data, 0, self.pattern_id)
        struct.pack_into('I', data, 4, self.flags)
        
        # Direct lighting (384 байта)
        offset = 8
        direct_bytes = self.direct_lighting.astype(np.float32).tobytes()
        data[offset:offset+384] = direct_bytes
        
        # Indirect lighting (384 байта)
        offset = 392
        indirect_bytes = self.indirect_lighting.astype(np.float32).tobytes()
        data[offset:offset+384] = indirect_bytes
        
        # SH coefficients (108 байт)
        offset = 776
        sh_bytes = self.sh_coeffs.astype(np.float32).tobytes()
        data[offset:offset+108] = sh_bytes
        
        # Material properties (40 байт)
        offset = 884
        mat_bytes = self.material_props.to_array().tobytes()
        data[offset:offset+40] = mat_bytes
        
        # Метаданные (оставшиеся байты)
        struct.pack_into('f', data, 924, self.importance)
        struct.pack_into('I', data, 928, self.last_used_tick)
        struct.pack_into('I', data, 932, self.use_count)
        
        return bytes(data)
    
    @classmethod
    def from_bytes(cls, data: bytes) -> 'LightPattern':
        """Десериализация из 1024 байт"""
        pattern = cls()
        
        pattern.pattern_id = struct.unpack_from('I', data, 0)[0]
        pattern.flags = struct.unpack_from('I', data, 4)[0]
        
        # Direct lighting
        direct_data = np.frombuffer(data[8:392], dtype=np.float32)
        pattern.direct_lighting = direct_data.reshape(32, 3).copy()
        
        # Indirect lighting
        indirect_data = np.frombuffer(data[392:776], dtype=np.float32)
        pattern.indirect_lighting = indirect_data.reshape(32, 3).copy()
        
        # SH coefficients
        sh_data = np.frombuffer(data[776:884], dtype=np.float32)
        pattern.sh_coeffs = sh_data.reshape(9, 3).copy()
        
        # Material properties
        mat_data = np.frombuffer(data[884:924], dtype=np.float32)
        pattern.material_props = MaterialProperties.from_array(mat_data)
        
        # Метаданные
        pattern.importance = struct.unpack_from('f', data, 924)[0]
        pattern.last_used_tick = struct.unpack_from('I', data, 928)[0]
        pattern.use_count = struct.unpack_from('I', data, 932)[0]
        
        return pattern
    
    def __repr__(self):
        return f"LightPattern(id={self.pattern_id}, importance={self.importance:.2f}, uses={self.use_count})"


def cosine_similarity(a: np.ndarray, b: np.ndarray) -> float:
    """Косинусное сходство между векторами"""
    dot = np.dot(a.flatten(), b.flatten())
    norm = np.linalg.norm(a) * np.linalg.norm(b)
    return float(dot / (norm + 1e-8))


class PatternDatabase:
    """
    База данных паттернов освещения
    
    Обеспечивает:
    - Хранение и поиск паттернов
    - Кластеризацию похожих паттернов
    - Оптимальный выбор паттернов для сцены
    """
    
    def __init__(self, max_patterns: int = 10000):
        """
        Создание базы данных паттернов
        
        Args:
            max_patterns: максимальное количество паттернов
        """
        self.max_patterns = max_patterns
        self.patterns: Dict[int, LightPattern] = {}
        self.next_id = 0
        
        # Матрица признаков для быстрого поиска
        self._feature_matrix: Optional[np.ndarray] = None
        self._pattern_ids: List[int] = []
        
        # Статистика
        self.total_lookups = 0
        self.cache_hits = 0
    
    def add_pattern(self, pattern: LightPattern) -> int:
        """
        Добавление паттерна в базу
        
        Args:
            pattern: паттерн для добавления
        
        Returns:
            ID добавленного паттерна
        """
        if len(self.patterns) >= self.max_patterns:
            # Удаляем наименее используемый паттерн
            min_pattern = min(
                self.patterns.values(),
                key=lambda p: p.use_count * p.importance
            )
            del self.patterns[min_pattern.pattern_id]
        
        pattern.pattern_id = self.next_id
        self.patterns[self.next_id] = pattern
        self.next_id += 1
        
        # Обновляем матрицу признаков
        self._rebuild_feature_matrix()
        
        return pattern.pattern_id
    
    def _rebuild_feature_matrix(self):
        """Перестроение матрицы признаков"""
        if not self.patterns:
            self._feature_matrix = None
            self._pattern_ids = []
            return
        
        self._pattern_ids = list(self.patterns.keys())
        features = [self.patterns[pid].get_feature_vector() for pid in self._pattern_ids]
        self._feature_matrix = np.stack(features)
    
    def find_similar(
        self,
        scene_features: np.ndarray,
        top_k: int = 4
    ) -> List[Tuple[float, LightPattern]]:
        """
        Поиск похожих паттернов
        
        Args:
            scene_features: вектор признаков сцены
            top_k: количество результатов
        
        Returns:
            Список (similarity, pattern)
        """
        self.total_lookups += 1
        
        if self._feature_matrix is None or len(self.patterns) == 0:
            return []
        
        # Нормализуем запрос
        query = scene_features.flatten()
        if len(query) != self._feature_matrix.shape[1]:
            # Resize query to match feature size
            query = np.resize(query, self._feature_matrix.shape[1])
        
        # Косинусное сходство
        dots = np.dot(self._feature_matrix, query)
        norms = np.linalg.norm(self._feature_matrix, axis=1) * np.linalg.norm(query)
        similarities = dots / (norms + 1e-8)
        
        # Топ-K
        top_indices = np.argsort(similarities)[-top_k:][::-1]
        
        results = []
        for idx in top_indices:
            pid = self._pattern_ids[idx]
            pattern = self.patterns[pid]
            pattern.use_count += 1
            results.append((float(similarities[idx]), pattern))
        
        return results
    
    def blend_patterns(
        self,
        patterns: List[LightPattern],
        weights: List[float]
    ) -> LightPattern:
        """
        Смешивание нескольких паттернов
        
        Args:
            patterns: список паттернов
            weights: веса (должны суммироваться в 1)
        
        Returns:
            Смешанный паттерн
        """
        if not patterns:
            return LightPattern()
        
        if len(patterns) == 1:
            return patterns[0]
        
        # Нормализуем веса
        total_weight = sum(weights)
        weights = [w / total_weight for w in weights]
        
        # Смешиваем
        result = LightPattern(pattern_id=-1)
        result.direct_lighting = np.zeros((32, 3), dtype=np.float32)
        result.indirect_lighting = np.zeros((32, 3), dtype=np.float32)
        result.sh_coeffs = np.zeros((9, 3), dtype=np.float32)
        
        for w, p in zip(weights, patterns):
            result.direct_lighting += w * p.direct_lighting
            result.indirect_lighting += w * p.indirect_lighting
            result.sh_coeffs += w * p.sh_coeffs
        
        # Смешиваем материал
        roughness = sum(w * p.material_props.roughness for w, p in zip(weights, patterns))
        metalness = sum(w * p.material_props.metalness for w, p in zip(weights, patterns))
        albedo = tuple(
            sum(w * p.material_props.albedo[i] for w, p in zip(weights, patterns))
            for i in range(3)
        )
        
        result.material_props = MaterialProperties(
            roughness=roughness,
            metalness=metalness,
            albedo=albedo
        )
        
        return result
    
    def compute_scene_lighting(
        self,
        scene_features: np.ndarray,
        scene_position: np.ndarray
    ) -> Tuple[np.ndarray, np.ndarray]:
        """
        Вычисление освещения для точки сцены
        
        Args:
            scene_features: признаки сцены
            scene_position: позиция в сцене
        
        Returns:
            (direct_color, indirect_color)
        """
        # Находим похожие паттерны
        similar = self.find_similar(scene_features, top_k=4)
        
        if not similar:
            return np.zeros(3), np.zeros(3)
        
        # Вычисляем веса на основе сходства
        total_sim = sum(s for s, _ in similar)
        weights = [s / total_sim for s, _ in similar]
        patterns = [p for _, p in similar]
        
        # Смешиваем паттерны
        blended = self.blend_patterns(patterns, weights)
        
        # Применяем к позиции
        return blended.apply_to_scene(scene_position)
    
    def generate_random_patterns(self, count: int = 100):
        """Генерация случайных паттернов для тестирования"""
        for _ in range(count):
            pattern = LightPattern()
            
            # Случайное прямое освещение
            pattern.direct_lighting = np.random.rand(32, 3).astype(np.float32) * 0.5
            
            # Случайное непрямое освещение
            pattern.indirect_lighting = np.random.rand(32, 3).astype(np.float32) * 0.2
            
            # Случайные SH коэффициенты
            pattern.sh_coeffs = np.random.randn(9, 3).astype(np.float32) * 0.1
            
            # Случайный материал
            pattern.material_props = MaterialProperties(
                roughness=np.random.rand(),
                metalness=np.random.rand() * 0.5,
                albedo=tuple(np.random.rand(3))
            )
            
            pattern.importance = np.random.rand()
            
            self.add_pattern(pattern)
        
        print(f"✨ Сгенерировано {count} случайных паттернов")
    
    def save(self, filepath: str):
        """Сохранение базы в файл"""
        import msgpack
        
        data = {
            'max_patterns': self.max_patterns,
            'next_id': self.next_id,
            'patterns': {pid: p.to_bytes() for pid, p in self.patterns.items()}
        }
        
        with open(filepath, 'wb') as f:
            msgpack.pack(data, f)
        
        print(f"💡 База паттернов сохранена в {filepath}")
    
    def load(self, filepath: str):
        """Загрузка базы из файла"""
        import msgpack
        
        with open(filepath, 'rb') as f:
            data = msgpack.unpack(f)
        
        self.max_patterns = data['max_patterns']
        self.next_id = data['next_id']
        
        self.patterns = {}
        for pid, pdata in data['patterns'].items():
            self.patterns[int(pid)] = LightPattern.from_bytes(pdata)
        
        self._rebuild_feature_matrix()
        
        print(f"💡 База паттернов загружена из {filepath}")
    
    def get_statistics(self) -> dict:
        """Получение статистики базы"""
        return {
            'pattern_count': len(self.patterns),
            'max_patterns': self.max_patterns,
            'total_lookups': self.total_lookups,
            'avg_importance': np.mean([p.importance for p in self.patterns.values()]) if self.patterns else 0,
            'avg_use_count': np.mean([p.use_count for p in self.patterns.values()]) if self.patterns else 0
        }

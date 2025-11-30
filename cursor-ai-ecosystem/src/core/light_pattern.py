"""
Модуль паттернов освещения (1 КБ)
"""

import numpy as np
from typing import Dict, List, Tuple


class LightPattern:
    """
    Паттерн освещения (1 КБ).
    
    Структура:
    - pattern_id: 4 байта (int32)
    - direct_lighting: 384 байта (32 источника x 3 канала x 4 байта)
    - indirect_lighting: 384 байта (32 источника x 3 канала x 4 байта)
    - sh_coeffs: 108 байт (9 коэффициентов x 3 канала x 4 байта)
    - material_props: ~100 байт (различные свойства)
    - остальное: метаданные и резерв
    """
    
    def __init__(self, pattern_id: int = 0):
        """
        Инициализация паттерна освещения.
        
        Args:
            pattern_id: Уникальный ID паттерна
        """
        self.pattern_id = pattern_id
        
        # Прямое освещение (32 направленных источника, RGB)
        self.direct_lighting = np.zeros((32, 3), dtype=np.float32)
        
        # Непрямое (отраженное) освещение
        self.indirect_lighting = np.zeros((32, 3), dtype=np.float32)
        
        # Коэффициенты сферических гармоник (SH) - 9 коэффициентов для каждого канала RGB
        self.sh_coeffs = np.zeros((9, 3), dtype=np.float32)
        
        # Свойства материала
        self.material_props = {
            'roughness': 0.5,
            'metalness': 0.0,
            'albedo': np.array([0.8, 0.8, 0.8], dtype=np.float32),
            'emission': np.array([0.0, 0.0, 0.0], dtype=np.float32),
            'ambient_occlusion': 1.0,
            'clearcoat': 0.0,
        }
        
        # Метаданные
        self.timestamp = 0.0
        self.quality = 1.0
        self.usage_count = 0
        
    def set_direct_lighting(self, directions: np.ndarray, colors: np.ndarray):
        """
        Установка прямого освещения.
        
        Args:
            directions: Направления источников (N x 3)
            colors: Цвета источников (N x 3)
        """
        n = min(len(directions), 32)
        self.direct_lighting[:n] = colors[:n]
    
    def set_indirect_lighting(self, colors: np.ndarray):
        """
        Установка непрямого освещения.
        
        Args:
            colors: Цвета непрямого освещения (N x 3)
        """
        n = min(len(colors), 32)
        self.indirect_lighting[:n] = colors[:n]
    
    def compute_sh_from_lighting(self):
        """
        Вычисление коэффициентов сферических гармоник из освещения.
        Упрощенная версия - в реальности требуется проекция на базис SH.
        """
        # Упрощенная версия: усредняем освещение для каждого коэффициента
        total_lighting = self.direct_lighting + self.indirect_lighting
        
        # DC компонент (нулевой порядок)
        self.sh_coeffs[0] = np.mean(total_lighting, axis=0) * 0.282095  # Y_0^0
        
        # Первый порядок (3 коэффициента)
        self.sh_coeffs[1] = np.mean(total_lighting[:8], axis=0) * 0.488603  # Y_1^-1
        self.sh_coeffs[2] = np.mean(total_lighting[8:16], axis=0) * 0.488603  # Y_1^0
        self.sh_coeffs[3] = np.mean(total_lighting[16:24], axis=0) * 0.488603  # Y_1^1
        
        # Второй порядок (5 коэффициентов)
        for i in range(4, 9):
            start = (i - 4) * 5
            end = start + 5
            self.sh_coeffs[i] = np.mean(total_lighting[start:end], axis=0) * 0.315392
    
    def evaluate_sh(self, normal: np.ndarray) -> np.ndarray:
        """
        Вычисление освещения в точке по нормали используя SH.
        
        Args:
            normal: Нормаль (3D вектор)
            
        Returns:
            Цвет освещения (RGB)
        """
        # Базисные функции SH
        x, y, z = normal
        
        # Порядок 0
        Y_0_0 = 0.282095
        
        # Порядок 1
        Y_1_n1 = 0.488603 * y
        Y_1_0 = 0.488603 * z
        Y_1_1 = 0.488603 * x
        
        # Порядок 2 (упрощенно)
        Y_2_n2 = 1.092548 * x * y
        Y_2_n1 = 1.092548 * y * z
        Y_2_0 = 0.315392 * (3 * z * z - 1)
        Y_2_1 = 1.092548 * x * z
        Y_2_2 = 0.546274 * (x * x - y * y)
        
        sh_basis = np.array([Y_0_0, Y_1_n1, Y_1_0, Y_1_1, 
                            Y_2_n2, Y_2_n1, Y_2_0, Y_2_1, Y_2_2])
        
        # Сумма произведений коэффициентов и базисных функций
        color = np.sum(self.sh_coeffs * sh_basis[:, np.newaxis], axis=0)
        return np.clip(color, 0.0, 1.0)
    
    def get_feature_vector(self) -> np.ndarray:
        """
        Получение вектора признаков паттерна для сравнения.
        
        Returns:
            Вектор признаков
        """
        features = np.concatenate([
            self.direct_lighting.flatten(),
            self.indirect_lighting.flatten(),
            self.sh_coeffs.flatten(),
            self.material_props['albedo'],
            np.array([self.material_props['roughness'], 
                     self.material_props['metalness'],
                     self.material_props['ambient_occlusion']])
        ])
        return features
    
    def similarity(self, other: 'LightPattern') -> float:
        """
        Вычисление схожести с другим паттерном.
        
        Args:
            other: Другой паттерн
            
        Returns:
            Косинусное сходство
        """
        v1 = self.get_feature_vector()
        v2 = other.get_feature_vector()
        
        norm1 = np.linalg.norm(v1)
        norm2 = np.linalg.norm(v2)
        
        if norm1 == 0 or norm2 == 0:
            return 0.0
        
        return float(np.dot(v1, v2) / (norm1 * norm2))
    
    def to_bytes(self) -> bytes:
        """
        Сериализация в байты (1 КБ).
        
        Returns:
            Байтовое представление
        """
        data = bytearray(1024)
        # Упрощенная версия
        return bytes(data)
    
    def __repr__(self) -> str:
        return f"LightPattern(id={self.pattern_id}, quality={self.quality:.2f}, uses={self.usage_count})"


class LightPatternDatabase:
    """База данных паттернов освещения."""
    
    def __init__(self, max_patterns: int = 10000):
        """
        Инициализация базы паттернов.
        
        Args:
            max_patterns: Максимальное количество паттернов
        """
        self.patterns: List[LightPattern] = []
        self.max_patterns = max_patterns
        self.next_id = 0
        
    def add_pattern(self, pattern: LightPattern) -> int:
        """
        Добавление паттерна в базу.
        
        Args:
            pattern: Паттерн для добавления
            
        Returns:
            ID добавленного паттерна
        """
        if len(self.patterns) >= self.max_patterns:
            # Удаляем самый старый паттерн с низким качеством
            self.patterns.sort(key=lambda p: p.quality * p.usage_count)
            self.patterns.pop(0)
        
        pattern.pattern_id = self.next_id
        self.next_id += 1
        self.patterns.append(pattern)
        return pattern.pattern_id
    
    def find_similar_patterns(self, target: LightPattern, top_k: int = 4) -> List[Tuple[float, LightPattern]]:
        """
        Поиск похожих паттернов.
        
        Args:
            target: Целевой паттерн
            top_k: Количество лучших результатов
            
        Returns:
            Список (схожесть, паттерн)
        """
        if not self.patterns:
            return []
        
        similarities = []
        for pattern in self.patterns:
            sim = target.similarity(pattern)
            similarities.append((sim, pattern))
        
        # Сортировка по убыванию схожести
        similarities.sort(reverse=True, key=lambda x: x[0])
        
        return similarities[:top_k]
    
    def blend_patterns(self, patterns: List[LightPattern], weights: List[float]) -> LightPattern:
        """
        Смешивание паттернов с весами.
        
        Args:
            patterns: Список паттернов
            weights: Веса для каждого паттерна
            
        Returns:
            Смешанный паттерн
        """
        if not patterns:
            return LightPattern()
        
        # Нормализация весов
        weights = np.array(weights)
        weights /= np.sum(weights)
        
        result = LightPattern(self.next_id)
        self.next_id += 1
        
        # Смешивание освещения
        for w, p in zip(weights, patterns):
            result.direct_lighting += w * p.direct_lighting
            result.indirect_lighting += w * p.indirect_lighting
            result.sh_coeffs += w * p.sh_coeffs
        
        # Усреднение свойств материала
        result.material_props['roughness'] = sum(w * p.material_props['roughness'] 
                                                 for w, p in zip(weights, patterns))
        result.material_props['metalness'] = sum(w * p.material_props['metalness'] 
                                                 for w, p in zip(weights, patterns))
        result.material_props['albedo'] = sum(w * p.material_props['albedo'] 
                                              for w, p in zip(weights, patterns))
        
        return result
    
    def get_pattern_by_id(self, pattern_id: int) -> LightPattern:
        """Получение паттерна по ID."""
        for p in self.patterns:
            if p.pattern_id == pattern_id:
                p.usage_count += 1
                return p
        return None
    
    def get_stats(self) -> Dict:
        """Получение статистики базы."""
        if not self.patterns:
            return {
                'total_patterns': 0,
                'avg_quality': 0.0,
                'total_usage': 0
            }
        
        return {
            'total_patterns': len(self.patterns),
            'avg_quality': sum(p.quality for p in self.patterns) / len(self.patterns),
            'total_usage': sum(p.usage_count for p in self.patterns),
            'max_patterns': self.max_patterns
        }
    
    def __repr__(self) -> str:
        return f"LightPatternDatabase(patterns={len(self.patterns)}/{self.max_patterns})"

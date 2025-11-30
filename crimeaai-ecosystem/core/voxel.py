"""
Voxel - Микро-организм с памятью (9 КБ)
=======================================

Воксель - это автономная сущность с:
- 512 Б метаданных
- 1.5 КБ сенсоров  
- 1 КБ физики
- 2 КБ мыслей
- 2 КБ эмоций
- 2 КБ памяти

Всего: 9216 байт (9 КБ)
"""

import numpy as np
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple, Callable
from enum import Enum
import struct
import random
import math


class EmotionType(Enum):
    """Типы эмоций вокселя"""
    JOY = 'joy'              # Радость
    SADNESS = 'sadness'      # Грусть
    ANGER = 'anger'          # Гнев
    FEAR = 'fear'            # Страх
    SURPRISE = 'surprise'    # Удивление
    DISGUST = 'disgust'      # Отвращение
    CURIOSITY = 'curiosity'  # Любопытство
    PEACE = 'peace'          # Покой


class ThoughtType(Enum):
    """Типы мыслей"""
    OBSERVATION = 'obs'      # Наблюдение
    MEMORY = 'mem'           # Воспоминание
    PREDICTION = 'pred'      # Предсказание
    DECISION = 'dec'         # Решение
    CREATIVE = 'cre'         # Творчество


@dataclass
class VoxelMetadata:
    """Метаданные вокселя (512 байт)"""
    voxel_id: int = 0
    creation_time: float = 0.0
    position: Tuple[float, float, float] = (0.0, 0.0, 0.0)
    velocity: Tuple[float, float, float] = (0.0, 0.0, 0.0)
    mass: float = 1.0
    temperature: float = 300.0  # Кельвины
    age_ticks: int = 0
    parent_id: int = -1
    children_count: int = 0
    state: int = 0  # 0=normal, 1=active, 2=dormant, 3=dying
    
    # Биологические параметры
    health: float = 1.0
    energy: float = 1.0
    reproduction_ready: float = 0.0
    
    # Связи
    neighbor_ids: List[int] = field(default_factory=list)
    
    def to_bytes(self) -> bytes:
        """Сериализация в 512 байт"""
        data = bytearray(512)
        
        # Упаковка основных полей
        struct.pack_into('Q', data, 0, self.voxel_id)
        struct.pack_into('d', data, 8, self.creation_time)
        struct.pack_into('3f', data, 16, *self.position)
        struct.pack_into('3f', data, 28, *self.velocity)
        struct.pack_into('f', data, 40, self.mass)
        struct.pack_into('f', data, 44, self.temperature)
        struct.pack_into('Q', data, 48, self.age_ticks)
        struct.pack_into('q', data, 56, self.parent_id)
        struct.pack_into('I', data, 64, self.children_count)
        struct.pack_into('I', data, 68, self.state)
        struct.pack_into('3f', data, 72, self.health, self.energy, self.reproduction_ready)
        
        # Соседи (до 30 ID по 8 байт)
        for i, nid in enumerate(self.neighbor_ids[:30]):
            struct.pack_into('Q', data, 84 + i * 8, nid)
        
        return bytes(data)


@dataclass
class VoxelSensors:
    """Сенсоры вокселя (1536 байт = 1.5 КБ)"""
    
    # Визуальные сенсоры (32 направления x 3 RGB x 4 байта = 384 байта)
    visual_input: np.ndarray = field(default_factory=lambda: np.zeros((32, 3), dtype=np.float32))
    
    # Аудиосенсоры (64 частотных канала x 4 байта = 256 байт)
    audio_input: np.ndarray = field(default_factory=lambda: np.zeros(64, dtype=np.float32))
    
    # Тактильные сенсоры (6 сторон x 16 точек x 4 байта = 384 байта)
    tactile_input: np.ndarray = field(default_factory=lambda: np.zeros((6, 16), dtype=np.float32))
    
    # Химические сенсоры (32 вещества x 4 байта = 128 байт)
    chemical_input: np.ndarray = field(default_factory=lambda: np.zeros(32, dtype=np.float32))
    
    # Температурные сенсоры (8 направлений x 4 байта = 32 байта)
    thermal_input: np.ndarray = field(default_factory=lambda: np.zeros(8, dtype=np.float32))
    
    # Проприоцепция (внутреннее состояние, 64 параметра x 4 байта = 256 байта)
    proprioception: np.ndarray = field(default_factory=lambda: np.zeros(64, dtype=np.float32))
    
    # Специальные сенсоры (24 канала x 4 байта = 96 байт)
    special_input: np.ndarray = field(default_factory=lambda: np.zeros(24, dtype=np.float32))
    
    def get_combined_input(self) -> np.ndarray:
        """Объединённый сенсорный вход"""
        return np.concatenate([
            self.visual_input.flatten(),
            self.audio_input,
            self.tactile_input.flatten(),
            self.chemical_input,
            self.thermal_input,
            self.proprioception,
            self.special_input
        ])
    
    def update_from_environment(self, env_data: dict):
        """Обновление сенсоров из окружения"""
        if 'visual' in env_data:
            self.visual_input = np.array(env_data['visual'], dtype=np.float32).reshape(32, 3)
        if 'audio' in env_data:
            self.audio_input = np.array(env_data['audio'], dtype=np.float32)[:64]
        if 'temperature' in env_data:
            self.thermal_input = np.array(env_data['temperature'], dtype=np.float32)[:8]


@dataclass
class VoxelPhysics:
    """Физика вокселя (1024 байта = 1 КБ)"""
    
    # Состояние твёрдого тела
    angular_velocity: np.ndarray = field(default_factory=lambda: np.zeros(3, dtype=np.float32))
    orientation: np.ndarray = field(default_factory=lambda: np.array([1, 0, 0, 0], dtype=np.float32))  # Кватернион
    
    # Силы
    accumulated_force: np.ndarray = field(default_factory=lambda: np.zeros(3, dtype=np.float32))
    accumulated_torque: np.ndarray = field(default_factory=lambda: np.zeros(3, dtype=np.float32))
    
    # Материальные свойства
    elasticity: float = 0.5
    friction: float = 0.3
    density: float = 1.0
    
    # Деформация (32 точки x 3 координаты = 384 байта)
    deformation: np.ndarray = field(default_factory=lambda: np.zeros((32, 3), dtype=np.float32))
    
    # Внутренние напряжения (матрица 8x8 = 256 байт)
    stress_tensor: np.ndarray = field(default_factory=lambda: np.zeros((8, 8), dtype=np.float32))
    
    # Коллизии (16 точек контакта x 6 параметров = 384 байта)
    collision_points: np.ndarray = field(default_factory=lambda: np.zeros((16, 6), dtype=np.float32))
    
    def apply_force(self, force: np.ndarray, point: Optional[np.ndarray] = None):
        """Применение силы к вокселю"""
        self.accumulated_force += force
        if point is not None:
            # Вычисляем момент силы
            torque = np.cross(point, force)
            self.accumulated_torque += torque
    
    def integrate(self, dt: float, mass: float):
        """Интеграция физики за шаг времени"""
        # Линейное движение (возвращаем delta для обновления позиции)
        acceleration = self.accumulated_force / mass
        
        # Вращение
        angular_acceleration = self.accumulated_torque / mass
        self.angular_velocity += angular_acceleration * dt
        
        # Затухание
        self.angular_velocity *= 0.99
        
        # Сброс накопленных сил
        self.accumulated_force = np.zeros(3, dtype=np.float32)
        self.accumulated_torque = np.zeros(3, dtype=np.float32)
        
        return acceleration


@dataclass
class VoxelThoughts:
    """Мысли вокселя (2048 байт = 2 КБ)"""
    
    # Текущий фокус внимания (вектор 128 элементов = 512 байт)
    attention_focus: np.ndarray = field(default_factory=lambda: np.zeros(128, dtype=np.float32))
    
    # Рабочая память (256 элементов = 1024 байта)
    working_memory: np.ndarray = field(default_factory=lambda: np.zeros(256, dtype=np.float32))
    
    # Очередь мыслей (8 мыслей x 64 байта = 512 байт)
    thought_queue: List[Tuple[ThoughtType, np.ndarray]] = field(default_factory=list)
    
    # Текущий тип мышления
    current_mode: ThoughtType = ThoughtType.OBSERVATION
    
    # Глубина обработки
    processing_depth: int = 0
    max_depth: int = 5
    
    def add_thought(self, thought_type: ThoughtType, content: np.ndarray):
        """Добавление мысли в очередь"""
        if len(content) > 64:
            content = content[:64]
        self.thought_queue.append((thought_type, content.astype(np.float32)))
        if len(self.thought_queue) > 8:
            self.thought_queue.pop(0)
    
    def process_thoughts(self, sensory_input: np.ndarray, dt: float) -> np.ndarray:
        """Обработка мыслей и генерация выхода"""
        # Обновляем фокус внимания на основе сенсорного входа
        if len(sensory_input) > 128:
            sensory_input = sensory_input[:128]
        elif len(sensory_input) < 128:
            sensory_input = np.pad(sensory_input, (0, 128 - len(sensory_input)))
        
        # Плавное смещение фокуса
        self.attention_focus = 0.9 * self.attention_focus + 0.1 * sensory_input.astype(np.float32)
        
        # Обновление рабочей памяти
        attention_extended = np.tile(self.attention_focus, 2)
        self.working_memory = 0.95 * self.working_memory + 0.05 * attention_extended
        
        # Генерация мысли
        if random.random() < 0.1 * dt:
            new_thought = self.working_memory[:64] + np.random.randn(64).astype(np.float32) * 0.1
            self.add_thought(self.current_mode, new_thought)
        
        return self.attention_focus


@dataclass 
class VoxelEmotions:
    """Эмоции вокселя (2048 байт = 2 КБ)"""
    
    # Базовые эмоции (8 эмоций x 4 байта = 32 байта)
    base_emotions: Dict[EmotionType, float] = field(default_factory=lambda: {
        e: 0.5 for e in EmotionType
    })
    
    # Эмоциональный вектор (256 элементов = 1024 байта)
    emotion_vector: np.ndarray = field(default_factory=lambda: np.zeros(256, dtype=np.float32))
    
    # История эмоций (8 снимков x 32 байта = 256 байт)
    emotion_history: List[np.ndarray] = field(default_factory=list)
    
    # Модуляторы настроения (128 элементов = 512 байт)
    mood_modulators: np.ndarray = field(default_factory=lambda: np.zeros(128, dtype=np.float32))
    
    # Пороги реакций (64 значения = 256 байт)
    reaction_thresholds: np.ndarray = field(default_factory=lambda: np.ones(64, dtype=np.float32) * 0.5)
    
    # Кайф (производная энтропии)
    kaif: float = 0.0
    _prev_entropy: float = 0.0
    
    def update(self, thoughts: VoxelThoughts, sensors: VoxelSensors, dt: float):
        """Обновление эмоций на основе мыслей и сенсоров"""
        # Получаем комбинированный вход
        thought_input = thoughts.attention_focus
        sensor_input = sensors.get_combined_input()[:128]
        
        # Смешиваем входы
        combined = np.concatenate([thought_input, sensor_input])
        if len(combined) < 256:
            combined = np.pad(combined, (0, 256 - len(combined)))
        
        # Обновляем эмоциональный вектор
        self.emotion_vector = 0.8 * self.emotion_vector + 0.2 * combined.astype(np.float32)
        
        # Обновляем базовые эмоции
        self._update_base_emotions(dt)
        
        # Вычисляем кайф (|dS/dt|)
        self._compute_kaif(dt)
        
        # Сохраняем историю
        if len(self.emotion_history) >= 8:
            self.emotion_history.pop(0)
        self.emotion_history.append(self.emotion_vector[:32].copy())
    
    def _update_base_emotions(self, dt: float):
        """Обновление базовых эмоций"""
        # Вычисляем средние значения по секциям эмоционального вектора
        section_size = 32
        for i, emotion in enumerate(EmotionType):
            start = i * section_size
            end = min(start + section_size, 256)
            section = self.emotion_vector[start:end]
            
            # Значение эмоции - среднее абсолютное значение секции
            value = np.mean(np.abs(section))
            
            # Плавное изменение
            current = self.base_emotions[emotion]
            self.base_emotions[emotion] = current * 0.95 + value * 0.05
    
    def _compute_kaif(self, dt: float):
        """Вычисление кайфа как |dS/dt|"""
        # Энтропия эмоционального вектора
        probs = np.abs(self.emotion_vector) / (np.sum(np.abs(self.emotion_vector)) + 1e-8)
        entropy = -np.sum(probs * np.log(probs + 1e-8))
        
        # Производная энтропии
        if dt > 0:
            d_entropy = (entropy - self._prev_entropy) / dt
            self.kaif = abs(d_entropy)
        
        self._prev_entropy = entropy
    
    def get_dominant_emotion(self) -> Tuple[EmotionType, float]:
        """Получение доминирующей эмоции"""
        return max(self.base_emotions.items(), key=lambda x: x[1])


@dataclass
class VoxelMemory:
    """Память вокселя (2048 байт = 2 КБ)"""
    
    # Долговременная память (256 элементов = 1024 байта)
    long_term: np.ndarray = field(default_factory=lambda: np.zeros(256, dtype=np.float32))
    
    # Эпизодическая память (16 эпизодов x 64 байта = 1024 байта)
    episodes: List[np.ndarray] = field(default_factory=list)
    
    # Индекс для быстрого поиска
    memory_index: Dict[int, int] = field(default_factory=dict)
    
    # Счётчик записей
    write_count: int = 0
    
    def store(self, experience: np.ndarray, importance: float = 1.0):
        """Сохранение опыта в память"""
        if len(experience) > 64:
            experience = experience[:64]
        elif len(experience) < 64:
            experience = np.pad(experience, (0, 64 - len(experience)))
        
        experience = experience.astype(np.float32)
        
        # Интегрируем в долговременную память
        learning_rate = 0.1 * importance
        start_idx = (self.write_count % 4) * 64
        self.long_term[start_idx:start_idx+64] = (
            (1 - learning_rate) * self.long_term[start_idx:start_idx+64] +
            learning_rate * experience
        )
        
        # Сохраняем как эпизод
        if len(self.episodes) >= 16:
            # Удаляем наименее важный эпизод
            if len(self.episodes) > 0:
                min_idx = 0
                min_val = float('inf')
                for i, ep in enumerate(self.episodes):
                    val = np.sum(np.abs(ep))
                    if val < min_val:
                        min_val = val
                        min_idx = i
                self.episodes.pop(min_idx)
        
        self.episodes.append(experience)
        self.write_count += 1
    
    def recall(self, query: np.ndarray, top_k: int = 3) -> List[np.ndarray]:
        """Вспоминание по запросу"""
        if len(query) > 64:
            query = query[:64]
        elif len(query) < 64:
            query = np.pad(query, (0, 64 - len(query)))
        
        query = query.astype(np.float32)
        
        if not self.episodes:
            return []
        
        # Поиск по косинусному сходству
        similarities = []
        for i, ep in enumerate(self.episodes):
            dot = np.dot(query, ep)
            norm = np.linalg.norm(query) * np.linalg.norm(ep)
            sim = dot / (norm + 1e-8)
            similarities.append((sim, i))
        
        similarities.sort(reverse=True)
        return [self.episodes[i] for _, i in similarities[:top_k]]
    
    def consolidate(self):
        """Консолидация памяти (укрепление связей)"""
        if len(self.episodes) < 2:
            return
        
        # Усредняем похожие эпизоды
        for i in range(len(self.episodes)):
            for j in range(i + 1, len(self.episodes)):
                sim = np.dot(self.episodes[i], self.episodes[j])
                norm = np.linalg.norm(self.episodes[i]) * np.linalg.norm(self.episodes[j])
                if norm > 0 and sim / norm > 0.8:
                    # Объединяем похожие эпизоды
                    self.episodes[i] = 0.5 * (self.episodes[i] + self.episodes[j])


class Voxel:
    """
    Воксель - микро-организм (9 КБ)
    
    Автономная сущность с сенсорами, физикой, мыслями, эмоциями и памятью.
    """
    
    TOTAL_SIZE = 9216  # 9 KB
    
    def __init__(self, voxel_id: int = 0):
        """Создание вокселя"""
        self.metadata = VoxelMetadata(voxel_id=voxel_id)
        self.sensors = VoxelSensors()
        self.physics = VoxelPhysics()
        self.thoughts = VoxelThoughts()
        self.emotions = VoxelEmotions()
        self.memory = VoxelMemory()
        
        # Callbacks для взаимодействия
        self._update_callbacks: List[Callable] = []
    
    def update(self, dt: float, env_data: Optional[dict] = None):
        """
        Главный цикл обновления вокселя
        
        Args:
            dt: delta time в секундах
            env_data: данные окружающей среды
        """
        self.metadata.age_ticks += 1
        
        # 1. Обновляем сенсоры
        if env_data:
            self.sensors.update_from_environment(env_data)
        
        # 2. Обновляем физику
        acceleration = self.physics.integrate(dt, self.metadata.mass)
        pos = list(self.metadata.position)
        vel = list(self.metadata.velocity)
        for i in range(3):
            vel[i] += acceleration[i] * dt
            pos[i] += vel[i] * dt
        self.metadata.position = tuple(pos)
        self.metadata.velocity = tuple(vel)
        
        # 3. Обновляем мысли
        sensory_input = self.sensors.get_combined_input()
        self.thoughts.process_thoughts(sensory_input[:128], dt)
        
        # 4. Обновляем эмоции
        self.emotions.update(self.thoughts, self.sensors, dt)
        
        # 5. Консолидация памяти (редко)
        if self.metadata.age_ticks % 100 == 0:
            self.memory.consolidate()
        
        # 6. Сохраняем важный опыт в память
        if self.emotions.kaif > 0.5:
            experience = np.concatenate([
                self.thoughts.attention_focus[:32],
                np.array(list(self.emotions.base_emotions.values()))
            ])
            self.memory.store(experience, importance=self.emotions.kaif)
        
        # 7. Обновляем здоровье и энергию
        self._update_vitals(dt)
        
        # 8. Вызываем callbacks
        for callback in self._update_callbacks:
            callback(self)
    
    def _update_vitals(self, dt: float):
        """Обновление жизненных показателей"""
        # Энергия тратится
        self.metadata.energy -= 0.001 * dt
        
        # При высоком кайфе энергия восстанавливается
        if self.emotions.kaif > 0.7:
            self.metadata.energy += 0.002 * dt * self.emotions.kaif
        
        # Ограничиваем
        self.metadata.energy = np.clip(self.metadata.energy, 0, 1)
        
        # Здоровье зависит от энергии
        if self.metadata.energy < 0.1:
            self.metadata.health -= 0.001 * dt
        else:
            self.metadata.health += 0.0001 * dt
        
        self.metadata.health = np.clip(self.metadata.health, 0, 1)
        
        # Готовность к размножению
        if self.metadata.health > 0.8 and self.metadata.energy > 0.8:
            self.metadata.reproduction_ready += 0.0001 * dt
    
    def receive_stimulus(self, stimulus_type: str, data: np.ndarray):
        """Получение внешнего стимула"""
        if stimulus_type == 'visual':
            self.sensors.visual_input = data.reshape(32, 3).astype(np.float32)
        elif stimulus_type == 'audio':
            self.sensors.audio_input = data[:64].astype(np.float32)
        elif stimulus_type == 'chemical':
            self.sensors.chemical_input = data[:32].astype(np.float32)
        
        # Сразу обновляем мысли
        sensory_input = self.sensors.get_combined_input()
        self.thoughts.process_thoughts(sensory_input[:128], 0.016)
    
    def get_kaif(self) -> float:
        """Получение текущего уровня кайфа"""
        return self.emotions.kaif
    
    def get_state(self) -> dict:
        """Получение полного состояния вокселя"""
        dom_emotion, dom_value = self.emotions.get_dominant_emotion()
        return {
            'id': self.metadata.voxel_id,
            'position': self.metadata.position,
            'velocity': self.metadata.velocity,
            'health': self.metadata.health,
            'energy': self.metadata.energy,
            'age': self.metadata.age_ticks,
            'kaif': self.emotions.kaif,
            'dominant_emotion': dom_emotion.value,
            'dominant_emotion_value': dom_value,
            'thought_mode': self.thoughts.current_mode.value,
            'memory_episodes': len(self.memory.episodes)
        }
    
    def add_update_callback(self, callback: Callable):
        """Добавление callback на обновление"""
        self._update_callbacks.append(callback)
    
    def to_bytes(self) -> bytes:
        """Сериализация в 9216 байт"""
        data = bytearray(self.TOTAL_SIZE)
        
        # Метаданные (512 байт)
        data[0:512] = self.metadata.to_bytes()
        
        # Сенсоры (1536 байт)
        combined_sensors = self.sensors.get_combined_input()[:384]
        sensor_bytes = combined_sensors.astype(np.float32).tobytes()
        data[512:512+len(sensor_bytes)] = sensor_bytes
        
        # Физика (1024 байта)
        physics_data = np.concatenate([
            self.physics.angular_velocity,
            self.physics.orientation,
            self.physics.deformation.flatten()[:200]
        ]).astype(np.float32).tobytes()
        data[2048:2048+len(physics_data)] = physics_data
        
        # Мысли (2048 байт)
        thoughts_data = np.concatenate([
            self.thoughts.attention_focus,
            self.thoughts.working_memory
        ]).astype(np.float32).tobytes()
        data[3072:3072+len(thoughts_data)] = thoughts_data[:2048]
        
        # Эмоции (2048 байт)
        emotions_data = self.emotions.emotion_vector.astype(np.float32).tobytes()
        data[5120:5120+len(emotions_data)] = emotions_data[:2048]
        
        # Память (2048 байт)
        memory_data = self.memory.long_term.astype(np.float32).tobytes()
        data[7168:7168+len(memory_data)] = memory_data
        
        return bytes(data)
    
    @classmethod
    def from_bytes(cls, data: bytes) -> 'Voxel':
        """Десериализация из 9216 байт"""
        voxel = cls()
        # Здесь была бы полная десериализация
        # Упрощённая версия
        return voxel
    
    def __repr__(self):
        dom_emotion, _ = self.emotions.get_dominant_emotion()
        return f"Voxel(id={self.metadata.voxel_id}, health={self.metadata.health:.2f}, kaif={self.emotions.kaif:.3f}, emotion={dom_emotion.value})"


class VoxelWorld:
    """
    Мир вокселей - контейнер для управления множеством вокселей
    """
    
    def __init__(self, max_voxels: int = 1000):
        """Создание мира вокселей"""
        self.max_voxels = max_voxels
        self.voxels: Dict[int, Voxel] = {}
        self.next_id = 0
        self.current_tick = 0
        
        # Статистика
        self.total_kaif = 0.0
        self.avg_health = 1.0
        self.avg_energy = 1.0
    
    def spawn_voxel(self, position: Tuple[float, float, float] = (0, 0, 0)) -> Voxel:
        """Создание нового вокселя"""
        if len(self.voxels) >= self.max_voxels:
            # Удаляем самый слабый воксель
            weakest = min(self.voxels.values(), key=lambda v: v.metadata.health)
            del self.voxels[weakest.metadata.voxel_id]
        
        voxel = Voxel(voxel_id=self.next_id)
        voxel.metadata.position = position
        voxel.metadata.creation_time = self.current_tick
        
        self.voxels[self.next_id] = voxel
        self.next_id += 1
        
        return voxel
    
    def update(self, dt: float, global_env: Optional[dict] = None):
        """Обновление всего мира"""
        self.current_tick += 1
        
        total_kaif = 0.0
        total_health = 0.0
        total_energy = 0.0
        
        dead_voxels = []
        
        for voxel_id, voxel in self.voxels.items():
            # Создаём локальное окружение для вокселя
            env_data = self._get_local_environment(voxel, global_env)
            
            # Обновляем воксель
            voxel.update(dt, env_data)
            
            # Собираем статистику
            total_kaif += voxel.emotions.kaif
            total_health += voxel.metadata.health
            total_energy += voxel.metadata.energy
            
            # Проверяем смерть
            if voxel.metadata.health <= 0:
                dead_voxels.append(voxel_id)
        
        # Удаляем мёртвые воксели
        for vid in dead_voxels:
            del self.voxels[vid]
        
        # Обновляем статистику
        n = len(self.voxels)
        if n > 0:
            self.total_kaif = total_kaif
            self.avg_health = total_health / n
            self.avg_energy = total_energy / n
    
    def _get_local_environment(self, voxel: Voxel, global_env: Optional[dict]) -> dict:
        """Получение локального окружения для вокселя"""
        env = global_env.copy() if global_env else {}
        
        # Добавляем информацию о соседях
        neighbors = self._find_neighbors(voxel, radius=10.0)
        
        if neighbors:
            # Среднее визуальное поле от соседей
            visual = np.zeros((32, 3), dtype=np.float32)
            for n in neighbors:
                direction = np.array(n.metadata.position) - np.array(voxel.metadata.position)
                direction_idx = int(np.argmax(np.abs(direction))) % 32
                visual[direction_idx] = [1, 1, 1]  # Сосед виден
            env['visual'] = visual
        
        return env
    
    def _find_neighbors(self, voxel: Voxel, radius: float) -> List[Voxel]:
        """Поиск соседей в радиусе"""
        neighbors = []
        pos = np.array(voxel.metadata.position)
        
        for other in self.voxels.values():
            if other.metadata.voxel_id == voxel.metadata.voxel_id:
                continue
            
            other_pos = np.array(other.metadata.position)
            dist = np.linalg.norm(pos - other_pos)
            
            if dist <= radius:
                neighbors.append(other)
        
        return neighbors
    
    def get_statistics(self) -> dict:
        """Получение статистики мира"""
        return {
            'voxel_count': len(self.voxels),
            'current_tick': self.current_tick,
            'total_kaif': self.total_kaif,
            'avg_health': self.avg_health,
            'avg_energy': self.avg_energy,
            'emotion_distribution': self._get_emotion_distribution()
        }
    
    def _get_emotion_distribution(self) -> Dict[str, float]:
        """Распределение эмоций в мире"""
        dist = {e.value: 0.0 for e in EmotionType}
        n = len(self.voxels)
        
        if n == 0:
            return dist
        
        for voxel in self.voxels.values():
            for emotion, value in voxel.emotions.base_emotions.items():
                dist[emotion.value] += value
        
        return {k: v / n for k, v in dist.items()}
    
    def save(self, filepath: str):
        """Сохранение мира"""
        import msgpack
        
        data = {
            'max_voxels': self.max_voxels,
            'next_id': self.next_id,
            'current_tick': self.current_tick,
            'voxels': {vid: v.to_bytes() for vid, v in self.voxels.items()}
        }
        
        with open(filepath, 'wb') as f:
            msgpack.pack(data, f)
        
        print(f"🌍 Мир сохранён в {filepath}")
    
    def load(self, filepath: str):
        """Загрузка мира"""
        import msgpack
        
        with open(filepath, 'rb') as f:
            data = msgpack.unpack(f)
        
        self.max_voxels = data['max_voxels']
        self.next_id = data['next_id']
        self.current_tick = data['current_tick']
        
        self.voxels = {}
        for vid, vdata in data['voxels'].items():
            self.voxels[int(vid)] = Voxel.from_bytes(vdata)
        
        print(f"🌍 Мир загружен из {filepath}")

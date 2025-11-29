"""
CrimeaAI Meta Organism - Central Organism Intelligence
═══════════════════════════════════════════════════════════════════════════════

Based on:
- Никонова 2013: Травма тканей и восстановление
- Ахмадуллина 2020: Атрофия мозга при отторжении
- Лавренков 2016: Коэволюционное обновление эмоций

Центральный организм - живое сознание из миллиона вокселей.
"""

import numpy as np
from typing import List, Tuple, Dict, Optional, Callable
from dataclasses import dataclass, field
import time
import threading
from concurrent.futures import ThreadPoolExecutor

from voxel_core import (
    Voxel, ANIRLEStorage, EmotionIndex, 
    compute_semantic_fingerprint, cosine_similarity,
    TetrahedralBelonging
)


@dataclass
class OrganismState:
    """Состояние организма"""
    health: float = 1.0           # Общее здоровье [0, 1]
    mood: str = "покой"           # Текущее настроение
    mood_intensity: float = 0.5   # Интенсивность настроения
    total_voxels: int = 0
    alive_voxels: int = 0
    memory_saved: float = 0.0
    last_event: str = ""
    integration_count: int = 0
    trauma_count: int = 0


class FileCreature:
    """
    Файл-существо - шар из вокселей, созданный из файла.
    
    Движется к центральному организму для интеграции или отторжения.
    """
    
    def __init__(self, file_path: str, file_data: bytes, spawn_pos: np.ndarray,
                 num_voxels: int = 1000):
        self.file_path = file_path
        self.semantic = compute_semantic_fingerprint(file_data)
        self.spawn_pos = spawn_pos.copy()
        self.current_pos = spawn_pos.copy()
        self.target_pos = np.zeros(3)
        self.num_voxels = num_voxels
        self.voxels: List[Voxel] = []
        self.alive = True
        self.integrated = False
        self.rejected = False
        self.entity_id = int(time.time() * 1000) % 1000000
        
        # Эмоция существа на основе семантики
        self.base_emotion = self._compute_emotion_from_semantic()
        
        self._generate_voxels()
    
    def _compute_emotion_from_semantic(self) -> np.ndarray:
        """Вычислить базовую эмоцию из семантики"""
        emotion = np.zeros(4)
        
        # Используем семантический вектор для определения эмоции
        # Семантика уже нормализована в [-1, 1], поэтому преобразуем
        s = self.semantic
        
        # Joy - от положительных значений семантики
        emotion[0] = max(0.1, (s[0] + s[4] + 1) / 3)
        # Fear - от отрицательных значений и высокой вариативности  
        emotion[1] = max(0.05, (-s[1] + abs(s[5]) + 1) / 4)
        # Anger - от резких значений
        emotion[2] = max(0.05, (abs(s[2]) + abs(s[6]) - 0.5) / 2)
        # Peace - от сбалансированности
        emotion[3] = max(0.1, 1 - np.std(s[:4]))
        
        # Нормализация
        emotion = np.clip(emotion, 0.05, 1.0)
        emotion = emotion / (np.sum(emotion) + 1e-10)
        return emotion
    
    def _generate_voxels(self):
        """Генерация вокселей сферической формы"""
        # Радиус сферы
        radius = (self.num_voxels / (4/3 * np.pi)) ** (1/3) * 1.5
        
        # Генерация точек в сфере (Фибоначчи-спираль для равномерности)
        golden_ratio = (1 + np.sqrt(5)) / 2
        
        for i in range(self.num_voxels):
            # Фибоначчи-сфера
            theta = 2 * np.pi * i / golden_ratio
            phi = np.arccos(1 - 2 * (i + 0.5) / self.num_voxels)
            
            # Случайный радиус для заполнения объёма
            r = radius * (np.random.random() ** (1/3))
            
            x = self.current_pos[0] + r * np.sin(phi) * np.cos(theta)
            y = self.current_pos[1] + r * np.sin(phi) * np.sin(theta)
            z = self.current_pos[2] + r * np.cos(phi)
            
            voxel = Voxel(
                x=x, y=y, z=z,
                energy=0.8 + np.random.random() * 0.2,
                emotion=self.base_emotion.copy() + np.random.randn(4) * 0.05,
                trauma=0.0,
                semantic=self.semantic.copy(),
                entity_id=self.entity_id
            )
            voxel.emotion = np.clip(voxel.emotion, 0, 1)
            voxel.emotion = voxel.emotion / (np.sum(voxel.emotion) + 1e-10)
            
            self.voxels.append(voxel)
    
    def move_towards(self, target: np.ndarray, speed: float = 2.0):
        """Движение к цели"""
        direction = target - self.current_pos
        distance = np.linalg.norm(direction)
        
        if distance < 1.0:
            return True  # Достигли цели
        
        direction = direction / distance
        movement = direction * min(speed, distance)
        
        # Обновление позиции всех вокселей
        for voxel in self.voxels:
            voxel.x += movement[0]
            voxel.y += movement[1]
            voxel.z += movement[2]
        
        self.current_pos += movement
        return False
    
    def get_positions(self) -> np.ndarray:
        """Получить позиции всех вокселей"""
        return np.array([[v.x, v.y, v.z] for v in self.voxels])
    
    def get_colors(self, highlight: str = None) -> np.ndarray:
        """Получить цвета с возможной подсветкой"""
        colors = []
        
        for v in self.voxels:
            if highlight == 'green':
                # Зелёная интеграция
                r = 0.1 + v.emotion[0] * 0.2
                g = 0.7 + v.energy * 0.3
                b = 0.3
            elif highlight == 'red':
                # Красное отторжение
                r = 0.8 + v.trauma * 0.2
                g = 0.1
                b = 0.1
            else:
                # Обычный цвет
                joy, fear, anger, peace = v.emotion
                r = anger * 0.5 + fear * 0.3 + 0.3
                g = joy * 0.6 + peace * 0.3 + 0.2
                b = peace * 0.6 + 0.2
            
            colors.append([r * v.energy, g * v.energy, b * v.energy])
        
        return np.clip(np.array(colors), 0, 1)


class MetaOrganism:
    """
    Центральный Мета-Организм - живое цифровое сознание.
    
    100k-1M вокселей, пульсирующий, дышащий, чувствующий.
    """
    
    def __init__(self, num_voxels: int = 100000, center: np.ndarray = None):
        self.storage = ANIRLEStorage(resolution=1.0)
        self.center = center if center is not None else np.zeros(3)
        self.num_voxels = num_voxels
        self.radius = (num_voxels / (4/3 * np.pi)) ** (1/3) * 2
        
        # Состояние
        self.state = OrganismState()
        # Базовая семантика организма (нормализованный вектор в [-1, 1])
        self.base_semantic = np.random.randn(8)  # Нормальное распределение для лучшего разброса
        self.base_semantic = self.base_semantic / np.linalg.norm(self.base_semantic)
        
        # Пульсация
        self.pulse_phase = 0.0
        self.pulse_frequency = 0.5  # Hz
        self.breath_amplitude = 0.03
        
        # Коэволюция (Лавренков 2016)
        self.emotion_history: List[np.ndarray] = []
        self.global_emotion = np.array([0.4, 0.1, 0.1, 0.4])  # joy, fear, anger, peace
        
        # Живые существа
        self.creatures: List[FileCreature] = []
        
        # События
        self.on_integration: Optional[Callable] = None
        self.on_trauma: Optional[Callable] = None
        self.on_update: Optional[Callable] = None
        
        # Генерация
        self._generate_organism()
    
    def _generate_organism(self):
        """Генерация начального организма в форме пульсирующего шара/дерева"""
        print(f"🧬 Генерация организма из {self.num_voxels} вокселей...")
        
        golden_ratio = (1 + np.sqrt(5)) / 2
        
        # Основное тело - сфера
        main_body_count = int(self.num_voxels * 0.7)
        
        for i in range(main_body_count):
            # Фибоначчи-сфера для равномерного распределения
            theta = 2 * np.pi * i / golden_ratio
            phi = np.arccos(1 - 2 * (i + 0.5) / main_body_count)
            
            # Радиус с вариацией (органическая форма)
            r_variation = 1.0 + 0.1 * np.sin(theta * 5) * np.cos(phi * 3)
            r = self.radius * (np.random.random() ** (1/3)) * r_variation
            
            x = self.center[0] + r * np.sin(phi) * np.cos(theta)
            y = self.center[1] + r * np.sin(phi) * np.sin(theta)
            z = self.center[2] + r * np.cos(phi)
            
            # Энергия выше в центре
            distance_ratio = r / self.radius
            energy = 0.9 - distance_ratio * 0.3 + np.random.random() * 0.1
            
            # Эмоция зависит от позиции
            emotion = self.global_emotion.copy()
            emotion[0] += (1 - distance_ratio) * 0.2  # Больше радости в центре
            emotion[3] += distance_ratio * 0.1  # Больше покоя на периферии
            emotion = np.clip(emotion, 0, 1)
            emotion = emotion / (np.sum(emotion) + 1e-10)
            
            voxel = Voxel(
                x=x, y=y, z=z,
                energy=energy,
                emotion=emotion,
                trauma=0.0,
                semantic=self.base_semantic.copy() + np.random.randn(8) * 0.05,
                entity_id=-1  # Центральный организм
            )
            self.storage.add(voxel)
        
        # Дендриты - ветви от центра
        dendrite_count = self.num_voxels - main_body_count
        num_branches = 12
        voxels_per_branch = dendrite_count // num_branches
        
        for branch in range(num_branches):
            # Направление ветви
            branch_theta = 2 * np.pi * branch / num_branches
            branch_phi = np.pi / 4 + np.random.random() * np.pi / 2
            
            branch_dir = np.array([
                np.sin(branch_phi) * np.cos(branch_theta),
                np.sin(branch_phi) * np.sin(branch_theta),
                np.cos(branch_phi)
            ])
            
            for i in range(voxels_per_branch):
                # Позиция вдоль ветви с рандомизацией
                t = (i / voxels_per_branch) * 1.5
                spread = 0.1 + t * 0.3  # Ветви расширяются
                
                pos = self.center + branch_dir * (self.radius + t * self.radius * 0.5)
                pos += np.random.randn(3) * spread
                
                energy = 0.7 - t * 0.2 + np.random.random() * 0.1
                
                voxel = Voxel(
                    x=pos[0], y=pos[1], z=pos[2],
                    energy=max(0.3, energy),
                    emotion=self.global_emotion.copy(),
                    trauma=0.0,
                    semantic=self.base_semantic.copy(),
                    entity_id=-1
                )
                self.storage.add(voxel)
        
        self._update_state()
        print(f"✅ Организм создан: {len(self.storage)} вокселей")
    
    def spawn_creature(self, file_path: str, file_data: bytes) -> FileCreature:
        """Создать существо из файла"""
        # Позиция спавна - случайная точка на расстоянии от организма
        angle = np.random.random() * 2 * np.pi
        phi = np.random.random() * np.pi
        spawn_distance = self.radius * 3
        
        spawn_pos = self.center + np.array([
            spawn_distance * np.sin(phi) * np.cos(angle),
            spawn_distance * np.sin(phi) * np.sin(angle),
            spawn_distance * np.cos(phi)
        ])
        
        # Размер существа пропорционален размеру файла
        file_size = len(file_data)
        num_voxels = min(2000, max(500, int(np.log2(file_size + 1) * 100)))
        
        creature = FileCreature(file_path, file_data, spawn_pos, num_voxels)
        creature.target_pos = self.center.copy()
        
        self.creatures.append(creature)
        self.state.last_event = f"🌟 Создано существо: {file_path}"
        
        print(f"🌟 Существо создано из {file_path}: {num_voxels} вокселей")
        return creature
    
    def check_compatibility(self, creature: FileCreature) -> Tuple[float, float]:
        """
        Проверить совместимость существа с организмом.
        
        Returns:
            (semantic_similarity, emotion_similarity)
        """
        # Семантическое сходство
        semantic_sim = cosine_similarity(creature.semantic, self.base_semantic)
        
        # Эмоциональное сходство
        emotion_sim = cosine_similarity(creature.base_emotion, self.global_emotion)
        
        return semantic_sim, emotion_sim
    
    def integrate_creature(self, creature: FileCreature):
        """
        Интеграция существа в организм (совместимость > 0.7).
        
        Зелёный свет, +energy, рост организма.
        """
        print(f"💚 ИНТЕГРАЦИЯ: {creature.file_path}")
        
        # Добавляем воксели существа в организм
        for voxel in creature.voxels:
            voxel.entity_id = -1  # Теперь часть организма
            voxel.energy = min(1.0, voxel.energy + 0.2)  # Буст энергии
            voxel.emotion[0] = min(1.0, voxel.emotion[0] + 0.3)  # Буст радости
            self.storage.add(voxel)
        
        # Обновление семантики организма (обучение)
        self.base_semantic = (self.base_semantic * 0.95 + creature.semantic * 0.05)
        self.base_semantic = self.base_semantic / np.linalg.norm(self.base_semantic)
        
        # Глобальная эмоция - радость
        self.global_emotion[0] = min(1.0, self.global_emotion[0] + 0.1)
        self.global_emotion = self.global_emotion / np.sum(self.global_emotion)
        
        # Энергетический импульс существующим вокселям
        for voxel in self.storage:
            if voxel.entity_id == -1:
                distance = np.linalg.norm(voxel.pos - creature.current_pos)
                if distance < self.radius * 0.5:
                    voxel.energy = min(1.0, voxel.energy + 0.1)
        
        creature.integrated = True
        creature.alive = False
        self.state.integration_count += 1
        self.state.last_event = f"💚 Интегрировано: {creature.file_path}"
        self.state.mood = "кайф"
        self.state.mood_intensity = 0.9
        
        if self.on_integration:
            self.on_integration(creature)
        
        self._update_state()
    
    def reject_creature(self, creature: FileCreature, severity: float = 0.5):
        """
        Отторжение существа (совместимость < 0.4).
        
        По Никоновой 2013 (травма тканей) и Ахмадуллиной 2020 (атрофия).
        
        Красный свет, +trauma, удаление 5-20% вокселей.
        """
        print(f"🔴 ОТТОРЖЕНИЕ: {creature.file_path} (severity={severity:.2f})")
        
        # Процент атрофии (по Ахмадуллиной)
        atrophy_percent = 0.05 + severity * 0.15  # 5-20%
        
        # Определяем зону травмы - ближайшие к существу воксели
        trauma_zone_radius = self.radius * 0.3
        trauma_center = creature.current_pos.copy()
        
        voxels_to_remove = []
        trauma_voxels = []
        
        for voxel in self.storage:
            if voxel.entity_id == -1:
                distance = np.linalg.norm(voxel.pos - trauma_center)
                if distance < trauma_zone_radius:
                    trauma_voxels.append((voxel, distance))
        
        # Сортируем по близости к центру травмы
        trauma_voxels.sort(key=lambda x: x[1])
        
        # Удаляем часть (атрофия)
        num_to_remove = int(len(trauma_voxels) * atrophy_percent)
        for voxel, _ in trauma_voxels[:num_to_remove]:
            voxels_to_remove.append(voxel)
        
        # Остальным в зоне - травма
        for voxel, distance in trauma_voxels[num_to_remove:]:
            trauma_amount = (1 - distance / trauma_zone_radius) * severity
            voxel.trauma = min(1.0, voxel.trauma + trauma_amount)
            voxel.emotion[1] = min(1.0, voxel.emotion[1] + 0.2)  # Страх
            voxel.emotion[2] = min(1.0, voxel.emotion[2] + 0.3)  # Гнев
            voxel.emotion = voxel.emotion / np.sum(voxel.emotion)
        
        # Удаление
        for voxel in voxels_to_remove:
            self.storage.remove(voxel.x, voxel.y, voxel.z)
        
        # Глобальная эмоция - стресс
        self.global_emotion[1] += 0.15  # Страх
        self.global_emotion[2] += 0.2   # Гнев
        self.global_emotion[0] = max(0.05, self.global_emotion[0] - 0.1)  # Меньше радости
        self.global_emotion = self.global_emotion / np.sum(self.global_emotion)
        
        creature.rejected = True
        creature.alive = False
        self.state.trauma_count += 1
        self.state.last_event = f"🔴 Отторгнуто: {creature.file_path} ({num_to_remove} вокселей потеряно)"
        self.state.mood = "травма"
        self.state.mood_intensity = severity
        
        if self.on_trauma:
            self.on_trauma(creature, num_to_remove)
        
        self._update_state()
    
    def update(self, dt: float = 0.016):
        """
        Обновление организма (один тик).
        
        Включает:
        - Пульсацию
        - Движение существ
        - Коэволюцию эмоций (Лавренков 2016)
        - Декей и восстановление
        """
        # Пульсация
        self.pulse_phase += dt * self.pulse_frequency * 2 * np.pi
        pulse = np.sin(self.pulse_phase) * self.breath_amplitude
        
        # Обновление вокселей
        for voxel in self.storage:
            # Пульсация - дыхание
            direction = voxel.pos - self.center
            distance = np.linalg.norm(direction)
            if distance > 0.1:
                direction = direction / distance
                # Воксели "дышат" - двигаются к/от центра
                voxel.x += direction[0] * pulse * (1 - distance / self.radius)
                voxel.y += direction[1] * pulse * (1 - distance / self.radius)
                voxel.z += direction[2] * pulse * (1 - distance / self.radius)
            
            # Медленное восстановление
            if voxel.trauma > 0:
                voxel.trauma = max(0, voxel.trauma - dt * 0.02)
            
            # Коэволюция эмоций (движение к глобальной эмоции)
            voxel.emotion = voxel.emotion * 0.99 + self.global_emotion * 0.01
            voxel.emotion = voxel.emotion / np.sum(voxel.emotion)
        
        # Движение существ к центру
        creatures_to_process = []
        for creature in self.creatures:
            if creature.alive:
                reached = creature.move_towards(self.center, speed=dt * 50)
                if reached:
                    creatures_to_process.append(creature)
        
        # Обработка достигших существ
        for creature in creatures_to_process:
            sem_sim, emo_sim = self.check_compatibility(creature)
            # Семантика важнее эмоции для определения совместимости
            combined = sem_sim * 0.7 + emo_sim * 0.3
            
            print(f"📊 Совместимость {creature.file_path}: sem={sem_sim:.2f}, emo={emo_sim:.2f}, combined={combined:.2f}")
            
            # Также проверяем эмоциональную совместимость отдельно
            emotion_diff = np.linalg.norm(creature.base_emotion - self.global_emotion)
            is_emotional_mismatch = emotion_diff > 0.5
            
            if sem_sim > 0.3 and not is_emotional_mismatch:
                # Хорошая семантическая и эмоциональная совместимость
                self.integrate_creature(creature)
            elif sem_sim < 0.0 or is_emotional_mismatch:
                # Плохая семантическая или эмоциональная совместимость
                severity = max(0.3, 0.8 - combined)
                self.reject_creature(creature, severity=severity)
            else:
                # Неопределённость - частичная интеграция с предупреждением
                print(f"⚠️ Пограничная совместимость, интеграция с риском")
                self.integrate_creature(creature)
        
        # Удаление мёртвых существ из списка
        self.creatures = [c for c in self.creatures if c.alive]
        
        # Коэволюция глобальной эмоции (медленное возвращение к покою)
        self.global_emotion[3] = min(1.0, self.global_emotion[3] + dt * 0.01)
        self.global_emotion = self.global_emotion / np.sum(self.global_emotion)
        
        # Обновление состояния
        self._update_state()
        self.storage.tick += 1
        
        if self.on_update:
            self.on_update(self.state)
    
    def _update_state(self):
        """Обновление состояния организма"""
        stats = self.storage.get_statistics()
        
        self.state.total_voxels = stats['total_voxels']
        self.state.alive_voxels = stats['alive_voxels']
        self.state.memory_saved = stats['memory_saved_percent']
        
        # Здоровье = среднее от энергии и отсутствия травмы
        if stats['total_voxels'] > 0:
            self.state.health = (stats['avg_energy'] + (1 - stats['avg_trauma'])) / 2
        else:
            self.state.health = 0
        
        # Определение настроения
        dominant_idx = np.argmax(self.global_emotion)
        moods = ['кайф', 'тревога', 'гнев', 'покой']
        self.state.mood = moods[dominant_idx]
        self.state.mood_intensity = self.global_emotion[dominant_idx]
    
    def get_all_positions(self) -> np.ndarray:
        """Получить все позиции (организм + существа)"""
        org_pos = self.storage.get_all_positions()
        
        creature_positions = []
        for creature in self.creatures:
            if creature.alive:
                creature_positions.append(creature.get_positions())
        
        if creature_positions:
            all_positions = np.vstack([org_pos] + creature_positions)
        else:
            all_positions = org_pos
        
        return all_positions
    
    def get_all_colors(self) -> np.ndarray:
        """Получить все цвета"""
        org_colors = self.storage.get_all_colors()
        
        creature_colors = []
        for creature in self.creatures:
            if creature.alive:
                # Подсветка на основе приближения
                distance = np.linalg.norm(creature.current_pos - self.center)
                if distance < self.radius * 1.5:
                    sem_sim, emo_sim = self.check_compatibility(creature)
                    combined = (sem_sim + emo_sim) / 2
                    if combined > 0.5:
                        highlight = 'green'
                    elif combined < 0.35:
                        highlight = 'red'
                    else:
                        highlight = None
                else:
                    highlight = None
                creature_colors.append(creature.get_colors(highlight))
        
        if creature_colors:
            all_colors = np.vstack([org_colors] + creature_colors)
        else:
            all_colors = org_colors
        
        return all_colors

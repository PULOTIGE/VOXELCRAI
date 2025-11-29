#!/usr/bin/env python3
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                     CrimeaAI META ORGANISM - SINGLE FILE                      ║
║                   🧬 Живое Цифровое Сознание 🧬                               ║
║                                                                              ║
║  GitHub: https://github.com/PULOTIGE/crimeaai-meta-organism                  ║
║  Для песочницы Grok - упрощённая версия в одном файле                        ║
╚══════════════════════════════════════════════════════════════════════════════╝

Запуск: python crimeaai_single_file.py
Зависимости: pip install numpy
"""

import numpy as np
from dataclasses import dataclass, field
from typing import List, Dict, Tuple, Optional
import hashlib
import time

# ═══════════════════════════════════════════════════════════════════════════════
# VOXEL CORE
# ═══════════════════════════════════════════════════════════════════════════════

@dataclass
class Voxel:
    """Единица цифрового сознания"""
    x: float
    y: float
    z: float
    energy: float = 1.0
    emotion: np.ndarray = field(default_factory=lambda: np.array([0.4, 0.1, 0.1, 0.4]))
    trauma: float = 0.0
    semantic: np.ndarray = field(default_factory=lambda: np.zeros(8))
    connections: np.ndarray = field(default_factory=lambda: np.full(6, -1, dtype=np.int64))
    id: int = -1
    entity_id: int = -1
    
    @property
    def pos(self) -> np.ndarray:
        return np.array([self.x, self.y, self.z])
    
    def is_alive(self) -> bool:
        return self.energy > 0.01 and self.trauma < 0.99


class ANIRLEStorage:
    """ANIRLE-подобная sparse компрессия (по Бланко 2013)"""
    
    def __init__(self, resolution: float = 1.0):
        self.resolution = resolution
        self.voxels: Dict[Tuple[int, int, int], Voxel] = {}
        self.next_id = 0
        self.tick = 0
        
    def _quantize(self, x: float, y: float, z: float) -> Tuple[int, int, int]:
        return (int(np.floor(x / self.resolution)),
                int(np.floor(y / self.resolution)),
                int(np.floor(z / self.resolution)))
    
    def add(self, voxel: Voxel) -> int:
        key = self._quantize(voxel.x, voxel.y, voxel.z)
        voxel.id = self.next_id
        self.voxels[key] = voxel
        self.next_id += 1
        return voxel.id
    
    def get(self, x: float, y: float, z: float) -> Optional[Voxel]:
        return self.voxels.get(self._quantize(x, y, z))
    
    def remove(self, x: float, y: float, z: float) -> bool:
        key = self._quantize(x, y, z)
        if key in self.voxels:
            del self.voxels[key]
            return True
        return False
    
    def __len__(self):
        return len(self.voxels)
    
    def __iter__(self):
        return iter(self.voxels.values())


def compute_semantic_fingerprint(data: bytes) -> np.ndarray:
    """Вычислить семантический отпечаток данных (8 измерений)"""
    hash_bytes = hashlib.md5(data).digest()
    semantic = np.zeros(8)
    for i in range(4):
        semantic[i] = (hash_bytes[i * 4] / 255.0) * 2 - 1
    if len(data) > 0:
        arr = np.frombuffer(data[:min(len(data), 10000)], dtype=np.uint8)
        semantic[4] = (np.mean(arr) / 255.0) * 2 - 1
        semantic[5] = (np.std(arr) / 128.0) - 1
        semantic[6] = (len(np.unique(arr)) / 256.0) * 2 - 1
        semantic[7] = (np.sum(arr > 127) / len(arr)) * 2 - 1
    norm = np.linalg.norm(semantic)
    if norm > 0:
        semantic = semantic / norm
    return semantic


def cosine_similarity(a: np.ndarray, b: np.ndarray) -> float:
    """Косинусное сходство между векторами"""
    norm_a, norm_b = np.linalg.norm(a), np.linalg.norm(b)
    if norm_a < 1e-10 or norm_b < 1e-10:
        return 0.0
    return np.dot(a, b) / (norm_a * norm_b)


# ═══════════════════════════════════════════════════════════════════════════════
# FILE CREATURE
# ═══════════════════════════════════════════════════════════════════════════════

class FileCreature:
    """Файл-существо - шар из вокселей"""
    
    def __init__(self, file_path: str, file_data: bytes, spawn_pos: np.ndarray, num_voxels: int = 800):
        self.file_path = file_path
        self.semantic = compute_semantic_fingerprint(file_data)
        self.current_pos = spawn_pos.copy()
        self.num_voxels = num_voxels
        self.voxels: List[Voxel] = []
        self.alive = True
        self.integrated = False
        self.rejected = False
        self.entity_id = int(time.time() * 1000) % 1000000
        self.base_emotion = self._compute_emotion()
        self._generate_voxels()
    
    def _compute_emotion(self) -> np.ndarray:
        s = self.semantic
        emotion = np.array([
            max(0.1, (s[0] + s[4] + 1) / 3),
            max(0.05, (-s[1] + abs(s[5]) + 1) / 4),
            max(0.05, (abs(s[2]) + abs(s[6]) - 0.5) / 2),
            max(0.1, 1 - np.std(s[:4]))
        ])
        return emotion / np.sum(emotion)
    
    def _generate_voxels(self):
        radius = (self.num_voxels / (4/3 * np.pi)) ** (1/3) * 1.5
        golden_ratio = (1 + np.sqrt(5)) / 2
        for i in range(self.num_voxels):
            theta = 2 * np.pi * i / golden_ratio
            phi = np.arccos(1 - 2 * (i + 0.5) / self.num_voxels)
            r = radius * (np.random.random() ** (1/3))
            x = self.current_pos[0] + r * np.sin(phi) * np.cos(theta)
            y = self.current_pos[1] + r * np.sin(phi) * np.sin(theta)
            z = self.current_pos[2] + r * np.cos(phi)
            voxel = Voxel(x=x, y=y, z=z, energy=0.8 + np.random.random() * 0.2,
                         emotion=self.base_emotion.copy(), semantic=self.semantic.copy(),
                         entity_id=self.entity_id)
            self.voxels.append(voxel)
    
    def move_towards(self, target: np.ndarray, speed: float = 2.0) -> bool:
        direction = target - self.current_pos
        distance = np.linalg.norm(direction)
        if distance < 1.0:
            return True
        direction = direction / distance
        movement = direction * min(speed, distance)
        for voxel in self.voxels:
            voxel.x += movement[0]
            voxel.y += movement[1]
            voxel.z += movement[2]
        self.current_pos += movement
        return False


# ═══════════════════════════════════════════════════════════════════════════════
# META ORGANISM
# ═══════════════════════════════════════════════════════════════════════════════

class MetaOrganism:
    """Центральный Мета-Организм - живое цифровое сознание"""
    
    def __init__(self, num_voxels: int = 10000):
        self.storage = ANIRLEStorage(resolution=1.0)
        self.center = np.zeros(3)
        self.num_voxels = num_voxels
        self.radius = (num_voxels / (4/3 * np.pi)) ** (1/3) * 2
        
        self.base_semantic = np.random.randn(8)
        self.base_semantic = self.base_semantic / np.linalg.norm(self.base_semantic)
        
        self.global_emotion = np.array([0.4, 0.1, 0.1, 0.4])
        self.creatures: List[FileCreature] = []
        
        self.health = 1.0
        self.mood = "покой"
        self.integration_count = 0
        self.trauma_count = 0
        
        self._generate_organism()
    
    def _generate_organism(self):
        print(f"🧬 Генерация организма ({self.num_voxels} вокселей)...")
        golden_ratio = (1 + np.sqrt(5)) / 2
        for i in range(self.num_voxels):
            theta = 2 * np.pi * i / golden_ratio
            phi = np.arccos(1 - 2 * (i + 0.5) / self.num_voxels)
            r = self.radius * (np.random.random() ** (1/3))
            x = self.center[0] + r * np.sin(phi) * np.cos(theta)
            y = self.center[1] + r * np.sin(phi) * np.sin(theta)
            z = self.center[2] + r * np.cos(phi)
            distance_ratio = r / self.radius
            energy = 0.9 - distance_ratio * 0.3 + np.random.random() * 0.1
            voxel = Voxel(x=x, y=y, z=z, energy=energy, emotion=self.global_emotion.copy(),
                         semantic=self.base_semantic.copy(), entity_id=-1)
            self.storage.add(voxel)
        print(f"✅ Организм создан: {len(self.storage)} вокселей")
    
    def spawn_creature(self, file_path: str, file_data: bytes) -> FileCreature:
        angle = np.random.random() * 2 * np.pi
        phi = np.random.random() * np.pi
        spawn_distance = self.radius * 3
        spawn_pos = self.center + np.array([
            spawn_distance * np.sin(phi) * np.cos(angle),
            spawn_distance * np.sin(phi) * np.sin(angle),
            spawn_distance * np.cos(phi)
        ])
        num_voxels = min(1500, max(400, int(np.log2(len(file_data) + 1) * 80)))
        creature = FileCreature(file_path, file_data, spawn_pos, num_voxels)
        self.creatures.append(creature)
        print(f"🌟 Существо создано: {file_path} ({num_voxels} вокселей)")
        return creature
    
    def check_compatibility(self, creature: FileCreature) -> Tuple[float, float]:
        sem_sim = cosine_similarity(creature.semantic, self.base_semantic)
        emo_sim = cosine_similarity(creature.base_emotion, self.global_emotion)
        return sem_sim, emo_sim
    
    def integrate_creature(self, creature: FileCreature):
        print(f"💚 ИНТЕГРАЦИЯ: {creature.file_path}")
        for voxel in creature.voxels:
            voxel.entity_id = -1
            voxel.energy = min(1.0, voxel.energy + 0.2)
            self.storage.add(voxel)
        self.global_emotion[0] = min(1.0, self.global_emotion[0] + 0.1)
        self.global_emotion = self.global_emotion / np.sum(self.global_emotion)
        creature.integrated = True
        creature.alive = False
        self.integration_count += 1
        self.mood = "кайф"
    
    def reject_creature(self, creature: FileCreature, severity: float = 0.5):
        print(f"🔴 ОТТОРЖЕНИЕ: {creature.file_path} (severity={severity:.2f})")
        atrophy_percent = 0.05 + severity * 0.15
        trauma_zone_radius = self.radius * 0.3
        trauma_voxels = []
        for voxel in self.storage:
            if voxel.entity_id == -1:
                distance = np.linalg.norm(voxel.pos - creature.current_pos)
                if distance < trauma_zone_radius:
                    trauma_voxels.append((voxel, distance))
        trauma_voxels.sort(key=lambda x: x[1])
        num_to_remove = int(len(trauma_voxels) * atrophy_percent)
        for voxel, _ in trauma_voxels[:num_to_remove]:
            self.storage.remove(voxel.x, voxel.y, voxel.z)
        for voxel, distance in trauma_voxels[num_to_remove:]:
            trauma_amount = (1 - distance / trauma_zone_radius) * severity
            voxel.trauma = min(1.0, voxel.trauma + trauma_amount)
        self.global_emotion[1] += 0.15
        self.global_emotion[2] += 0.2
        self.global_emotion = self.global_emotion / np.sum(self.global_emotion)
        creature.rejected = True
        creature.alive = False
        self.trauma_count += 1
        self.mood = "травма"
        print(f"   Потеряно {num_to_remove} вокселей (атрофия)")
    
    def update(self, dt: float = 0.016):
        creatures_to_process = []
        for creature in self.creatures:
            if creature.alive:
                reached = creature.move_towards(self.center, speed=dt * 50)
                if reached:
                    creatures_to_process.append(creature)
        
        for creature in creatures_to_process:
            sem_sim, emo_sim = self.check_compatibility(creature)
            combined = sem_sim * 0.7 + emo_sim * 0.3
            print(f"📊 Совместимость {creature.file_path}: sem={sem_sim:.2f}, emo={emo_sim:.2f}, combined={combined:.2f}")
            
            emotion_diff = np.linalg.norm(creature.base_emotion - self.global_emotion)
            if sem_sim > 0.3 and emotion_diff <= 0.5:
                self.integrate_creature(creature)
            elif sem_sim < 0.0 or emotion_diff > 0.5:
                self.reject_creature(creature, severity=max(0.3, 0.8 - combined))
            else:
                print(f"⚠️ Пограничная совместимость")
                self.integrate_creature(creature)
        
        self.creatures = [c for c in self.creatures if c.alive]
        self.global_emotion[3] = min(1.0, self.global_emotion[3] + dt * 0.01)
        self.global_emotion = self.global_emotion / np.sum(self.global_emotion)
        
        if len(self.storage) > 0:
            energies = [v.energy for v in self.storage]
            traumas = [v.trauma for v in self.storage]
            self.health = (np.mean(energies) + (1 - np.mean(traumas))) / 2
        
        moods = ['кайф', 'тревога', 'гнев', 'покой']
        self.mood = moods[np.argmax(self.global_emotion)]
        self.storage.tick += 1


# ═══════════════════════════════════════════════════════════════════════════════
# DEMO
# ═══════════════════════════════════════════════════════════════════════════════

def generate_synthetic_file(organism: MetaOrganism, compatibility: float) -> bytes:
    """Генерация синтетических данных файла"""
    org_semantic = organism.base_semantic
    size = np.random.randint(5000, 30000)
    
    if compatibility > 0.7:
        target_mean = (org_semantic[4] + 1) / 2 * 255
        target_std = (org_semantic[5] + 1) * 64
        data = np.random.normal(target_mean, max(10, target_std), size)
        data = np.clip(data, 0, 255).astype(np.uint8)
    elif compatibility < 0.3:
        target_mean = 255 - ((org_semantic[4] + 1) / 2 * 255)
        target_std = max(10, 128 - (org_semantic[5] + 1) * 64)
        data = np.random.normal(target_mean, target_std, size)
        data = np.clip(data, 0, 255).astype(np.uint8)
        for i in range(0, len(data) - 100, 100):
            if np.random.random() > 0.5:
                data[i:i+50] = 0 if np.random.random() > 0.5 else 255
    else:
        data = np.random.randint(0, 256, size, dtype=np.uint8)
    
    return data.tobytes()


def run_demo():
    """30-секундная демонстрация"""
    print("""
╔══════════════════════════════════════════════════════════════════════════════╗
║                   CrimeaAI META ORGANISM - ДЕМОНСТРАЦИЯ                       ║
║                       🧬 Живое Цифровое Сознание 🧬                           ║
╚══════════════════════════════════════════════════════════════════════════════╝
    """)
    
    organism = MetaOrganism(num_voxels=8000)
    
    events = [
        (1.0, 0.9, "compatible.py"),
        (3.0, 0.2, "malware.exe"),
        (5.0, 0.85, "module.py"),
        (7.0, 0.15, "virus.bin"),
        (9.0, 0.75, "data.json"),
    ]
    events_idx = 0
    
    print("\n📟 Консольная демонстрация")
    print("=" * 60)
    
    start_time = time.time()
    last_time = start_time
    
    while True:
        current_time = time.time()
        elapsed = current_time - start_time
        dt = current_time - last_time
        last_time = current_time
        
        if elapsed > 15:
            break
        
        if events_idx < len(events) and elapsed >= events[events_idx][0]:
            event_time, compat, name = events[events_idx]
            events_idx += 1
            data = generate_synthetic_file(organism, compat)
            organism.spawn_creature(name, data)
            symbol = "💚" if compat > 0.5 else "🔴"
            print(f"\n{symbol} Новое существо: {name}")
        
        organism.update(dt)
        
        health_bar_len = 20
        health_filled = int(organism.health * health_bar_len)
        health_bar = "█" * health_filled + "░" * (health_bar_len - health_filled)
        
        mood_symbols = {'кайф': '😊', 'тревога': '😰', 'гнев': '😠', 'покой': '😌'}
        mood_sym = mood_symbols.get(organism.mood, '🤔')
        
        status = (
            f"\r[{elapsed:5.1f}s] "
            f"Health: [{health_bar}] {organism.health*100:5.1f}% | "
            f"Mood: {mood_sym} {organism.mood:6} | "
            f"Voxels: {len(organism.storage):6,} | "
            f"Int: {organism.integration_count} | Trauma: {organism.trauma_count}"
        )
        print(status, end="", flush=True)
        
        time.sleep(0.1)
    
    print(f"""

╔══════════════════════════════════════════════════════════════╗
║                    ФИНАЛЬНАЯ СТАТИСТИКА                       ║
╠══════════════════════════════════════════════════════════════╣
║  Здоровье организма:  {organism.health*100:6.1f}%                           ║
║  Всего вокселей:      {len(organism.storage):6,}                            ║
║  Настроение:          {organism.mood:6}                             ║
║  Успешных интеграций: {organism.integration_count:6}                            ║
║  Травм:               {organism.trauma_count:6}                            ║
╚══════════════════════════════════════════════════════════════╝

🧬 CrimeaAI Meta Organism - Это не симуляция, это новая форма жизни!
    """)


if __name__ == "__main__":
    run_demo()

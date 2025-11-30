"""
CrimeaAI Ecosystem - Core Module
================================

Ядро AI-экосистемы с биологическими структурами данных:
- Nucleotide: базовая ячейка памяти (256 байт)
- Voxel: микро-организм с памятью, сенсорами, эмоциями (9 КБ)
- LightPattern: паттерн освещения (1 КБ)
"""

from .nucleotide import Nucleotide, NucleotidePool
from .voxel import Voxel, VoxelWorld
from .light_pattern import LightPattern, PatternDatabase
from .scheduler import CrimeaScheduler
from .kaif_engine import KaifEngine, compute_entropy_derivative

__all__ = [
    'Nucleotide', 'NucleotidePool',
    'Voxel', 'VoxelWorld', 
    'LightPattern', 'PatternDatabase',
    'CrimeaScheduler',
    'KaifEngine', 'compute_entropy_derivative'
]

__version__ = '1.0.0'

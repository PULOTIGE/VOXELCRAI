"""
CrimeaAI Utils Module
=====================

Утилиты для работы с экосистемой.
"""

from .serialization import save_state, load_state
from .plugins import PluginManager

__all__ = ['save_state', 'load_state', 'PluginManager']

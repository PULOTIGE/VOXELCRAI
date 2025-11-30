"""
Пример конфигурации для Cursor AI Ecosystem
Скопируйте в config.py и настройте под себя
"""

# Количество компонентов
NUCLEOTIDES_COUNT = 100000  # от 1000 до 62000000
VOXELS_COUNT = 400  # от 10 до 10000

# Производительность
TARGET_FPS = 60  # целевая частота обновления
UPDATE_SAMPLE_SIZE = 1000  # размер выборки нуклеотидов для обновления

# Поиск концептов
ENABLE_CONCEPT_SEARCH = True
CONCEPT_SEARCH_INTERVAL = 1140  # секунд (19 минут)
CONCEPT_KEYWORDS = [
    '19V',
    'CrimeaAI',
    'artificial intelligence',
    'consciousness',
    'neural networks',
    'machine learning',
]

# Визуализация
ENABLE_VISUALIZATION = True
WINDOW_WIDTH = 1200
WINDOW_HEIGHT = 800
VOXEL_GRID_SIZE = 20  # количество ячеек в сетке

# Паттерны освещения
MAX_LIGHT_PATTERNS = 10000
PATTERN_SIMILARITY_THRESHOLD = 0.8

# Нуклеотиды
NUCLEOTIDE_EXPRESSION_DECAY = 0.999
NUCLEOTIDE_LEARNING_RATE = 0.001

# Воксели
VOXEL_ENERGY_START = 100.0
VOXEL_ENERGY_CONSUMPTION = 0.1
VOXEL_KAIF_THRESHOLD_HIGH = 0.1
VOXEL_KAIF_THRESHOLD_LOW = -0.1

# Система кайфа
EMOTION_HISTORY_LENGTH = 10
ENTROPY_BINS = 20

# Сеть
REQUEST_TIMEOUT = 10  # секунд
MAX_RESULTS_PER_SEARCH = 5
USER_AGENT = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'

# Отладка
DEBUG = False
VERBOSE_LOGGING = False
STATS_INTERVAL = 5.0  # секунд

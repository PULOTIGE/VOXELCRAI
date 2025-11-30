"""
CrimeaAI UI Theme - Cyberpunk Aesthetic
=======================================

Тема в стиле киберпанка с неоновыми цветами и тёмным фоном.
"""

# Основные цвета (Cyberpunk palette)
COLORS = {
    # Фоны
    'bg_primary': (10, 10, 20),           # Почти чёрный с синим оттенком
    'bg_secondary': (20, 20, 35),          # Чуть светлее
    'bg_tertiary': (30, 30, 50),           # Панели
    'bg_highlight': (40, 40, 70),          # Выделение
    
    # Акценты
    'accent_cyan': (0, 255, 255),          # Неоновый голубой
    'accent_magenta': (255, 0, 128),       # Неоновый розовый
    'accent_yellow': (255, 255, 0),        # Жёлтый
    'accent_green': (0, 255, 128),         # Неоновый зелёный
    'accent_orange': (255, 128, 0),        # Оранжевый
    'accent_purple': (180, 0, 255),        # Фиолетовый
    
    # Текст
    'text_primary': (240, 240, 255),       # Белый с лёгким голубым
    'text_secondary': (160, 160, 180),     # Серый
    'text_muted': (100, 100, 120),         # Приглушённый
    
    # Состояния
    'success': (0, 255, 128),              # Успех
    'warning': (255, 200, 0),              # Предупреждение
    'error': (255, 50, 80),                # Ошибка
    'info': (0, 200, 255),                 # Информация
    
    # Эмоции (для визуализации)
    'emotion_joy': (255, 220, 50),         # Радость - жёлтый
    'emotion_sadness': (80, 100, 200),     # Грусть - синий
    'emotion_anger': (255, 50, 50),        # Гнев - красный
    'emotion_fear': (180, 100, 255),       # Страх - фиолетовый
    'emotion_surprise': (255, 150, 0),     # Удивление - оранжевый
    'emotion_disgust': (100, 180, 80),     # Отвращение - зелёный
    'emotion_curiosity': (0, 220, 255),    # Любопытство - голубой
    'emotion_peace': (200, 200, 220),      # Покой - серебро
    
    # Кайф-уровни
    'kaif_dormant': (60, 60, 80),          # Покой
    'kaif_calm': (100, 150, 200),          # Спокойствие
    'kaif_active': (150, 200, 100),        # Активность
    'kaif_excited': (255, 180, 50),        # Возбуждение
    'kaif_ecstatic': (255, 100, 200),      # Экстаз
}

# Шрифты
FONTS = {
    'primary': 'JetBrains Mono',
    'secondary': 'Fira Code',
    'fallback': 'Consolas',
    'system': None  # Системный
}

# Размеры шрифтов
FONT_SIZES = {
    'h1': 32,
    'h2': 24,
    'h3': 18,
    'body': 14,
    'small': 12,
    'tiny': 10
}

# Размеры элементов
SIZES = {
    'padding_xs': 4,
    'padding_sm': 8,
    'padding_md': 12,
    'padding_lg': 16,
    'padding_xl': 24,
    
    'border_radius': 8,
    'border_width': 2,
    
    'button_height': 36,
    'input_height': 32,
    'panel_width': 280,
    
    'graph_height': 150,
    'status_bar_height': 32,
}

# Анимации
ANIMATIONS = {
    'transition_fast': 100,    # мс
    'transition_normal': 200,
    'transition_slow': 400,
    
    'pulse_speed': 1000,       # мс для пульсации
    'glow_intensity': 0.3,
}


def get_kaif_color(kaif: float) -> tuple:
    """Получение цвета по уровню кайфа"""
    if kaif < 0.1:
        return COLORS['kaif_dormant']
    elif kaif < 0.3:
        return COLORS['kaif_calm']
    elif kaif < 0.6:
        return COLORS['kaif_active']
    elif kaif < 0.8:
        return COLORS['kaif_excited']
    else:
        return COLORS['kaif_ecstatic']


def get_emotion_color(emotion: str) -> tuple:
    """Получение цвета для эмоции"""
    color_key = f'emotion_{emotion}'
    return COLORS.get(color_key, COLORS['text_secondary'])


def lerp_color(color1: tuple, color2: tuple, t: float) -> tuple:
    """Линейная интерполяция цветов"""
    return tuple(
        int(c1 + (c2 - c1) * t)
        for c1, c2 in zip(color1, color2)
    )


def add_glow(color: tuple, intensity: float = 0.3) -> tuple:
    """Добавление свечения к цвету"""
    return tuple(
        min(255, int(c + (255 - c) * intensity))
        for c in color
    )


def darken(color: tuple, factor: float = 0.5) -> tuple:
    """Затемнение цвета"""
    return tuple(int(c * factor) for c in color)


def alpha_blend(color: tuple, alpha: int) -> tuple:
    """Добавление альфа-канала"""
    if len(color) == 3:
        return (*color, alpha)
    return color

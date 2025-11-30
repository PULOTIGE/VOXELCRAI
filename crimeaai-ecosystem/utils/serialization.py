"""
Serialization - Сохранение и загрузка состояния
===============================================
"""

import os
import json
import time
from typing import Any, Dict, Optional
from dataclasses import asdict, is_dataclass

try:
    import msgpack
    MSGPACK_AVAILABLE = True
except ImportError:
    MSGPACK_AVAILABLE = False

try:
    import cloudpickle
    CLOUDPICKLE_AVAILABLE = True
except ImportError:
    CLOUDPICKLE_AVAILABLE = False


def save_state(
    data: Any,
    filepath: str,
    format: str = 'auto',
    compress: bool = False
) -> bool:
    """
    Сохранение состояния в файл
    
    Args:
        data: данные для сохранения
        filepath: путь к файлу
        format: формат ('json', 'msgpack', 'pickle', 'auto')
        compress: сжимать ли данные
    
    Returns:
        True если успешно
    """
    # Определяем формат
    if format == 'auto':
        ext = os.path.splitext(filepath)[1].lower()
        if ext == '.json':
            format = 'json'
        elif ext in ('.msgpack', '.mp'):
            format = 'msgpack'
        elif ext in ('.pkl', '.pickle'):
            format = 'pickle'
        else:
            format = 'json'
    
    # Создаём директорию если нужно
    os.makedirs(os.path.dirname(filepath) or '.', exist_ok=True)
    
    try:
        if format == 'json':
            # Преобразуем dataclass в dict
            if is_dataclass(data) and not isinstance(data, type):
                data = asdict(data)
            
            with open(filepath, 'w', encoding='utf-8') as f:
                json.dump(data, f, ensure_ascii=False, indent=2, default=str)
        
        elif format == 'msgpack' and MSGPACK_AVAILABLE:
            with open(filepath, 'wb') as f:
                msgpack.pack(data, f, default=str)
        
        elif format == 'pickle' and CLOUDPICKLE_AVAILABLE:
            with open(filepath, 'wb') as f:
                cloudpickle.dump(data, f)
        
        else:
            raise ValueError(f"Unsupported format: {format}")
        
        return True
    
    except Exception as e:
        print(f"❌ Ошибка сохранения: {e}")
        return False


def load_state(
    filepath: str,
    format: str = 'auto',
    default: Any = None
) -> Any:
    """
    Загрузка состояния из файла
    
    Args:
        filepath: путь к файлу
        format: формат ('json', 'msgpack', 'pickle', 'auto')
        default: значение по умолчанию если файл не найден
    
    Returns:
        Загруженные данные или default
    """
    if not os.path.exists(filepath):
        return default
    
    # Определяем формат
    if format == 'auto':
        ext = os.path.splitext(filepath)[1].lower()
        if ext == '.json':
            format = 'json'
        elif ext in ('.msgpack', '.mp'):
            format = 'msgpack'
        elif ext in ('.pkl', '.pickle'):
            format = 'pickle'
        else:
            format = 'json'
    
    try:
        if format == 'json':
            with open(filepath, 'r', encoding='utf-8') as f:
                return json.load(f)
        
        elif format == 'msgpack' and MSGPACK_AVAILABLE:
            with open(filepath, 'rb') as f:
                return msgpack.unpack(f)
        
        elif format == 'pickle' and CLOUDPICKLE_AVAILABLE:
            with open(filepath, 'rb') as f:
                return cloudpickle.load(f)
        
        else:
            raise ValueError(f"Unsupported format: {format}")
    
    except Exception as e:
        print(f"❌ Ошибка загрузки: {e}")
        return default


class StateManager:
    """
    Менеджер состояния системы
    
    Управляет автосохранением и восстановлением.
    """
    
    def __init__(self, save_dir: str = "data/saves"):
        """
        Создание менеджера
        
        Args:
            save_dir: директория для сохранений
        """
        self.save_dir = save_dir
        os.makedirs(save_dir, exist_ok=True)
        
        self._components: Dict[str, Any] = {}
        self._dirty: Dict[str, bool] = {}
    
    def register_component(self, name: str, component: Any):
        """Регистрация компонента для сохранения"""
        self._components[name] = component
        self._dirty[name] = False
    
    def mark_dirty(self, name: str):
        """Пометка компонента как изменённого"""
        if name in self._dirty:
            self._dirty[name] = True
    
    def save_all(self, slot: str = "default") -> bool:
        """
        Сохранение всех компонентов
        
        Args:
            slot: имя слота сохранения
        
        Returns:
            True если успешно
        """
        slot_dir = os.path.join(self.save_dir, slot)
        os.makedirs(slot_dir, exist_ok=True)
        
        metadata = {
            'timestamp': time.time(),
            'components': list(self._components.keys())
        }
        
        success = True
        
        for name, component in self._components.items():
            filepath = os.path.join(slot_dir, f"{name}.json")
            
            # Пробуем получить состояние через метод
            if hasattr(component, 'get_state'):
                data = component.get_state()
            elif hasattr(component, 'to_dict'):
                data = component.to_dict()
            elif hasattr(component, '__dict__'):
                data = component.__dict__
            else:
                data = component
            
            if not save_state(data, filepath):
                success = False
            else:
                self._dirty[name] = False
        
        # Сохраняем метаданные
        save_state(metadata, os.path.join(slot_dir, "metadata.json"))
        
        return success
    
    def load_all(self, slot: str = "default") -> bool:
        """
        Загрузка всех компонентов
        
        Args:
            slot: имя слота сохранения
        
        Returns:
            True если успешно
        """
        slot_dir = os.path.join(self.save_dir, slot)
        
        if not os.path.exists(slot_dir):
            return False
        
        metadata = load_state(os.path.join(slot_dir, "metadata.json"), default={})
        
        for name in metadata.get('components', []):
            if name not in self._components:
                continue
            
            filepath = os.path.join(slot_dir, f"{name}.json")
            data = load_state(filepath)
            
            if data is None:
                continue
            
            component = self._components[name]
            
            # Пробуем загрузить состояние через метод
            if hasattr(component, 'load_state'):
                component.load_state(data)
            elif hasattr(component, 'from_dict'):
                component.from_dict(data)
            elif hasattr(component, '__dict__'):
                component.__dict__.update(data)
        
        return True
    
    def list_slots(self) -> list:
        """Получение списка слотов сохранения"""
        slots = []
        
        if os.path.exists(self.save_dir):
            for name in os.listdir(self.save_dir):
                slot_dir = os.path.join(self.save_dir, name)
                if os.path.isdir(slot_dir):
                    metadata_path = os.path.join(slot_dir, "metadata.json")
                    if os.path.exists(metadata_path):
                        metadata = load_state(metadata_path, default={})
                        slots.append({
                            'name': name,
                            'timestamp': metadata.get('timestamp', 0),
                            'components': metadata.get('components', [])
                        })
        
        return sorted(slots, key=lambda x: x['timestamp'], reverse=True)
    
    def delete_slot(self, slot: str) -> bool:
        """Удаление слота сохранения"""
        import shutil
        
        slot_dir = os.path.join(self.save_dir, slot)
        
        if os.path.exists(slot_dir):
            shutil.rmtree(slot_dir)
            return True
        
        return False

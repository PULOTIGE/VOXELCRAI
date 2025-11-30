"""
Plugin System - Система плагинов
================================

Позволяет расширять функциональность экосистемы без изменения основного кода.
"""

import os
import sys
import importlib
import importlib.util
from typing import Dict, List, Optional, Callable, Any
from dataclasses import dataclass
from abc import ABC, abstractmethod


@dataclass
class PluginInfo:
    """Информация о плагине"""
    name: str
    version: str
    author: str
    description: str
    dependencies: List[str]


class PluginBase(ABC):
    """
    Базовый класс для плагинов
    
    Все плагины должны наследоваться от этого класса.
    """
    
    @property
    @abstractmethod
    def info(self) -> PluginInfo:
        """Информация о плагине"""
        pass
    
    @abstractmethod
    def initialize(self, ecosystem: Any) -> bool:
        """
        Инициализация плагина
        
        Args:
            ecosystem: ссылка на экосистему
        
        Returns:
            True если успешно
        """
        pass
    
    @abstractmethod
    def shutdown(self):
        """Завершение работы плагина"""
        pass
    
    def on_tick(self, dt: float):
        """Вызывается каждый тик"""
        pass
    
    def on_voxel_update(self, voxel: Any):
        """Вызывается при обновлении вокселя"""
        pass
    
    def on_nucleotide_update(self, nucleotide: Any):
        """Вызывается при обновлении нуклеотида"""
        pass
    
    def on_concept_found(self, concept: Any):
        """Вызывается при нахождении нового концепта"""
        pass


class PluginManager:
    """
    Менеджер плагинов
    
    Управляет загрузкой, инициализацией и выполнением плагинов.
    """
    
    def __init__(self, plugins_dir: str = "plugins"):
        """
        Создание менеджера
        
        Args:
            plugins_dir: директория с плагинами
        """
        self.plugins_dir = plugins_dir
        os.makedirs(plugins_dir, exist_ok=True)
        
        self.plugins: Dict[str, PluginBase] = {}
        self.enabled: Dict[str, bool] = {}
        
        # Callbacks
        self._hooks: Dict[str, List[Callable]] = {
            'tick': [],
            'voxel_update': [],
            'nucleotide_update': [],
            'concept_found': []
        }
    
    def discover_plugins(self) -> List[str]:
        """
        Поиск доступных плагинов
        
        Returns:
            Список найденных плагинов
        """
        found = []
        
        if not os.path.exists(self.plugins_dir):
            return found
        
        for filename in os.listdir(self.plugins_dir):
            if filename.endswith('.py') and not filename.startswith('_'):
                plugin_name = filename[:-3]
                found.append(plugin_name)
            elif os.path.isdir(os.path.join(self.plugins_dir, filename)):
                init_file = os.path.join(self.plugins_dir, filename, '__init__.py')
                if os.path.exists(init_file):
                    found.append(filename)
        
        return found
    
    def load_plugin(self, name: str) -> bool:
        """
        Загрузка плагина
        
        Args:
            name: имя плагина
        
        Returns:
            True если успешно
        """
        try:
            # Путь к файлу плагина
            plugin_path = os.path.join(self.plugins_dir, f"{name}.py")
            
            if not os.path.exists(plugin_path):
                # Проверяем директорию
                plugin_path = os.path.join(self.plugins_dir, name, '__init__.py')
                if not os.path.exists(plugin_path):
                    print(f"❌ Плагин не найден: {name}")
                    return False
            
            # Загружаем модуль
            spec = importlib.util.spec_from_file_location(name, plugin_path)
            module = importlib.util.module_from_spec(spec)
            sys.modules[name] = module
            spec.loader.exec_module(module)
            
            # Ищем класс плагина
            plugin_class = None
            for attr_name in dir(module):
                attr = getattr(module, attr_name)
                if (isinstance(attr, type) and 
                    issubclass(attr, PluginBase) and 
                    attr is not PluginBase):
                    plugin_class = attr
                    break
            
            if plugin_class is None:
                print(f"❌ Класс плагина не найден в {name}")
                return False
            
            # Создаём экземпляр
            plugin = plugin_class()
            self.plugins[name] = plugin
            self.enabled[name] = False
            
            print(f"✅ Плагин загружен: {plugin.info.name} v{plugin.info.version}")
            return True
        
        except Exception as e:
            print(f"❌ Ошибка загрузки плагина {name}: {e}")
            return False
    
    def unload_plugin(self, name: str) -> bool:
        """
        Выгрузка плагина
        
        Args:
            name: имя плагина
        
        Returns:
            True если успешно
        """
        if name not in self.plugins:
            return False
        
        # Отключаем если включён
        if self.enabled.get(name, False):
            self.disable_plugin(name)
        
        # Удаляем
        del self.plugins[name]
        del self.enabled[name]
        
        # Удаляем из sys.modules
        if name in sys.modules:
            del sys.modules[name]
        
        print(f"✅ Плагин выгружен: {name}")
        return True
    
    def enable_plugin(self, name: str, ecosystem: Any) -> bool:
        """
        Включение плагина
        
        Args:
            name: имя плагина
            ecosystem: ссылка на экосистему
        
        Returns:
            True если успешно
        """
        if name not in self.plugins:
            return False
        
        if self.enabled.get(name, False):
            return True  # Уже включён
        
        plugin = self.plugins[name]
        
        try:
            if plugin.initialize(ecosystem):
                self.enabled[name] = True
                
                # Регистрируем хуки
                if hasattr(plugin, 'on_tick'):
                    self._hooks['tick'].append(plugin.on_tick)
                if hasattr(plugin, 'on_voxel_update'):
                    self._hooks['voxel_update'].append(plugin.on_voxel_update)
                if hasattr(plugin, 'on_nucleotide_update'):
                    self._hooks['nucleotide_update'].append(plugin.on_nucleotide_update)
                if hasattr(plugin, 'on_concept_found'):
                    self._hooks['concept_found'].append(plugin.on_concept_found)
                
                print(f"✅ Плагин включён: {name}")
                return True
            else:
                print(f"❌ Ошибка инициализации плагина: {name}")
                return False
        
        except Exception as e:
            print(f"❌ Ошибка включения плагина {name}: {e}")
            return False
    
    def disable_plugin(self, name: str) -> bool:
        """
        Отключение плагина
        
        Args:
            name: имя плагина
        
        Returns:
            True если успешно
        """
        if name not in self.plugins:
            return False
        
        if not self.enabled.get(name, False):
            return True  # Уже отключён
        
        plugin = self.plugins[name]
        
        try:
            plugin.shutdown()
            
            # Удаляем хуки
            for hook_list in self._hooks.values():
                for method in [plugin.on_tick, plugin.on_voxel_update, 
                              plugin.on_nucleotide_update, plugin.on_concept_found]:
                    if method in hook_list:
                        hook_list.remove(method)
            
            self.enabled[name] = False
            print(f"✅ Плагин отключён: {name}")
            return True
        
        except Exception as e:
            print(f"❌ Ошибка отключения плагина {name}: {e}")
            return False
    
    def call_hook(self, hook_name: str, *args, **kwargs):
        """
        Вызов хука для всех плагинов
        
        Args:
            hook_name: имя хука
            *args, **kwargs: аргументы
        """
        for callback in self._hooks.get(hook_name, []):
            try:
                callback(*args, **kwargs)
            except Exception as e:
                print(f"❌ Ошибка в хуке {hook_name}: {e}")
    
    def get_plugin_info(self, name: str) -> Optional[PluginInfo]:
        """Получение информации о плагине"""
        if name in self.plugins:
            return self.plugins[name].info
        return None
    
    def list_plugins(self) -> List[Dict[str, Any]]:
        """Получение списка плагинов"""
        result = []
        
        for name, plugin in self.plugins.items():
            info = plugin.info
            result.append({
                'name': info.name,
                'version': info.version,
                'author': info.author,
                'description': info.description,
                'enabled': self.enabled.get(name, False)
            })
        
        return result


# Пример плагина
class ExamplePlugin(PluginBase):
    """Пример плагина для CrimeaAI"""
    
    @property
    def info(self) -> PluginInfo:
        return PluginInfo(
            name="Example Plugin",
            version="1.0.0",
            author="CrimeaAI Team",
            description="Демонстрационный плагин",
            dependencies=[]
        )
    
    def initialize(self, ecosystem: Any) -> bool:
        self.ecosystem = ecosystem
        print("🔌 Example Plugin initialized!")
        return True
    
    def shutdown(self):
        print("🔌 Example Plugin shutdown")
    
    def on_tick(self, dt: float):
        pass  # Вызывается каждый тик

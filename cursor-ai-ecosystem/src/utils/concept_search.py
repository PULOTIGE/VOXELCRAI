"""
Модуль поиска и интеграции концептов через DuckDuckGo
"""

import requests
import re
import time
from bs4 import BeautifulSoup
from typing import List, Dict, Set
import random


class ConceptSearcher:
    """
    Система поиска концептов в интернете.
    Периодически ищет новые идеи и термины для расширения базы знаний.
    """
    
    def __init__(self, keywords: List[str] = None):
        """
        Инициализация поисковика.
        
        Args:
            keywords: Начальные ключевые слова для поиска
        """
        self.keywords = keywords if keywords else ['19V', 'CrimeaAI', 'artificial intelligence', 'consciousness']
        self.discovered_concepts: Set[str] = set()
        self.concept_history: List[Dict] = []
        self.search_count = 0
        
        # User-Agent для обхода блокировок
        self.headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
        }
        
        # Лимиты
        self.max_results_per_search = 5
        self.max_concepts_per_page = 20
        self.request_timeout = 10
        
    def search_concepts(self, query: str = None) -> List[str]:
        """
        Поиск концептов по запросу.
        
        Args:
            query: Поисковый запрос (если None, используются случайные keywords)
            
        Returns:
            Список найденных концептов
        """
        if query is None:
            query = ' '.join(random.sample(self.keywords, min(3, len(self.keywords))))
        
        self.search_count += 1
        new_concepts = []
        
        try:
            # Формирование URL для DuckDuckGo
            url = f"https://duckduckgo.com/html/?q={requests.utils.quote(query)}"
            
            # Запрос
            response = requests.get(url, headers=self.headers, timeout=self.request_timeout)
            response.raise_for_status()
            
            # Парсинг результатов
            soup = BeautifulSoup(response.text, 'html.parser')
            
            # Поиск ссылок результатов
            results = soup.find_all('a', class_='result__a', limit=self.max_results_per_search)
            
            for result in results:
                try:
                    # Извлечение заголовка
                    title = result.get_text(strip=True)
                    if title:
                        concepts = self.extract_concepts_from_text(title)
                        new_concepts.extend(concepts)
                    
                    # Попытка получить содержимое страницы (опционально)
                    # В продакшене это может быть слишком медленно
                    # link = result.get('href')
                    # if link:
                    #     page_concepts = self._fetch_page_concepts(link)
                    #     new_concepts.extend(page_concepts)
                    
                except Exception as e:
                    print(f"Ошибка обработки результата: {e}")
                    continue
            
            # Сохранение истории
            self.concept_history.append({
                'query': query,
                'timestamp': time.time(),
                'concepts_found': len(new_concepts)
            })
            
            # Обновление множества концептов
            self.discovered_concepts.update(new_concepts)
            
            return new_concepts
            
        except requests.RequestException as e:
            print(f"Ошибка сетевого запроса: {e}")
            return []
        except Exception as e:
            print(f"Неожиданная ошибка при поиске: {e}")
            return []
    
    def _fetch_page_concepts(self, url: str) -> List[str]:
        """
        Получение концептов со страницы.
        
        Args:
            url: URL страницы
            
        Returns:
            Список концептов
        """
        try:
            response = requests.get(url, headers=self.headers, timeout=self.request_timeout)
            response.raise_for_status()
            
            soup = BeautifulSoup(response.text, 'html.parser')
            
            # Извлечение текста
            text = soup.get_text(separator=' ', strip=True)
            
            # Ограничение длины текста
            text = text[:5000]  # первые 5000 символов
            
            return self.extract_concepts_from_text(text)
            
        except Exception as e:
            print(f"Ошибка загрузки страницы {url}: {e}")
            return []
    
    def extract_concepts_from_text(self, text: str) -> List[str]:
        """
        Извлечение концептов (терминов) из текста.
        
        Args:
            text: Исходный текст
            
        Returns:
            Список концептов
        """
        concepts = []
        
        # Поиск слов с заглавной буквы (возможные термины)
        # Паттерн: слова из 2+ символов, начинающиеся с заглавной
        capitalized_words = re.findall(r'\b[A-ZА-Я][a-zа-я]{1,}(?:\s+[A-ZА-Я][a-zа-я]+)*\b', text)
        
        for word in capitalized_words:
            # Фильтрация: не слишком длинные, не общие слова
            if 2 <= len(word) <= 50 and word not in ['The', 'This', 'That', 'These', 'Those']:
                concepts.append(word.strip())
        
        # Поиск аббревиатур (2-6 заглавных букв)
        abbreviations = re.findall(r'\b[A-ZА-Я]{2,6}\b', text)
        concepts.extend(abbreviations)
        
        # Поиск терминов в кавычках
        quoted_terms = re.findall(r'["\']([^"\']{2,50})["\']', text)
        concepts.extend(quoted_terms)
        
        # Удаление дубликатов и ограничение количества
        unique_concepts = list(set(concepts))[:self.max_concepts_per_page]
        
        return unique_concepts
    
    def add_keywords(self, new_keywords: List[str]):
        """
        Добавление новых ключевых слов для поиска.
        
        Args:
            new_keywords: Список новых ключевых слов
        """
        self.keywords.extend(new_keywords)
        self.keywords = list(set(self.keywords))  # удаление дубликатов
    
    def get_random_concepts(self, count: int = 10) -> List[str]:
        """
        Получение случайных концептов из найденных.
        
        Args:
            count: Количество концептов
            
        Returns:
            Список случайных концептов
        """
        concepts_list = list(self.discovered_concepts)
        return random.sample(concepts_list, min(count, len(concepts_list)))
    
    def get_stats(self) -> Dict:
        """Получение статистики поиска."""
        return {
            'total_searches': self.search_count,
            'unique_concepts': len(self.discovered_concepts),
            'keywords_count': len(self.keywords),
            'recent_searches': len(self.concept_history)
        }
    
    def __repr__(self) -> str:
        return f"ConceptSearcher(searches={self.search_count}, concepts={len(self.discovered_concepts)})"


class ConceptIntegrator:
    """
    Интеграция найденных концептов в систему.
    """
    
    def __init__(self):
        """Инициализация интегратора."""
        self.integrated_concepts: Dict[str, Dict] = {}
        self.concept_graph: Dict[str, Set[str]] = {}  # граф связей между концептами
        
    def integrate_concept(self, concept: str, source: str = "search", metadata: Dict = None):
        """
        Интеграция концепта в систему.
        
        Args:
            concept: Концепт для интеграции
            source: Источник концепта
            metadata: Дополнительные метаданные
        """
        if concept not in self.integrated_concepts:
            self.integrated_concepts[concept] = {
                'name': concept,
                'source': source,
                'timestamp': time.time(),
                'frequency': 1,
                'metadata': metadata or {}
            }
            self.concept_graph[concept] = set()
        else:
            self.integrated_concepts[concept]['frequency'] += 1
    
    def link_concepts(self, concept1: str, concept2: str):
        """
        Создание связи между концептами.
        
        Args:
            concept1: Первый концепт
            concept2: Второй концепт
        """
        if concept1 in self.concept_graph:
            self.concept_graph[concept1].add(concept2)
        if concept2 in self.concept_graph:
            self.concept_graph[concept2].add(concept1)
    
    def get_related_concepts(self, concept: str, max_depth: int = 2) -> Set[str]:
        """
        Получение связанных концептов.
        
        Args:
            concept: Исходный концепт
            max_depth: Максимальная глубина поиска
            
        Returns:
            Множество связанных концептов
        """
        if concept not in self.concept_graph:
            return set()
        
        related = set()
        queue = [(concept, 0)]
        visited = {concept}
        
        while queue:
            current, depth = queue.pop(0)
            
            if depth >= max_depth:
                continue
            
            for neighbor in self.concept_graph[current]:
                if neighbor not in visited:
                    related.add(neighbor)
                    visited.add(neighbor)
                    queue.append((neighbor, depth + 1))
        
        return related
    
    def get_top_concepts(self, n: int = 10) -> List[Tuple[str, int]]:
        """
        Получение топ-N концептов по частоте.
        
        Args:
            n: Количество концептов
            
        Returns:
            Список (концепт, частота)
        """
        concepts_with_freq = [(c, data['frequency']) 
                             for c, data in self.integrated_concepts.items()]
        concepts_with_freq.sort(key=lambda x: x[1], reverse=True)
        return concepts_with_freq[:n]
    
    def get_stats(self) -> Dict:
        """Получение статистики интегратора."""
        return {
            'total_concepts': len(self.integrated_concepts),
            'total_links': sum(len(links) for links in self.concept_graph.values()) // 2,
            'avg_connections': sum(len(links) for links in self.concept_graph.values()) / max(len(self.concept_graph), 1)
        }

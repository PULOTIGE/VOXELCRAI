"""
Concept Search - Поиск и загрузка концептов
===========================================

Модуль для поиска новых концептов через DuckDuckGo и их интеграции
в память системы.
"""

import re
import time
import asyncio
from dataclasses import dataclass, field
from typing import List, Optional, Dict, Set, Tuple
from collections import Counter
import random


# Попытка импорта requests и BeautifulSoup
try:
    import requests
    from bs4 import BeautifulSoup
    SCRAPING_AVAILABLE = True
except ImportError:
    SCRAPING_AVAILABLE = False
    print("⚠️ requests/beautifulsoup4 не установлены. Поиск концептов будет симулирован.")


@dataclass
class Concept:
    """Концепт - единица знания"""
    term: str                              # Термин
    definition: str = ""                   # Определение
    source_url: str = ""                   # Источник
    related_terms: List[str] = field(default_factory=list)  # Связанные термины
    embedding: Optional[List[float]] = None  # Векторное представление
    importance: float = 1.0                # Важность
    discovery_time: float = 0.0            # Время обнаружения
    access_count: int = 0                  # Количество обращений
    
    def to_dict(self) -> dict:
        return {
            'term': self.term,
            'definition': self.definition,
            'source_url': self.source_url,
            'related_terms': self.related_terms,
            'importance': self.importance,
            'discovery_time': self.discovery_time
        }


class ConceptExtractor:
    """Извлечение концептов из текста"""
    
    # Паттерны для извлечения терминов
    TERM_PATTERNS = [
        r'\b[A-Z][a-z]+(?:\s+[A-Z][a-z]+)+\b',  # CamelCase phrases
        r'\b[A-Z]{2,}\b',                         # Аббревиатуры
        r'\b\w+(?:tion|ment|ness|ity|ism)\b',    # Существительные
    ]
    
    # Стоп-слова
    STOP_WORDS = {
        'the', 'a', 'an', 'is', 'are', 'was', 'were', 'be', 'been',
        'have', 'has', 'had', 'do', 'does', 'did', 'will', 'would',
        'could', 'should', 'may', 'might', 'must', 'shall', 'can',
        'this', 'that', 'these', 'those', 'it', 'its', 'with', 'for',
        'from', 'into', 'onto', 'upon', 'about', 'above', 'below',
        'between', 'under', 'over', 'through', 'during', 'before',
        'after', 'while', 'where', 'when', 'what', 'which', 'who',
        'whom', 'whose', 'why', 'how', 'all', 'each', 'every', 'both',
        'few', 'more', 'most', 'other', 'some', 'such', 'than', 'too',
        'very', 'just', 'only', 'own', 'same', 'so', 'then', 'there'
    }
    
    def __init__(self, min_term_length: int = 3, max_term_length: int = 50):
        self.min_term_length = min_term_length
        self.max_term_length = max_term_length
    
    def extract_terms(self, text: str) -> List[str]:
        """
        Извлечение терминов из текста
        
        Args:
            text: исходный текст
        
        Returns:
            Список извлечённых терминов
        """
        terms = []
        
        # Применяем паттерны
        for pattern in self.TERM_PATTERNS:
            matches = re.findall(pattern, text)
            terms.extend(matches)
        
        # Извлекаем n-граммы
        words = text.split()
        for i in range(len(words)):
            # Униграммы
            word = words[i].strip('.,!?;:()[]{}"\'-')
            if self._is_valid_term(word):
                terms.append(word.lower())
            
            # Биграммы
            if i < len(words) - 1:
                bigram = f"{words[i]} {words[i+1]}".strip('.,!?;:()[]{}"\'-')
                if len(bigram.split()) == 2 and len(bigram) <= self.max_term_length:
                    terms.append(bigram.lower())
        
        # Удаляем дубликаты, сохраняя порядок
        seen = set()
        unique_terms = []
        for term in terms:
            if term not in seen:
                seen.add(term)
                unique_terms.append(term)
        
        return unique_terms
    
    def _is_valid_term(self, term: str) -> bool:
        """Проверка валидности термина"""
        term_lower = term.lower()
        
        if len(term) < self.min_term_length:
            return False
        if len(term) > self.max_term_length:
            return False
        if term_lower in self.STOP_WORDS:
            return False
        if not any(c.isalpha() for c in term):
            return False
        
        return True
    
    def rank_terms(self, terms: List[str], context: str = "") -> List[Tuple[str, float]]:
        """
        Ранжирование терминов по важности
        
        Args:
            terms: список терминов
            context: контекст для оценки
        
        Returns:
            Список (термин, важность)
        """
        # Считаем частоту
        term_counts = Counter(terms)
        
        # Вычисляем важность
        ranked = []
        for term, count in term_counts.items():
            importance = count / len(terms) if terms else 0
            
            # Бонус за длину (более длинные термины часто более специфичны)
            length_bonus = min(len(term) / 20, 0.5)
            
            # Бонус за CamelCase или аббревиатуры
            if term[0].isupper():
                importance *= 1.5
            if term.isupper():
                importance *= 2.0
            
            # Бонус за присутствие в контексте
            if context and term.lower() in context.lower():
                importance *= 1.2
            
            ranked.append((term, importance + length_bonus))
        
        # Сортируем по важности
        ranked.sort(key=lambda x: x[1], reverse=True)
        
        return ranked


class ConceptSearcher:
    """
    Поисковик концептов через DuckDuckGo
    
    Осуществляет поиск новых знаний и их интеграцию в систему.
    """
    
    USER_AGENTS = [
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
        'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15',
        'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36',
    ]
    
    def __init__(
        self,
        base_keywords: Optional[List[str]] = None,
        search_interval: float = 19 * 60  # 19 минут
    ):
        """
        Создание поисковика
        
        Args:
            base_keywords: базовые ключевые слова для поиска
            search_interval: интервал между поисками (секунды)
        """
        self.base_keywords = base_keywords or ['AI', 'machine learning', 'neural network']
        self.search_interval = search_interval
        
        self.extractor = ConceptExtractor()
        
        # База концептов
        self.concepts: Dict[str, Concept] = {}
        
        # История поисков
        self.search_history: List[dict] = []
        self.last_search_time = 0.0
        
        # Статистика
        self.total_searches = 0
        self.total_concepts_found = 0
        self.failed_searches = 0
    
    def search_concepts(self, keywords: Optional[List[str]] = None) -> List[Concept]:
        """
        Поиск концептов по ключевым словам
        
        Args:
            keywords: ключевые слова (или используются базовые)
        
        Returns:
            Список найденных концептов
        """
        if keywords is None:
            keywords = self.base_keywords
        
        self.total_searches += 1
        self.last_search_time = time.time()
        
        concepts = []
        
        if SCRAPING_AVAILABLE:
            concepts = self._search_duckduckgo(keywords)
        else:
            concepts = self._simulate_search(keywords)
        
        # Записываем в историю
        self.search_history.append({
            'time': self.last_search_time,
            'keywords': keywords,
            'concepts_found': len(concepts)
        })
        
        # Ограничиваем историю
        if len(self.search_history) > 100:
            self.search_history = self.search_history[-100:]
        
        # Добавляем в базу
        for concept in concepts:
            self._add_concept(concept)
        
        self.total_concepts_found += len(concepts)
        
        return concepts
    
    def _search_duckduckgo(self, keywords: List[str]) -> List[Concept]:
        """Реальный поиск через DuckDuckGo"""
        concepts = []
        query = ' '.join(keywords)
        
        try:
            url = f"https://duckduckgo.com/html/?q={query}"
            headers = {
                'User-Agent': random.choice(self.USER_AGENTS),
                'Accept': 'text/html,application/xhtml+xml',
                'Accept-Language': 'en-US,en;q=0.9',
            }
            
            response = requests.get(url, headers=headers, timeout=10)
            response.raise_for_status()
            
            soup = BeautifulSoup(response.text, 'html.parser')
            
            # Извлекаем результаты
            results = soup.find_all('a', class_='result__a')[:5]
            
            for result in results:
                link_url = result.get('href', '')
                title = result.get_text(strip=True)
                
                # Извлекаем термины из заголовка
                terms = self.extractor.extract_terms(title)
                ranked = self.extractor.rank_terms(terms, title)
                
                for term, importance in ranked[:3]:
                    concept = Concept(
                        term=term,
                        definition=title,
                        source_url=link_url,
                        importance=importance,
                        discovery_time=time.time()
                    )
                    concepts.append(concept)
                
                # Пытаемся загрузить страницу для извлечения большего контента
                try:
                    page_response = requests.get(link_url, headers=headers, timeout=5)
                    if page_response.status_code == 200:
                        page_soup = BeautifulSoup(page_response.text, 'html.parser')
                        
                        # Извлекаем текст
                        text = page_soup.get_text()[:5000]
                        page_terms = self.extractor.extract_terms(text)
                        page_ranked = self.extractor.rank_terms(page_terms, text)
                        
                        for term, importance in page_ranked[:5]:
                            if term not in [c.term for c in concepts]:
                                concept = Concept(
                                    term=term,
                                    source_url=link_url,
                                    importance=importance,
                                    discovery_time=time.time()
                                )
                                concepts.append(concept)
                except:
                    pass
                
                # Задержка между запросами
                time.sleep(0.5)
        
        except Exception as e:
            print(f"⚠️ Ошибка поиска: {e}")
            self.failed_searches += 1
        
        return concepts
    
    def _simulate_search(self, keywords: List[str]) -> List[Concept]:
        """Симуляция поиска (когда requests недоступен)"""
        # Генерируем случайные концепты на основе ключевых слов
        simulated_concepts = [
            "neural architecture", "deep learning", "gradient descent",
            "backpropagation", "attention mechanism", "transformer model",
            "convolutional network", "recurrent network", "generative model",
            "reinforcement learning", "policy gradient", "value function",
            "embedding space", "latent representation", "feature extraction",
            "batch normalization", "dropout regularization", "weight decay",
            "learning rate", "optimizer algorithm", "loss function",
            "activation function", "softmax layer", "pooling operation"
        ]
        
        concepts = []
        for _ in range(random.randint(3, 8)):
            term = random.choice(simulated_concepts)
            if term not in [c.term for c in concepts]:
                concept = Concept(
                    term=term,
                    definition=f"Simulated concept related to {', '.join(keywords)}",
                    importance=random.uniform(0.3, 1.0),
                    discovery_time=time.time()
                )
                concepts.append(concept)
        
        return concepts
    
    def _add_concept(self, concept: Concept):
        """Добавление концепта в базу"""
        if concept.term in self.concepts:
            # Обновляем существующий
            existing = self.concepts[concept.term]
            existing.access_count += 1
            existing.importance = max(existing.importance, concept.importance)
            if concept.definition and not existing.definition:
                existing.definition = concept.definition
        else:
            self.concepts[concept.term] = concept
    
    async def search_async(self, keywords: Optional[List[str]] = None) -> List[Concept]:
        """Асинхронный поиск концептов"""
        # Выполняем в executor для неблокирующего поиска
        loop = asyncio.get_event_loop()
        return await loop.run_in_executor(None, self.search_concepts, keywords)
    
    def get_concept(self, term: str) -> Optional[Concept]:
        """Получение концепта по термину"""
        if term in self.concepts:
            self.concepts[term].access_count += 1
            return self.concepts[term]
        return None
    
    def get_related_concepts(self, term: str, top_k: int = 5) -> List[Concept]:
        """Получение связанных концептов"""
        if term not in self.concepts:
            return []
        
        target = self.concepts[term]
        
        # Простой поиск по пересечению слов
        target_words = set(term.lower().split())
        
        related = []
        for other_term, other_concept in self.concepts.items():
            if other_term == term:
                continue
            
            other_words = set(other_term.lower().split())
            overlap = len(target_words & other_words)
            
            if overlap > 0:
                related.append((other_concept, overlap))
        
        related.sort(key=lambda x: x[1], reverse=True)
        return [c for c, _ in related[:top_k]]
    
    def get_top_concepts(self, top_k: int = 10) -> List[Concept]:
        """Получение топ концептов по важности"""
        sorted_concepts = sorted(
            self.concepts.values(),
            key=lambda c: c.importance * (1 + c.access_count * 0.1),
            reverse=True
        )
        return sorted_concepts[:top_k]
    
    def get_statistics(self) -> dict:
        """Получение статистики поиска"""
        return {
            'total_concepts': len(self.concepts),
            'total_searches': self.total_searches,
            'total_concepts_found': self.total_concepts_found,
            'failed_searches': self.failed_searches,
            'last_search_time': self.last_search_time,
            'search_interval': self.search_interval,
            'base_keywords': self.base_keywords
        }
    
    def save(self, filepath: str):
        """Сохранение базы концептов"""
        import json
        
        data = {
            'concepts': {k: v.to_dict() for k, v in self.concepts.items()},
            'statistics': self.get_statistics()
        }
        
        with open(filepath, 'w', encoding='utf-8') as f:
            json.dump(data, f, ensure_ascii=False, indent=2)
        
        print(f"📚 База концептов сохранена в {filepath}")
    
    def load(self, filepath: str):
        """Загрузка базы концептов"""
        import json
        
        with open(filepath, 'r', encoding='utf-8') as f:
            data = json.load(f)
        
        for term, concept_data in data.get('concepts', {}).items():
            self.concepts[term] = Concept(**concept_data)
        
        print(f"📚 Загружено {len(self.concepts)} концептов из {filepath}")


class ConceptIntegrator:
    """
    Интегратор концептов в память системы
    
    Преобразует концепты в векторные представления и
    интегрирует их в нуклеотиды/воксели.
    """
    
    def __init__(self, vector_size: int = 512):
        """
        Создание интегратора
        
        Args:
            vector_size: размер векторного представления
        """
        self.vector_size = vector_size
        
        # Простая модель для создания эмбеддингов
        self._char_to_idx = {chr(i): i - 32 for i in range(32, 127)}
    
    def concept_to_vector(self, concept: Concept) -> List[float]:
        """
        Преобразование концепта в вектор
        
        Args:
            concept: концепт
        
        Returns:
            Векторное представление
        """
        # Простой метод: хеширование символов
        text = f"{concept.term} {concept.definition}"
        
        vector = [0.0] * self.vector_size
        
        for i, char in enumerate(text[:self.vector_size]):
            idx = self._char_to_idx.get(char, 0)
            pos = i % self.vector_size
            vector[pos] += (idx / 95.0) * 0.1  # Нормализуем
        
        # Добавляем позиционное кодирование
        import math
        for i in range(self.vector_size):
            vector[i] += math.sin(i / 10.0) * 0.05 * concept.importance
        
        # Нормализуем
        norm = sum(v * v for v in vector) ** 0.5
        if norm > 0:
            vector = [v / norm for v in vector]
        
        return vector
    
    def integrate_into_nucleotide(self, concept: Concept, nucleotide) -> None:
        """
        Интеграция концепта в нуклеотид
        
        Args:
            concept: концепт для интеграции
            nucleotide: целевой нуклеотид
        """
        import numpy as np
        
        vector = self.concept_to_vector(concept)
        experience = np.array(vector, dtype=np.float16)
        
        # Интегрируем как опыт
        nucleotide.update(0.016, experience)
    
    def integrate_into_voxel(self, concept: Concept, voxel) -> None:
        """
        Интеграция концепта в воксель
        
        Args:
            concept: концепт
            voxel: целевой воксель
        """
        import numpy as np
        
        vector = self.concept_to_vector(concept)
        experience = np.array(vector[:64], dtype=np.float32)
        
        # Сохраняем в память вокселя
        voxel.memory.store(experience, importance=concept.importance)
        
        # Добавляем как мысль
        from .voxel import ThoughtType
        voxel.thoughts.add_thought(ThoughtType.OBSERVATION, experience)

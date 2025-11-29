package net.voxelcrai.pattern;

import net.minecraft.block.BlockState;
import net.minecraft.util.math.BlockPos;
import net.minecraft.util.math.Direction;
import net.minecraft.world.chunk.WorldChunk;
import net.voxelcrai.config.VoxelCraiConfig;
import net.voxelcrai.mod.VoxelCraiMod;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.function.Consumer;

/**
 * 🔮 PatternGenerator - Генератор SH паттернов освещения
 * 
 * Генерирует LightPattern1KB на основе данных чанков:
 * - Анализ блоков и их светопропускания
 * - Вычисление SH коэффициентов для GI/теней
 * - Определение материалов (roughness, metallic)
 * - Обновление при изменении времени суток/погоды
 * 
 * Алгоритм SH:
 * 1. Сэмплирование направлений вокруг точки (64-256 лучей)
 * 2. Проекция на SH базисные функции (до 4 bands)
 * 3. Компрессия коэффициентов в i8 [-127, 127]
 */
public class PatternGenerator {
    
    // 🔧 Конфигурация
    private final VoxelCraiConfig config;
    
    // 🧵 Thread pool для асинхронной генерации
    private final ExecutorService executor;
    
    // 📍 Отслеживание чанков
    private final ConcurrentHashMap<Long, List<Long>> chunkPatternMap;
    
    // 🎲 ID генератор
    private long nextPatternId;
    
    // 🔮 SH базисные функции (предвычисленные)
    // Band 0: 1 коэффициент
    // Band 1: 3 коэффициента  
    // Band 2: 5 коэффициентов
    // Band 3: 7 коэффициентов
    private static final float[] SH_CONSTANTS = {
        // Band 0
        0.282095f,  // Y00
        // Band 1
        0.488603f,  // Y1-1
        0.488603f,  // Y10
        0.488603f,  // Y11
        // Band 2
        1.092548f,  // Y2-2
        1.092548f,  // Y2-1
        0.315392f,  // Y20
        1.092548f,  // Y21
        0.546274f,  // Y22
        // Band 3
        0.590044f,  // Y3-3
        2.890611f,  // Y3-2
        0.457046f,  // Y3-1
        0.373176f,  // Y30
        0.457046f,  // Y31
        2.890611f,  // Y32
        0.590044f   // Y33
    };
    
    // 📐 Направления сэмплирования (фибоначчиева сфера)
    private final float[][] sampleDirections;
    private static final int SAMPLE_COUNT = 64;  // Количество сэмплов на точку
    
    /**
     * 🏗️ Конструктор
     */
    public PatternGenerator(VoxelCraiConfig config) {
        this.config = config;
        this.executor = Executors.newFixedThreadPool(
            Math.max(2, Runtime.getRuntime().availableProcessors() / 2)
        );
        this.chunkPatternMap = new ConcurrentHashMap<>();
        this.nextPatternId = 1;
        this.sampleDirections = generateFibonacciSphere(SAMPLE_COUNT);
        
        VoxelCraiMod.LOGGER.info("🔮 PatternGenerator: {} sample directions", SAMPLE_COUNT);
    }
    
    /**
     * 🌍 Асинхронная генерация паттернов для чанка
     */
    public void generateForChunkAsync(WorldChunk chunk, Consumer<List<LightPattern1KB>> callback) {
        executor.submit(() -> {
            try {
                List<LightPattern1KB> patterns = generateForChunk(chunk);
                callback.accept(patterns);
            } catch (Exception e) {
                VoxelCraiMod.LOGGER.error("❌ Ошибка генерации паттернов: {}", e.getMessage());
            }
        });
    }
    
    /**
     * 🌍 Генерация паттернов для чанка (синхронная)
     */
    public List<LightPattern1KB> generateForChunk(WorldChunk chunk) {
        List<LightPattern1KB> patterns = new ArrayList<>();
        
        int chunkX = chunk.getPos().x;
        int chunkZ = chunk.getPos().z;
        long chunkKey = getChunkKey(chunkX, chunkZ);
        
        // 📍 Итерация по блокам чанка (с прореживанием для производительности)
        int step = config.getPatternDensity();  // 1 = каждый блок, 2 = каждый второй, и т.д.
        
        for (int x = 0; x < 16; x += step) {
            for (int z = 0; z < 16; z += step) {
                // Получаем высоту в этой точке
                int maxY = chunk.getHighestNonEmptySectionYOffset() + 16;
                int minY = chunk.getBottomY();
                
                for (int y = minY; y < maxY; y += step) {
                    BlockPos pos = new BlockPos(
                        chunk.getPos().getStartX() + x,
                        y,
                        chunk.getPos().getStartZ() + z
                    );
                    
                    BlockState state = chunk.getBlockState(pos);
                    
                    // 🚫 Пропускаем воздух
                    if (state.isAir()) continue;
                    
                    // 🔮 Генерируем паттерн для непрозрачных/полупрозрачных блоков
                    if (!state.isTransparent()) {
                        LightPattern1KB pattern = generatePatternForBlock(chunk, pos, state);
                        patterns.add(pattern);
                        
                        // Ограничение количества паттернов на чанк
                        if (patterns.size() >= config.getMaxPatternsPerChunk()) {
                            break;
                        }
                    }
                }
                
                if (patterns.size() >= config.getMaxPatternsPerChunk()) {
                    break;
                }
            }
            
            if (patterns.size() >= config.getMaxPatternsPerChunk()) {
                break;
            }
        }
        
        // 📝 Сохраняем ID паттернов для этого чанка
        List<Long> patternIds = new ArrayList<>();
        for (LightPattern1KB pattern : patterns) {
            patternIds.add(pattern.getId());
        }
        chunkPatternMap.put(chunkKey, patternIds);
        
        VoxelCraiMod.LOGGER.debug("✨ Чанк [{}, {}]: {} паттернов", chunkX, chunkZ, patterns.size());
        
        return patterns;
    }
    
    /**
     * 💡 Генерация паттерна для отдельного блока
     */
    private LightPattern1KB generatePatternForBlock(WorldChunk chunk, BlockPos pos, BlockState state) {
        LightPattern1KB pattern = new LightPattern1KB(nextPatternId++);
        
        // 📍 Позиция
        pattern.setPosition(pos.getX(), pos.getY(), pos.getZ());
        
        // 🔮 Вычисляем SH коэффициенты
        byte[] shCoeffs = computeShCoefficients(chunk, pos);
        pattern.setShCoefficients4Bands(shCoeffs);
        
        // 💡 Прямое освещение (от неба/солнца)
        // В 1.21.3+ используем World для получения уровня освещения
        float skyLight = 0.8f;  // Default sky light
        float blockLight = 0.0f;
        if (chunk.getWorld() != null) {
            skyLight = chunk.getWorld().getLightLevel(net.minecraft.world.LightType.SKY, pos) / 15.0f;
            blockLight = chunk.getWorld().getLightLevel(net.minecraft.world.LightType.BLOCK, pos) / 15.0f;
        }
        
        pattern.setDirectLight(skyLight, skyLight * 0.9f, skyLight * 0.8f);
        
        // 🌙 Непрямое освещение (bounce light аппроксимация)
        float indirectStrength = computeIndirectLighting(chunk, pos);
        pattern.setIndirectLight(
            indirectStrength * 0.8f,
            indirectStrength * 0.85f,
            indirectStrength * 0.9f
        );
        
        // 🎨 Материал (на основе типа блока)
        MaterialProperties mat = getMaterialForBlock(state);
        pattern.setRoughness(mat.roughness);
        pattern.setMetallic(mat.metallic);
        
        // ✨ AO, отражения
        float ao = computeAmbientOcclusion(chunk, pos);
        pattern.setAmbientOcclusion(ao);
        pattern.setReflection(mat.metallic * (1.0f - mat.roughness));
        
        // 🔥 Эмиссия (для светящихся блоков)
        if (blockLight > 0.5f) {
            pattern.setEmission(blockLight);
        }
        
        return pattern;
    }
    
    /**
     * 🔮 Вычисление SH коэффициентов для точки
     * 
     * Алгоритм:
     * 1. Сэмплируем видимость в направлениях сферы
     * 2. Проецируем на SH базис
     * 3. Нормализуем в [-127, 127]
     */
    private byte[] computeShCoefficients(WorldChunk chunk, BlockPos pos) {
        byte[] coeffs = new byte[16];  // 4 bands
        float[] shValues = new float[16];
        
        // 🎯 Сэмплирование направлений
        for (int i = 0; i < SAMPLE_COUNT; i++) {
            float[] dir = sampleDirections[i];
            
            // Проверяем видимость в этом направлении
            float visibility = traceVisibility(chunk, pos, dir);
            
            // Проецируем на SH базис
            projectToSH(dir, visibility, shValues);
        }
        
        // Нормализация и усреднение
        float scale = 4.0f * (float) Math.PI / SAMPLE_COUNT;
        
        for (int i = 0; i < 16; i++) {
            shValues[i] *= scale;
            // Конвертация в i8 [-127, 127]
            coeffs[i] = (byte) Math.max(-127, Math.min(127, (int) (shValues[i] * 127.0f)));
        }
        
        return coeffs;
    }
    
    /**
     * 📐 Проекция направления на SH базисные функции
     */
    private void projectToSH(float[] dir, float value, float[] shValues) {
        float x = dir[0];
        float y = dir[1];
        float z = dir[2];
        
        // Band 0
        shValues[0] += value * 0.282095f;
        
        // Band 1
        shValues[1] += value * 0.488603f * y;
        shValues[2] += value * 0.488603f * z;
        shValues[3] += value * 0.488603f * x;
        
        // Band 2
        shValues[4] += value * 1.092548f * x * y;
        shValues[5] += value * 1.092548f * y * z;
        shValues[6] += value * 0.315392f * (3.0f * z * z - 1.0f);
        shValues[7] += value * 1.092548f * x * z;
        shValues[8] += value * 0.546274f * (x * x - y * y);
        
        // Band 3 (опционально, для высокого качества)
        if (config.getShBands() >= 4) {
            shValues[9] += value * 0.590044f * y * (3.0f * x * x - y * y);
            shValues[10] += value * 2.890611f * x * y * z;
            shValues[11] += value * 0.457046f * y * (4.0f * z * z - x * x - y * y);
            shValues[12] += value * 0.373176f * z * (2.0f * z * z - 3.0f * x * x - 3.0f * y * y);
            shValues[13] += value * 0.457046f * x * (4.0f * z * z - x * x - y * y);
            shValues[14] += value * 2.890611f * z * (x * x - y * y);
            shValues[15] += value * 0.590044f * x * (x * x - 3.0f * y * y);
        }
    }
    
    /**
     * 👁️ Трассировка видимости в направлении
     */
    private float traceVisibility(WorldChunk chunk, BlockPos origin, float[] direction) {
        int maxDistance = 8;  // Максимальная дистанция трассировки
        
        float visibility = 1.0f;
        
        for (int step = 1; step <= maxDistance; step++) {
            int x = origin.getX() + Math.round(direction[0] * step);
            int y = origin.getY() + Math.round(direction[1] * step);
            int z = origin.getZ() + Math.round(direction[2] * step);
            
            BlockPos checkPos = new BlockPos(x, y, z);
            
            // Проверяем только в пределах чанка для производительности
            if (!isInChunk(chunk, checkPos)) {
                break;
            }
            
            BlockState state = chunk.getBlockState(checkPos);
            
            if (!state.isAir()) {
                if (state.isTransparent()) {
                    visibility *= 0.7f;  // Полупрозрачный
                } else {
                    visibility *= 0.1f;  // Непрозрачный = тень
                }
            }
        }
        
        return visibility;
    }
    
    /**
     * 🌙 Вычисление непрямого освещения
     */
    private float computeIndirectLighting(WorldChunk chunk, BlockPos pos) {
        float totalLight = 0.0f;
        int samples = 0;
        
        // Сэмплируем соседние блоки
        for (Direction dir : Direction.values()) {
            BlockPos neighbor = pos.offset(dir);
            
            if (isInChunk(chunk, neighbor) && chunk.getWorld() != null) {
                float skyLight = chunk.getWorld().getLightLevel(net.minecraft.world.LightType.SKY, neighbor) / 15.0f;
                float blockLight = chunk.getWorld().getLightLevel(net.minecraft.world.LightType.BLOCK, neighbor) / 15.0f;
                totalLight += Math.max(skyLight, blockLight);
                samples++;
            }
        }
        
        return samples > 0 ? totalLight / samples * 0.5f : 0.0f;
    }
    
    /**
     * 🌑 Вычисление Ambient Occlusion
     */
    private float computeAmbientOcclusion(WorldChunk chunk, BlockPos pos) {
        int occluded = 0;
        int total = 0;
        
        // Проверяем окклюзию в 26 соседних позициях (3x3x3 куб)
        for (int dx = -1; dx <= 1; dx++) {
            for (int dy = -1; dy <= 1; dy++) {
                for (int dz = -1; dz <= 1; dz++) {
                    if (dx == 0 && dy == 0 && dz == 0) continue;
                    
                    BlockPos neighbor = pos.add(dx, dy, dz);
                    total++;
                    
                    if (isInChunk(chunk, neighbor)) {
                        BlockState state = chunk.getBlockState(neighbor);
                        if (!state.isAir() && !state.isTransparent()) {
                            occluded++;
                        }
                    }
                }
            }
        }
        
        // AO = 1.0 (нет окклюзии) до 0.0 (полная окклюзия)
        return 1.0f - (occluded / (float) total);
    }
    
    /**
     * 🎨 Получение материала для блока
     */
    private MaterialProperties getMaterialForBlock(BlockState state) {
        String blockName = state.getBlock().getTranslationKey();
        
        // 🪨 Камень, земля
        if (blockName.contains("stone") || blockName.contains("dirt") || blockName.contains("grass")) {
            return new MaterialProperties(0.9f, 0.0f);  // Rough, non-metallic
        }
        
        // ⛏️ Руды, металлы
        if (blockName.contains("ore") || blockName.contains("iron") || blockName.contains("gold") ||
            blockName.contains("copper") || blockName.contains("diamond")) {
            return new MaterialProperties(0.3f, 0.8f);  // Smooth, metallic
        }
        
        // 🪵 Дерево
        if (blockName.contains("wood") || blockName.contains("log") || blockName.contains("plank")) {
            return new MaterialProperties(0.8f, 0.0f);  // Rough, non-metallic
        }
        
        // 🪟 Стекло
        if (blockName.contains("glass")) {
            return new MaterialProperties(0.1f, 0.0f);  // Smooth, non-metallic
        }
        
        // 💧 Вода, лед
        if (blockName.contains("water") || blockName.contains("ice")) {
            return new MaterialProperties(0.05f, 0.0f);  // Very smooth
        }
        
        // 🌿 Листья, растения
        if (blockName.contains("leaves") || blockName.contains("flower") || blockName.contains("plant")) {
            return new MaterialProperties(0.95f, 0.0f);  // Very rough
        }
        
        // По умолчанию
        return new MaterialProperties(0.7f, 0.0f);
    }
    
    /**
     * 🔄 Обновление динамических паттернов (время суток, погода)
     */
    public void updateDynamicPatterns(float timeOfDay, float rainGradient) {
        LightPatternBuffer buffer = VoxelCraiMod.getInstance().getPatternBuffer();
        
        // 🌅 Модификатор времени суток
        // 0.0 = полночь, 0.25 = рассвет, 0.5 = полдень, 0.75 = закат
        float sunIntensity = (float) Math.max(0, Math.sin(timeOfDay * 2 * Math.PI));
        
        // 🌧️ Модификатор погоды
        float weatherMod = 1.0f - rainGradient * 0.5f;
        
        // Обновляем все паттерны
        for (LightPattern1KB pattern : buffer.getAllPatterns()) {
            // Модифицируем SH коэффициенты на основе времени
            byte[] coeffs = pattern.getShCoefficients();
            
            // Band 0 (ambient) зависит от солнца
            float ambientMod = 0.2f + sunIntensity * 0.8f * weatherMod;
            coeffs[0] = (byte) Math.max(-127, Math.min(127, coeffs[0] * ambientMod));
            
            // Band 1 (направленный свет) зависит от позиции солнца
            float sunAngle = timeOfDay * 2 * (float) Math.PI;
            float sunX = (float) Math.cos(sunAngle);
            float sunY = (float) Math.sin(sunAngle);
            
            coeffs[1] = (byte) Math.max(-127, Math.min(127, coeffs[1] + (int)(sunY * 50 * weatherMod)));
            coeffs[3] = (byte) Math.max(-127, Math.min(127, coeffs[3] + (int)(sunX * 50 * weatherMod)));
        }
        
        buffer.clearDirty();  // Помечаем как обновленный
    }
    
    /**
     * 🗑️ Освобождение паттернов чанка
     */
    public void releaseChunk(int chunkX, int chunkZ) {
        long chunkKey = getChunkKey(chunkX, chunkZ);
        List<Long> patternIds = chunkPatternMap.remove(chunkKey);
        
        if (patternIds != null) {
            LightPatternBuffer buffer = VoxelCraiMod.getInstance().getPatternBuffer();
            for (Long id : patternIds) {
                buffer.removePattern(id);
            }
        }
    }
    
    /**
     * 🛑 Остановка генератора
     */
    public void shutdown() {
        executor.shutdown();
    }
    
    // ========== 🔧 Вспомогательные методы ==========
    
    /**
     * 🔑 Получение ключа чанка
     */
    private long getChunkKey(int chunkX, int chunkZ) {
        return ((long) chunkX << 32) | (chunkZ & 0xFFFFFFFFL);
    }
    
    /**
     * 📍 Проверка, находится ли позиция в чанке
     */
    private boolean isInChunk(WorldChunk chunk, BlockPos pos) {
        int startX = chunk.getPos().getStartX();
        int startZ = chunk.getPos().getStartZ();
        
        return pos.getX() >= startX && pos.getX() < startX + 16 &&
               pos.getZ() >= startZ && pos.getZ() < startZ + 16 &&
               pos.getY() >= chunk.getBottomY() && pos.getY() < chunk.getTopYInclusive();
    }
    
    /**
     * 🌐 Генерация точек на сфере (фибоначчиево распределение)
     */
    private float[][] generateFibonacciSphere(int samples) {
        float[][] points = new float[samples][3];
        float phi = (float) Math.PI * (3.0f - (float) Math.sqrt(5.0f));  // Golden angle
        
        for (int i = 0; i < samples; i++) {
            float y = 1.0f - (i / (float)(samples - 1)) * 2.0f;  // y от 1 до -1
            float radius = (float) Math.sqrt(1.0f - y * y);
            float theta = phi * i;
            
            points[i][0] = (float) Math.cos(theta) * radius;
            points[i][1] = y;
            points[i][2] = (float) Math.sin(theta) * radius;
        }
        
        return points;
    }
    
    /**
     * 🎨 Класс для свойств материала
     */
    private static class MaterialProperties {
        final float roughness;
        final float metallic;
        
        MaterialProperties(float roughness, float metallic) {
            this.roughness = roughness;
            this.metallic = metallic;
        }
    }
}

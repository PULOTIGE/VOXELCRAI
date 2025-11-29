package net.voxelcrai.pattern;

import net.minecraft.block.Block;
import net.minecraft.block.BlockState;
import net.minecraft.block.Blocks;
import net.minecraft.util.math.BlockPos;
import net.minecraft.world.chunk.WorldChunk;
import net.voxelcrai.mod.VoxelCraiMod;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.atomic.AtomicLong;

/**
 * 🎨 МЕНЕДЖЕР ПАТТЕРНОВ ОСВЕЩЕНИЯ
 * 
 * Управляет генерацией, обновлением и хранением LightPattern1KB.
 * Поддерживает до 10,000 паттернов для GPU SSBO.
 * 
 * @author PULOTIGE
 */
public class LightPatternManager {
    
    // 📊 БУФЕР ПАТТЕРНОВ
    private final LightPattern1KB[] patterns;
    private final int maxPatterns;
    private final int shBands;
    
    // 🗺️ КЭШИ
    private final ConcurrentHashMap<Long, Integer> chunkPatternIndex;  // chunkKey -> startIndex
    private final ConcurrentHashMap<Long, Integer> blockToPattern;     // blockKey -> patternIndex
    
    // 🔢 СЧЁТЧИКИ
    private final AtomicLong nextPatternId;
    private int activePatternCount;
    
    // ⚡ ASYNC
    private final ExecutorService asyncExecutor;
    
    // 📦 GPU БУФЕР
    private ByteBuffer gpuBuffer;
    private boolean gpuBufferDirty;
    
    /**
     * 🏗️ Конструктор
     */
    public LightPatternManager(int maxPatterns, int shBands) {
        this.maxPatterns = maxPatterns;
        this.shBands = shBands;
        this.patterns = new LightPattern1KB[maxPatterns];
        this.chunkPatternIndex = new ConcurrentHashMap<>();
        this.blockToPattern = new ConcurrentHashMap<>();
        this.nextPatternId = new AtomicLong(1);
        this.activePatternCount = 0;
        this.asyncExecutor = Executors.newFixedThreadPool(2);
        
        // 🎨 Инициализация паттернов
        for (int i = 0; i < maxPatterns; i++) {
            patterns[i] = new LightPattern1KB(i);
        }
        
        // 📦 Создание GPU буфера
        this.gpuBuffer = ByteBuffer.allocateDirect(maxPatterns * LightPattern1KB.SIZE_BYTES);
        this.gpuBuffer.order(ByteOrder.LITTLE_ENDIAN);
        this.gpuBufferDirty = true;
        
        VoxelCraiMod.LOGGER.info("🎨 LightPatternManager создан: {} паттернов, {} SH полос", maxPatterns, shBands);
    }
    
    /**
     * 📦 Генерация паттернов для чанка
     */
    public void generatePatternsForChunk(int chunkX, int chunkZ, WorldChunk chunk) {
        if (VoxelCraiMod.getConfig().isAsyncPatternGeneration()) {
            asyncExecutor.submit(() -> generatePatternsForChunkSync(chunkX, chunkZ, chunk));
        } else {
            generatePatternsForChunkSync(chunkX, chunkZ, chunk);
        }
    }
    
    /**
     * 📦 Синхронная генерация паттернов для чанка
     */
    private void generatePatternsForChunkSync(int chunkX, int chunkZ, WorldChunk chunk) {
        long chunkKey = packChunkKey(chunkX, chunkZ);
        
        // 🔍 Сканируем блоки чанка (каждый 4-й для оптимизации)
        int patternsGenerated = 0;
        int maxPatternsPerChunk = 256;  // Ограничение на чанк
        
        for (int y = chunk.getBottomY(); y < chunk.getTopY() && patternsGenerated < maxPatternsPerChunk; y += 4) {
            for (int x = 0; x < 16 && patternsGenerated < maxPatternsPerChunk; x += 4) {
                for (int z = 0; z < 16 && patternsGenerated < maxPatternsPerChunk; z += 4) {
                    BlockState state = chunk.getBlockState(new BlockPos(x, y, z));
                    
                    if (!state.isAir()) {
                        int patternIdx = allocatePattern();
                        if (patternIdx >= 0) {
                            generatePatternForBlock(patternIdx, state, chunkX, chunkZ, x, y, z);
                            
                            long blockKey = packBlockKey(chunkX * 16 + x, y, chunkZ * 16 + z);
                            blockToPattern.put(blockKey, patternIdx);
                            patternsGenerated++;
                        }
                    }
                }
            }
        }
        
        gpuBufferDirty = true;
        VoxelCraiMod.LOGGER.debug("📦 Чанк [{},{}]: сгенерировано {} паттернов", chunkX, chunkZ, patternsGenerated);
    }
    
    /**
     * 🎨 Генерация паттерна для конкретного блока
     */
    private void generatePatternForBlock(int patternIdx, BlockState state, int chunkX, int chunkZ, int localX, int localY, int localZ) {
        LightPattern1KB pattern = patterns[patternIdx];
        Block block = state.getBlock();
        
        // 🆔 ID
        pattern.setId(nextPatternId.getAndIncrement());
        
        // 📍 Позиция
        pattern.setPosition(chunkX, chunkZ, localX, localY, localZ);
        
        // 🎨 Материал по типу блока
        if (block == Blocks.STONE || block == Blocks.COBBLESTONE || block == Blocks.DEEPSLATE) {
            pattern.setRoughness(0.8f);
            pattern.setMetallic(0.0f);
            pattern.setDirectLight(0.3f, 0.3f, 0.3f);
        } else if (block == Blocks.IRON_BLOCK || block == Blocks.GOLD_BLOCK) {
            pattern.setRoughness(0.3f);
            pattern.setMetallic(1.0f);
            pattern.setFlag(LightPattern1KB.FLAG_REFLECTIVE, true);
            pattern.setDirectLight(0.8f, 0.8f, 0.8f);
        } else if (block == Blocks.GLOWSTONE || block == Blocks.SEA_LANTERN) {
            pattern.setRoughness(0.5f);
            pattern.setMetallic(0.0f);
            pattern.setFlag(LightPattern1KB.FLAG_EMISSIVE, true);
            pattern.setDirectLight(1.0f, 0.9f, 0.7f);
            pattern.setIndirectLight(0.8f, 0.7f, 0.5f);
        } else if (block == Blocks.WATER) {
            pattern.setRoughness(0.1f);
            pattern.setMetallic(0.0f);
            pattern.setFlag(LightPattern1KB.FLAG_WATER, true);
            pattern.setFlag(LightPattern1KB.FLAG_TRANSPARENT, true);
            pattern.setDirectLight(0.2f, 0.4f, 0.6f);
        } else if (block == Blocks.GLASS || block == Blocks.GLASS_PANE) {
            pattern.setRoughness(0.05f);
            pattern.setMetallic(0.0f);
            pattern.setFlag(LightPattern1KB.FLAG_TRANSPARENT, true);
            pattern.setFlag(LightPattern1KB.FLAG_REFLECTIVE, true);
            pattern.setDirectLight(0.9f, 0.9f, 0.9f);
        } else if (block == Blocks.OAK_LEAVES || block == Blocks.BIRCH_LEAVES || block == Blocks.SPRUCE_LEAVES) {
            pattern.setRoughness(0.9f);
            pattern.setMetallic(0.0f);
            pattern.setFlag(LightPattern1KB.FLAG_FOLIAGE, true);
            pattern.setDirectLight(0.2f, 0.5f, 0.1f);
        } else if (block == Blocks.GRASS_BLOCK) {
            pattern.setRoughness(0.7f);
            pattern.setMetallic(0.0f);
            pattern.setDirectLight(0.3f, 0.5f, 0.2f);
        } else if (block == Blocks.SAND || block == Blocks.SANDSTONE) {
            pattern.setRoughness(0.6f);
            pattern.setMetallic(0.0f);
            pattern.setDirectLight(0.8f, 0.75f, 0.5f);
        } else {
            // 🔧 По умолчанию
            pattern.setRoughness(0.5f);
            pattern.setMetallic(0.0f);
            pattern.setDirectLight(0.5f, 0.5f, 0.5f);
        }
        
        // 🌐 SH коэффициенты (простая аппроксимация)
        float[] shCoeffs = generateSHCoeffs(localX, localY, localZ);
        pattern.setShCoeffs(shCoeffs);
        
        // 🏴 Флаги по умолчанию
        if (!state.isAir() && state.isOpaque()) {
            pattern.setFlag(LightPattern1KB.FLAG_SHADOW_CASTER, true);
        }
        pattern.setFlag(LightPattern1KB.FLAG_SHADOW_RECV, true);
    }
    
    /**
     * 🌐 Генерация SH коэффициентов (простая аппроксимация)
     */
    private float[] generateSHCoeffs(int x, int y, int z) {
        float[] coeffs = new float[9];
        
        // Band 0 (DC term) - ambient
        coeffs[0] = 0.5f;
        
        // Band 1 (linear terms) - directional
        float nx = (x - 8) / 8.0f;
        float ny = (y - 64) / 64.0f;
        float nz = (z - 8) / 8.0f;
        
        coeffs[1] = ny * 0.3f;  // Y direction (sky)
        coeffs[2] = nz * 0.2f;  // Z direction
        coeffs[3] = nx * 0.2f;  // X direction
        
        // Band 2 (quadratic terms) - indirect bounce
        coeffs[4] = nx * ny * 0.1f;
        coeffs[5] = ny * nz * 0.1f;
        coeffs[6] = (3 * nz * nz - 1) * 0.05f;
        coeffs[7] = nx * nz * 0.1f;
        coeffs[8] = (nx * nx - ny * ny) * 0.05f;
        
        return coeffs;
    }
    
    /**
     * 🗑️ Выгрузка паттернов чанка
     */
    public void unloadChunkPatterns(int chunkX, int chunkZ) {
        long chunkKey = packChunkKey(chunkX, chunkZ);
        
        // 🗑️ Удаляем ссылки на паттерны этого чанка
        blockToPattern.entrySet().removeIf(entry -> {
            long blockKey = entry.getKey();
            int bx = (int) ((blockKey >> 40) & 0xFFFFFF) - 0x800000;
            int bz = (int) (blockKey & 0xFFFFFF) - 0x800000;
            return (bx >> 4) == chunkX && (bz >> 4) == chunkZ;
        });
        
        chunkPatternIndex.remove(chunkKey);
        gpuBufferDirty = true;
    }
    
    /**
     * 🔄 Обновление динамических паттернов (каждый тик)
     */
    public void updateDynamicPatterns() {
        // 🔄 Обновляем эмиссивные блоки (пульсация света)
        for (int i = 0; i < activePatternCount; i++) {
            LightPattern1KB pattern = patterns[i];
            if (pattern.hasFlag(LightPattern1KB.FLAG_EMISSIVE)) {
                // 💡 Простая пульсация
                float time = System.currentTimeMillis() / 1000.0f;
                float pulse = (float) (0.9f + 0.1f * Math.sin(time * 2.0f + pattern.getId() * 0.1f));
                
                float r = pattern.getDirectLightR() * pulse;
                float g = pattern.getDirectLightG() * pulse;
                float b = pattern.getDirectLightB() * pulse;
                pattern.setDirectLight(r, g, b);
            }
        }
        
        if (gpuBufferDirty) {
            updateGpuBuffer();
        }
    }
    
    /**
     * 📦 Обновление GPU буфера
     */
    private void updateGpuBuffer() {
        gpuBuffer.clear();
        for (int i = 0; i < activePatternCount; i++) {
            patterns[i].writeToBuffer(gpuBuffer);
        }
        gpuBuffer.flip();
        gpuBufferDirty = false;
    }
    
    /**
     * 🔢 Выделение нового индекса паттерна
     */
    private synchronized int allocatePattern() {
        if (activePatternCount >= maxPatterns) {
            return -1;
        }
        return activePatternCount++;
    }
    
    // 🔧 УТИЛИТЫ
    
    private long packChunkKey(int x, int z) {
        return ((long) x << 32) | (z & 0xFFFFFFFFL);
    }
    
    private long packBlockKey(int x, int y, int z) {
        return ((long) (x + 0x800000) << 40) | ((long) (y + 0x800) << 24) | (z + 0x800000);
    }
    
    // 🔧 ГЕТТЕРЫ
    
    public LightPattern1KB[] getPatterns() { return patterns; }
    public int getActivePatternCount() { return activePatternCount; }
    public ByteBuffer getGpuBuffer() { return gpuBuffer; }
    public boolean isGpuBufferDirty() { return gpuBufferDirty; }
    
    /**
     * 🛑 Остановка менеджера
     */
    public void shutdown() {
        asyncExecutor.shutdown();
    }
}

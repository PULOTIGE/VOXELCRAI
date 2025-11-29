package net.voxelcrai.mod;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientChunkEvents;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.voxelcrai.pattern.LightPatternManager;
import net.voxelcrai.shader.ShaderPackManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * 🚀 VOXELCRAI MOD - ТОЧКА ВХОДА
 * 
 * Мод для продвинутого воксельного освещения на основе паттернов LightPattern1KB.
 * Использует Spherical Harmonics (SH) для глобального освещения, теней и отражений.
 * 
 * Совместимость: Minecraft 1.21.3+, Iris 1.7+, Sodium 0.6+
 * Целевая производительность: 60+ FPS на AMD Radeon VII (1080p)
 * 
 * @author PULOTIGE
 */
public class VoxelCraiMod implements ClientModInitializer {
    
    public static final String MOD_ID = "voxelcrai";
    public static final Logger LOGGER = LoggerFactory.getLogger(MOD_ID);
    
    // 🎮 КОНФИГУРАЦИЯ
    public static final int DEFAULT_PATTERN_COUNT = 10000;  // 10k паттернов по умолчанию
    public static final int MIN_PATTERN_COUNT = 1000;       // Минимум 1k
    public static final int MAX_PATTERN_COUNT = 10000;      // Максимум 10k
    public static final int DEFAULT_SH_BANDS = 3;           // 3 полосы SH по умолчанию
    
    // 🔧 МЕНЕДЖЕРЫ
    private static LightPatternManager patternManager;
    private static ShaderPackManager shaderPackManager;
    private static VoxelCraiConfig config;
    
    // 📊 СТАТИСТИКА
    private static int frameCount = 0;
    private static long lastFpsTime = 0;
    private static int currentFps = 0;
    
    @Override
    public void onInitializeClient() {
        LOGGER.info("========================================");
        LOGGER.info("🚀 VoxelCrai Mod v1.0.0 - ИНИЦИАЛИЗАЦИЯ");
        LOGGER.info("========================================");
        
        // 📁 Загрузка конфигурации
        config = VoxelCraiConfig.load();
        LOGGER.info("📁 Конфигурация загружена: {} паттернов, {} SH полос", 
            config.getPatternCount(), config.getShBands());
        
        // 🎨 Инициализация менеджера паттернов
        patternManager = new LightPatternManager(config.getPatternCount(), config.getShBands());
        LOGGER.info("🎨 Менеджер паттернов инициализирован");
        
        // 🖼️ Инициализация шейдерпака
        shaderPackManager = new ShaderPackManager();
        shaderPackManager.extractShaderPack();
        LOGGER.info("🖼️ Шейдерпак извлечён");
        
        // 🔄 Регистрация событий чанков
        registerChunkEvents();
        
        // ⏱️ Регистрация тиков для обновления паттернов
        registerTickEvents();
        
        LOGGER.info("========================================");
        LOGGER.info("✅ VoxelCrai Mod успешно загружен!");
        LOGGER.info("📊 Готово {} паттернов LightPattern1KB", config.getPatternCount());
        LOGGER.info("========================================");
    }
    
    /**
     * 🔄 Регистрация событий загрузки/выгрузки чанков
     */
    private void registerChunkEvents() {
        // 📦 При загрузке чанка - генерируем паттерны для него
        ClientChunkEvents.CHUNK_LOAD.register((world, chunk) -> {
            int chunkX = chunk.getPos().x;
            int chunkZ = chunk.getPos().z;
            
            // 🎨 Генерируем паттерны для блоков чанка
            patternManager.generatePatternsForChunk(chunkX, chunkZ, chunk);
            
            LOGGER.debug("📦 Чанк [{}, {}] загружен, паттерны обновлены", chunkX, chunkZ);
        });
        
        // 🗑️ При выгрузке чанка - освобождаем память
        ClientChunkEvents.CHUNK_UNLOAD.register((world, chunk) -> {
            int chunkX = chunk.getPos().x;
            int chunkZ = chunk.getPos().z;
            
            patternManager.unloadChunkPatterns(chunkX, chunkZ);
            
            LOGGER.debug("🗑️ Чанк [{}, {}] выгружен", chunkX, chunkZ);
        });
    }
    
    /**
     * ⏱️ Регистрация тиков клиента
     */
    private void registerTickEvents() {
        ClientTickEvents.END_CLIENT_TICK.register(client -> {
            frameCount++;
            
            long currentTime = System.currentTimeMillis();
            if (currentTime - lastFpsTime >= 1000) {
                currentFps = frameCount;
                frameCount = 0;
                lastFpsTime = currentTime;
                
                // 📊 Логируем FPS каждую секунду (только в debug)
                if (currentFps < 60) {
                    LOGGER.debug("⚠️ FPS: {} (ниже целевого 60)", currentFps);
                }
            }
            
            // 🔄 Обновляем динамические паттерны (каждый тик)
            if (patternManager != null) {
                patternManager.updateDynamicPatterns();
            }
        });
    }
    
    // 🔧 ГЕТТЕРЫ
    
    public static LightPatternManager getPatternManager() {
        return patternManager;
    }
    
    public static ShaderPackManager getShaderPackManager() {
        return shaderPackManager;
    }
    
    public static VoxelCraiConfig getConfig() {
        return config;
    }
    
    public static int getCurrentFps() {
        return currentFps;
    }
}

package net.voxelcrai.mod;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import net.fabricmc.loader.api.FabricLoader;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * 📁 КОНФИГУРАЦИЯ VOXELCRAI
 * 
 * Настройки мода: количество паттернов, полосы SH, параметры производительности.
 */
public class VoxelCraiConfig {
    
    private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();
    private static final String CONFIG_FILE = "voxelcrai.json";
    
    // 🎮 НАСТРОЙКИ ПАТТЕРНОВ
    private int patternCount = VoxelCraiMod.DEFAULT_PATTERN_COUNT;
    private int shBands = VoxelCraiMod.DEFAULT_SH_BANDS;
    
    // 🖼️ НАСТРОЙКИ РЕНДЕРИНГА
    private boolean enableGI = true;           // Глобальное освещение
    private boolean enableShadows = true;      // Тени
    private boolean enableReflections = true;  // Отражения
    private float giIntensity = 1.0f;          // Интенсивность GI
    private float shadowSoftness = 0.5f;       // Мягкость теней
    private float reflectionIntensity = 0.8f;  // Интенсивность отражений
    
    // 📊 НАСТРОЙКИ ПРОИЗВОДИТЕЛЬНОСТИ
    private boolean asyncPatternGeneration = true;  // Асинхронная генерация
    private int maxPatternsPerTick = 100;           // Макс. паттернов за тик
    private boolean useSSBO = true;                 // Использовать SSBO
    
    /**
     * 📂 Загрузка конфигурации из файла
     */
    public static VoxelCraiConfig load() {
        Path configPath = FabricLoader.getInstance().getConfigDir().resolve(CONFIG_FILE);
        
        if (Files.exists(configPath)) {
            try {
                String json = Files.readString(configPath);
                VoxelCraiConfig config = GSON.fromJson(json, VoxelCraiConfig.class);
                config.validate();
                return config;
            } catch (IOException e) {
                VoxelCraiMod.LOGGER.warn("⚠️ Ошибка загрузки конфига, используем значения по умолчанию: {}", e.getMessage());
            }
        }
        
        // 📝 Создаём конфиг по умолчанию
        VoxelCraiConfig config = new VoxelCraiConfig();
        config.save();
        return config;
    }
    
    /**
     * 💾 Сохранение конфигурации в файл
     */
    public void save() {
        Path configPath = FabricLoader.getInstance().getConfigDir().resolve(CONFIG_FILE);
        
        try {
            Files.writeString(configPath, GSON.toJson(this));
            VoxelCraiMod.LOGGER.info("💾 Конфигурация сохранена: {}", configPath);
        } catch (IOException e) {
            VoxelCraiMod.LOGGER.error("❌ Ошибка сохранения конфига: {}", e.getMessage());
        }
    }
    
    /**
     * ✅ Валидация значений конфигурации
     */
    private void validate() {
        patternCount = Math.clamp(patternCount, VoxelCraiMod.MIN_PATTERN_COUNT, VoxelCraiMod.MAX_PATTERN_COUNT);
        shBands = Math.clamp(shBands, 2, 5);
        giIntensity = Math.clamp(giIntensity, 0.0f, 2.0f);
        shadowSoftness = Math.clamp(shadowSoftness, 0.0f, 1.0f);
        reflectionIntensity = Math.clamp(reflectionIntensity, 0.0f, 1.0f);
        maxPatternsPerTick = Math.clamp(maxPatternsPerTick, 10, 1000);
    }
    
    // 🔧 ГЕТТЕРЫ И СЕТТЕРЫ
    
    public int getPatternCount() { return patternCount; }
    public void setPatternCount(int count) { 
        this.patternCount = Math.clamp(count, VoxelCraiMod.MIN_PATTERN_COUNT, VoxelCraiMod.MAX_PATTERN_COUNT);
    }
    
    public int getShBands() { return shBands; }
    public void setShBands(int bands) { 
        this.shBands = Math.clamp(bands, 2, 5);
    }
    
    public boolean isEnableGI() { return enableGI; }
    public void setEnableGI(boolean enable) { this.enableGI = enable; }
    
    public boolean isEnableShadows() { return enableShadows; }
    public void setEnableShadows(boolean enable) { this.enableShadows = enable; }
    
    public boolean isEnableReflections() { return enableReflections; }
    public void setEnableReflections(boolean enable) { this.enableReflections = enable; }
    
    public float getGiIntensity() { return giIntensity; }
    public void setGiIntensity(float intensity) { this.giIntensity = Math.clamp(intensity, 0.0f, 2.0f); }
    
    public float getShadowSoftness() { return shadowSoftness; }
    public void setShadowSoftness(float softness) { this.shadowSoftness = Math.clamp(softness, 0.0f, 1.0f); }
    
    public float getReflectionIntensity() { return reflectionIntensity; }
    public void setReflectionIntensity(float intensity) { this.reflectionIntensity = Math.clamp(intensity, 0.0f, 1.0f); }
    
    public boolean isAsyncPatternGeneration() { return asyncPatternGeneration; }
    public int getMaxPatternsPerTick() { return maxPatternsPerTick; }
    public boolean isUseSSBO() { return useSSBO; }
}

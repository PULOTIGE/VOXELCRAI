package net.voxelcrai.config;

import net.minecraft.client.gui.DrawContext;
import net.minecraft.client.gui.screen.Screen;
import net.minecraft.client.gui.widget.ButtonWidget;
import net.minecraft.client.gui.widget.SliderWidget;
import net.minecraft.text.Text;
import net.voxelcrai.mod.VoxelCraiMod;

/**
 * 🎛️ VoxelCraiConfigScreen - Экран настроек мода
 * 
 * Слайдеры для настройки:
 * - Количество паттернов (1k-10k)
 * - SH bands (3-5)
 * - Интенсивность GI/теней/отражений
 */
public class VoxelCraiConfigScreen extends Screen {
    
    private final Screen parent;
    private VoxelCraiConfig config;
    
    // 🎚️ Слайдеры
    private PatternCountSlider patternCountSlider;
    private ShBandsSlider shBandsSlider;
    private IntensitySlider giIntensitySlider;
    private IntensitySlider shadowIntensitySlider;
    private IntensitySlider reflectionIntensitySlider;
    
    public VoxelCraiConfigScreen(Screen parent) {
        super(Text.literal("🔮 VoxelCraiMod - Настройки"));
        this.parent = parent;
        this.config = VoxelCraiMod.getInstance().getConfig();
    }
    
    @Override
    protected void init() {
        int centerX = this.width / 2;
        int startY = 40;
        int spacing = 25;
        
        // 🔢 Количество паттернов
        this.patternCountSlider = new PatternCountSlider(
            centerX - 100, startY, 200, 20,
            config.getPatternCount()
        );
        this.addDrawableChild(patternCountSlider);
        
        // 🔮 SH Bands
        this.shBandsSlider = new ShBandsSlider(
            centerX - 100, startY + spacing, 200, 20,
            config.getShBands()
        );
        this.addDrawableChild(shBandsSlider);
        
        // 💡 GI Intensity
        this.giIntensitySlider = new IntensitySlider(
            centerX - 100, startY + spacing * 2, 200, 20,
            "GI", config.getGiIntensity()
        );
        this.addDrawableChild(giIntensitySlider);
        
        // 🌑 Shadow Intensity
        this.shadowIntensitySlider = new IntensitySlider(
            centerX - 100, startY + spacing * 3, 200, 20,
            "Тени", config.getShadowIntensity()
        );
        this.addDrawableChild(shadowIntensitySlider);
        
        // ✨ Reflection Intensity
        this.reflectionIntensitySlider = new IntensitySlider(
            centerX - 100, startY + spacing * 4, 200, 20,
            "Отражения", config.getReflectionIntensity()
        );
        this.addDrawableChild(reflectionIntensitySlider);
        
        // ✅ Кнопка сохранения
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("💾 Сохранить"),
            button -> saveAndClose()
        ).dimensions(centerX - 100, startY + spacing * 6, 95, 20).build());
        
        // ❌ Кнопка отмены
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("❌ Отмена"),
            button -> close()
        ).dimensions(centerX + 5, startY + spacing * 6, 95, 20).build());
        
        // 🔄 Кнопка сброса
        this.addDrawableChild(ButtonWidget.builder(
            Text.literal("🔄 Сброс"),
            button -> resetToDefaults()
        ).dimensions(centerX - 50, startY + spacing * 7 + 10, 100, 20).build());
    }
    
    @Override
    public void render(DrawContext context, int mouseX, int mouseY, float delta) {
        this.renderBackground(context, mouseX, mouseY, delta);
        
        // 📝 Заголовок
        context.drawCenteredTextWithShadow(
            this.textRenderer,
            this.title,
            this.width / 2, 15,
            0xFFFFFF
        );
        
        // 📊 Информация о памяти
        int patternCount = patternCountSlider.getValue();
        int memoryKB = patternCount * 1024 / 1024;  // 1KB на паттерн
        
        context.drawCenteredTextWithShadow(
            this.textRenderer,
            Text.literal(String.format("📊 Память: %d KB (%.1f MB)", memoryKB, memoryKB / 1024.0f)),
            this.width / 2, this.height - 30,
            0xAAAAAA
        );
        
        super.render(context, mouseX, mouseY, delta);
    }
    
    /**
     * 💾 Сохранение и закрытие
     */
    private void saveAndClose() {
        config.setPatternCount(patternCountSlider.getValue());
        config.setShBands(shBandsSlider.getValue());
        config.setGiIntensity(giIntensitySlider.getValue());
        config.setShadowIntensity(shadowIntensitySlider.getValue());
        config.setReflectionIntensity(reflectionIntensitySlider.getValue());
        
        config.save();
        
        VoxelCraiMod.LOGGER.info("💾 Настройки сохранены");
        close();
    }
    
    /**
     * 🔄 Сброс к значениям по умолчанию
     */
    private void resetToDefaults() {
        patternCountSlider.setValue(10_000);
        shBandsSlider.setValue(4);
        giIntensitySlider.setValue(1.0f);
        shadowIntensitySlider.setValue(1.0f);
        reflectionIntensitySlider.setValue(0.8f);
    }
    
    @Override
    public void close() {
        if (this.client != null) {
            this.client.setScreen(parent);
        }
    }
    
    // ========== 🎚️ Кастомные слайдеры ==========
    
    /**
     * 🔢 Слайдер количества паттернов
     */
    private static class PatternCountSlider extends SliderWidget {
        private int patternValue;
        
        public PatternCountSlider(int x, int y, int width, int height, int initialValue) {
            super(x, y, width, height, Text.empty(), valueToSlider(initialValue));
            this.patternValue = initialValue;
            updateMessage();
        }
        
        @Override
        protected void updateMessage() {
            setMessage(Text.literal(String.format("🔢 Паттерны: %,d", patternValue)));
        }
        
        @Override
        protected void applyValue() {
            this.patternValue = sliderToValue(this.value);
            updateMessage();
        }
        
        public int getValue() { return patternValue; }
        
        public void setValue(int value) {
            this.patternValue = Math.max(1000, Math.min(100000, value));
            this.value = valueToSlider(this.patternValue);
            updateMessage();
        }
        
        private static double valueToSlider(int value) {
            // Логарифмическая шкала: 1k -> 100k
            return (Math.log10(value) - 3) / 2.0;  // log10(1000)=3, log10(100000)=5
        }
        
        private static int sliderToValue(double slider) {
            return (int) Math.pow(10, slider * 2.0 + 3.0);
        }
    }
    
    /**
     * 🔮 Слайдер SH bands
     */
    private static class ShBandsSlider extends SliderWidget {
        private int bandsValue;
        
        public ShBandsSlider(int x, int y, int width, int height, int initialValue) {
            super(x, y, width, height, Text.empty(), (initialValue - 3) / 2.0);
            this.bandsValue = initialValue;
            updateMessage();
        }
        
        @Override
        protected void updateMessage() {
            int coeffCount = bandsValue * bandsValue;  // 3 bands = 9, 4 bands = 16, 5 bands = 25
            setMessage(Text.literal(String.format("🔮 SH Bands: %d (%d коэфф.)", bandsValue, coeffCount)));
        }
        
        @Override
        protected void applyValue() {
            this.bandsValue = (int) Math.round(this.value * 2.0 + 3.0);
            updateMessage();
        }
        
        public int getValue() { return bandsValue; }
        
        public void setValue(int value) {
            this.bandsValue = Math.max(3, Math.min(5, value));
            this.value = (bandsValue - 3) / 2.0;
            updateMessage();
        }
    }
    
    /**
     * 📊 Слайдер интенсивности
     */
    private static class IntensitySlider extends SliderWidget {
        private final String name;
        private float intensityValue;
        
        public IntensitySlider(int x, int y, int width, int height, String name, float initialValue) {
            super(x, y, width, height, Text.empty(), initialValue / 2.0);
            this.name = name;
            this.intensityValue = initialValue;
            updateMessage();
        }
        
        @Override
        protected void updateMessage() {
            setMessage(Text.literal(String.format("💡 %s: %.0f%%", name, intensityValue * 100)));
        }
        
        @Override
        protected void applyValue() {
            this.intensityValue = (float) (this.value * 2.0);
            updateMessage();
        }
        
        public float getValue() { return intensityValue; }
        
        public void setValue(float value) {
            this.intensityValue = Math.max(0, Math.min(2, value));
            this.value = intensityValue / 2.0;
            updateMessage();
        }
    }
}

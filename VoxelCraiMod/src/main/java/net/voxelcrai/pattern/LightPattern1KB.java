package net.voxelcrai.pattern;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;

/**
 * 🎨 LIGHTPATTERN1KB - ПАТТЕРН ОСВЕЩЕНИЯ 1024 БАЙТА
 * 
 * Структура данных для GPU-ускоренного освещения с SH коэффициентами.
 * Выровнена до 1024 байт для оптимальной работы с GPU буферами (степень двойки).
 * 
 * Портировано из Rust прототипа с улучшениями для GLSL совместимости.
 * 
 * Layout (1024 bytes total):
 * - id:                 8 bytes  (u64)
 * - _pad0:              8 bytes  (alignment)
 * - direct_light:       6 bytes  (RGB fp16)
 * - indirect_light:     6 bytes  (RGB fp16)
 * - sh_coeffs:          9 bytes  (3 bands SH, i8 each)
 * - material_roughness: 1 byte   (u8)
 * - material_metallic:  1 byte   (u8)
 * - flags:              2 bytes  (u16)
 * - sh_coeffs_band4:    7 bytes  (4th band SH if enabled)
 * - sh_coeffs_band5:    9 bytes  (5th band SH if enabled)
 * - reserved:           16 bytes (future use)
 * - _padding:           951 bytes (to 1024)
 * 
 * @author PULOTIGE
 */
public class LightPattern1KB {
    
    public static final int SIZE_BYTES = 1024;  // 🔢 Размер в байтах (степень двойки!)
    
    // 🆔 ИДЕНТИФИКАЦИЯ
    private long id;                    // Уникальный ID паттерна (8B)
    
    // 💡 ОСВЕЩЕНИЕ (fp16 RGB)
    private short directLightR;         // Прямой свет R (fp16)
    private short directLightG;         // Прямой свет G (fp16)
    private short directLightB;         // Прямой свет B (fp16)
    private short indirectLightR;       // Непрямой свет R (fp16)
    private short indirectLightG;       // Непрямой свет G (fp16)
    private short indirectLightB;       // Непрямой свет B (fp16)
    
    // 🌐 SPHERICAL HARMONICS (SH) КОЭФФИЦИЕНТЫ
    // Band 0: 1 коэфф, Band 1: 3 коэфф, Band 2: 5 коэфф = 9 для 3 bands
    private byte[] shCoeffs = new byte[9];       // 3 полосы SH (i8 каждый)
    private byte[] shCoeffsBand4 = new byte[7];  // 4-я полоса (опционально)
    private byte[] shCoeffsBand5 = new byte[9];  // 5-я полоса (опционально)
    
    // 🎨 МАТЕРИАЛЫ
    private byte materialRoughness;     // Шероховатость (0-255 -> 0.0-1.0)
    private byte materialMetallic;      // Металличность (0-255 -> 0.0-1.0)
    
    // 🏴 ФЛАГИ
    private short flags;
    
    // 📍 ПОЗИЦИЯ (для chunk-local адресации)
    private int chunkX;
    private int chunkZ;
    private int localX;
    private int localY;
    private int localZ;
    
    // 🏴 ФЛАГИ КОНСТАНТ
    public static final short FLAG_EMISSIVE = 0x0001;       // Источник света
    public static final short FLAG_TRANSPARENT = 0x0002;    // Прозрачный
    public static final short FLAG_WATER = 0x0004;          // Вода
    public static final short FLAG_FOLIAGE = 0x0008;        // Листва
    public static final short FLAG_DYNAMIC = 0x0010;        // Динамический
    public static final short FLAG_SHADOW_CASTER = 0x0020;  // Отбрасывает тень
    public static final short FLAG_SHADOW_RECV = 0x0040;    // Принимает тень
    public static final short FLAG_REFLECTIVE = 0x0080;     // Отражающий
    
    /**
     * 🏗️ Конструктор по умолчанию
     */
    public LightPattern1KB() {
        this.id = 0;
        this.flags = FLAG_SHADOW_RECV;  // По умолчанию принимает тени
        this.materialRoughness = (byte) 128;  // 0.5 roughness
        this.materialMetallic = 0;
    }
    
    /**
     * 🏗️ Конструктор с ID
     */
    public LightPattern1KB(long id) {
        this();
        this.id = id;
    }
    
    /**
     * 💾 Сериализация в ByteBuffer для GPU
     */
    public void writeToBuffer(ByteBuffer buffer) {
        buffer.order(ByteOrder.LITTLE_ENDIAN);
        
        // ID + padding (16 bytes)
        buffer.putLong(id);
        buffer.putLong(0);  // _pad0
        
        // Direct light RGB fp16 (6 bytes)
        buffer.putShort(directLightR);
        buffer.putShort(directLightG);
        buffer.putShort(directLightB);
        
        // Indirect light RGB fp16 (6 bytes)
        buffer.putShort(indirectLightR);
        buffer.putShort(indirectLightG);
        buffer.putShort(indirectLightB);
        
        // SH coefficients band 0-2 (9 bytes)
        buffer.put(shCoeffs);
        
        // Material properties (2 bytes)
        buffer.put(materialRoughness);
        buffer.put(materialMetallic);
        
        // Flags (2 bytes)
        buffer.putShort(flags);
        
        // SH band 4 (7 bytes)
        buffer.put(shCoeffsBand4);
        
        // SH band 5 (9 bytes)
        buffer.put(shCoeffsBand5);
        
        // Position data (20 bytes)
        buffer.putInt(chunkX);
        buffer.putInt(chunkZ);
        buffer.putInt(localX);
        buffer.putInt(localY);
        buffer.putInt(localZ);
        
        // Padding to 1024 bytes
        int written = 16 + 6 + 6 + 9 + 2 + 2 + 7 + 9 + 20;  // 77 bytes
        int paddingSize = SIZE_BYTES - written;
        for (int i = 0; i < paddingSize; i++) {
            buffer.put((byte) 0);
        }
    }
    
    /**
     * 📖 Десериализация из ByteBuffer
     */
    public void readFromBuffer(ByteBuffer buffer) {
        buffer.order(ByteOrder.LITTLE_ENDIAN);
        
        id = buffer.getLong();
        buffer.getLong();  // _pad0
        
        directLightR = buffer.getShort();
        directLightG = buffer.getShort();
        directLightB = buffer.getShort();
        
        indirectLightR = buffer.getShort();
        indirectLightG = buffer.getShort();
        indirectLightB = buffer.getShort();
        
        buffer.get(shCoeffs);
        
        materialRoughness = buffer.get();
        materialMetallic = buffer.get();
        
        flags = buffer.getShort();
        
        buffer.get(shCoeffsBand4);
        buffer.get(shCoeffsBand5);
        
        chunkX = buffer.getInt();
        chunkZ = buffer.getInt();
        localX = buffer.getInt();
        localY = buffer.getInt();
        localZ = buffer.getInt();
        
        // Skip padding
        buffer.position(buffer.position() + (SIZE_BYTES - 77));
    }
    
    // 🔧 УТИЛИТЫ ДЛЯ FP16
    
    /**
     * 🔢 Конвертация float -> fp16 (half)
     */
    public static short floatToHalf(float value) {
        int bits = Float.floatToIntBits(value);
        int sign = (bits >> 16) & 0x8000;
        int val = (bits & 0x7FFFFFFF) + 0x1000;
        
        if (val >= 0x47800000) {
            if ((bits & 0x7FFFFFFF) >= 0x47800000) {
                if (val < 0x7F800000) return (short) (sign | 0x7C00);
                return (short) (sign | 0x7C00 | ((bits & 0x007FFFFF) >> 13));
            }
            return (short) (sign | 0x7BFF);
        }
        
        if (val >= 0x38800000) {
            return (short) (sign | ((val - 0x38000000) >> 13));
        }
        
        if (val < 0x33000000) {
            return (short) sign;
        }
        
        val = (bits & 0x7FFFFFFF) >> 23;
        return (short) (sign | (((bits & 0x7FFFFF) | 0x800000) + (0x800000 >> (val - 102))) >> (126 - val));
    }
    
    /**
     * 🔢 Конвертация fp16 (half) -> float
     */
    public static float halfToFloat(short half) {
        int h = half & 0xFFFF;
        int sign = (h >> 15) & 1;
        int exp = (h >> 10) & 0x1F;
        int mant = h & 0x3FF;
        
        if (exp == 0) {
            if (mant == 0) {
                return sign == 0 ? 0.0f : -0.0f;
            }
            // Denormalized
            float f = mant / 1024.0f;
            return sign == 0 ? f * (float) Math.pow(2, -14) : -f * (float) Math.pow(2, -14);
        }
        
        if (exp == 31) {
            if (mant == 0) {
                return sign == 0 ? Float.POSITIVE_INFINITY : Float.NEGATIVE_INFINITY;
            }
            return Float.NaN;
        }
        
        float f = 1.0f + mant / 1024.0f;
        f *= (float) Math.pow(2, exp - 15);
        return sign == 0 ? f : -f;
    }
    
    // 🔧 СЕТТЕРЫ ДЛЯ ОСВЕЩЕНИЯ (принимают float, конвертируют в fp16)
    
    public void setDirectLight(float r, float g, float b) {
        this.directLightR = floatToHalf(r);
        this.directLightG = floatToHalf(g);
        this.directLightB = floatToHalf(b);
    }
    
    public void setIndirectLight(float r, float g, float b) {
        this.indirectLightR = floatToHalf(r);
        this.indirectLightG = floatToHalf(g);
        this.indirectLightB = floatToHalf(b);
    }
    
    // 🔧 ГЕТТЕРЫ ДЛЯ ОСВЕЩЕНИЯ (возвращают float)
    
    public float getDirectLightR() { return halfToFloat(directLightR); }
    public float getDirectLightG() { return halfToFloat(directLightG); }
    public float getDirectLightB() { return halfToFloat(directLightB); }
    
    public float getIndirectLightR() { return halfToFloat(indirectLightR); }
    public float getIndirectLightG() { return halfToFloat(indirectLightG); }
    public float getIndirectLightB() { return halfToFloat(indirectLightB); }
    
    // 🔧 SH КОЭФФИЦИЕНТЫ
    
    /**
     * 🌐 Установка SH коэффициентов (3 bands = 9 коэффициентов)
     * Нормализованы в диапазон [-128, 127] из [-1.0, 1.0]
     */
    public void setShCoeffs(float[] coeffs) {
        for (int i = 0; i < Math.min(9, coeffs.length); i++) {
            shCoeffs[i] = (byte) Math.clamp((int)(coeffs[i] * 127.0f), -128, 127);
        }
    }
    
    /**
     * 🌐 Получение SH коэффициентов как float[]
     */
    public float[] getShCoeffsFloat() {
        float[] result = new float[9];
        for (int i = 0; i < 9; i++) {
            result[i] = shCoeffs[i] / 127.0f;
        }
        return result;
    }
    
    public byte[] getShCoeffs() { return shCoeffs; }
    
    // 🔧 МАТЕРИАЛЫ
    
    public void setRoughness(float roughness) {
        this.materialRoughness = (byte) Math.clamp((int)(roughness * 255.0f), 0, 255);
    }
    
    public float getRoughness() {
        return (materialRoughness & 0xFF) / 255.0f;
    }
    
    public void setMetallic(float metallic) {
        this.materialMetallic = (byte) Math.clamp((int)(metallic * 255.0f), 0, 255);
    }
    
    public float getMetallic() {
        return (materialMetallic & 0xFF) / 255.0f;
    }
    
    // 🔧 ФЛАГИ
    
    public void setFlag(short flag, boolean enabled) {
        if (enabled) {
            flags |= flag;
        } else {
            flags &= ~flag;
        }
    }
    
    public boolean hasFlag(short flag) {
        return (flags & flag) != 0;
    }
    
    // 🔧 ПОЗИЦИЯ
    
    public void setPosition(int chunkX, int chunkZ, int localX, int localY, int localZ) {
        this.chunkX = chunkX;
        this.chunkZ = chunkZ;
        this.localX = localX;
        this.localY = localY;
        this.localZ = localZ;
    }
    
    // 🔧 ID
    
    public long getId() { return id; }
    public void setId(long id) { this.id = id; }
    
    public short getFlags() { return flags; }
}

package net.voxelcrai.shader;

import net.fabricmc.loader.api.FabricLoader;
import net.voxelcrai.mod.VoxelCraiMod;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;

/**
 * 🖼️ МЕНЕДЖЕР ШЕЙДЕРПАКА
 * 
 * Извлекает и управляет Iris шейдерпаком VoxelCrai.
 * 
 * @author PULOTIGE
 */
public class ShaderPackManager {
    
    private static final String SHADERPACK_NAME = "VoxelCrai-Shaders";
    private Path shaderpackPath;
    
    /**
     * 📦 Извлечение шейдерпака в папку shaderpacks
     */
    public void extractShaderPack() {
        Path gameDir = FabricLoader.getInstance().getGameDir();
        Path shaderpacksDir = gameDir.resolve("shaderpacks");
        shaderpackPath = shaderpacksDir.resolve(SHADERPACK_NAME);
        
        try {
            // 📁 Создаём директории
            Files.createDirectories(shaderpackPath.resolve("shaders"));
            Files.createDirectories(shaderpackPath.resolve("shaders/lib"));
            Files.createDirectories(shaderpackPath.resolve("shaders/program"));
            Files.createDirectories(shaderpackPath.resolve("shaders/program/composite"));
            Files.createDirectories(shaderpackPath.resolve("shaders/program/gbuffers"));
            
            // 📝 Записываем файлы шейдеров
            writeShaderFile("shaders.properties", generateShadersProperties());
            writeShaderFile("shaders/lib/common.glsl", generateCommonGlsl());
            writeShaderFile("shaders/lib/sh.glsl", generateShGlsl());
            writeShaderFile("shaders/lib/lighting.glsl", generateLightingGlsl());
            writeShaderFile("shaders/lib/patterns.glsl", generatePatternsGlsl());
            writeShaderFile("shaders/program/gbuffers/gbuffers_terrain.vsh", generateGbuffersTerrainVsh());
            writeShaderFile("shaders/program/gbuffers/gbuffers_terrain.fsh", generateGbuffersTerrainFsh());
            writeShaderFile("shaders/program/composite/composite.vsh", generateCompositeVsh());
            writeShaderFile("shaders/program/composite/composite.fsh", generateCompositeFsh());
            writeShaderFile("shaders/program/composite/final.vsh", generateFinalVsh());
            writeShaderFile("shaders/program/composite/final.fsh", generateFinalFsh());
            
            VoxelCraiMod.LOGGER.info("🖼️ Шейдерпак извлечён: {}", shaderpackPath);
            
        } catch (IOException e) {
            VoxelCraiMod.LOGGER.error("❌ Ошибка извлечения шейдерпака: {}", e.getMessage());
        }
    }
    
    private void writeShaderFile(String path, String content) throws IOException {
        Path file = shaderpackPath.resolve(path);
        Files.writeString(file, content);
    }
    
    // ============================================
    // 🖼️ ГЕНЕРАЦИЯ ШЕЙДЕРОВ
    // ============================================
    
    private String generateShadersProperties() {
        return """
            # 🚀 VOXELCRAI SHADERS - КОНФИГУРАЦИЯ
            # Продвинутое воксельное освещение на основе LightPattern1KB
            
            shaders.world0=
            
            # 📊 Размер буферов
            const int shadowMapResolution = 2048;
            const float shadowDistance = 128.0;
            const float shadowDistanceRenderMul = 1.0;
            
            # 🎨 Цветовые буферы
            const int colortex0Format = RGBA16F;
            const int colortex1Format = RGBA16F;
            const int colortex2Format = RGBA16F;
            const int colortex3Format = RGBA32F;
            
            # 🔧 Параметры
            sliders = PATTERN_COUNT SH_BANDS GI_INTENSITY SHADOW_SOFTNESS
            
            # 📊 Количество паттернов
            const int PATTERN_COUNT = 10000;   // [1000 2000 5000 10000]
            
            # 🌐 Полосы SH
            const int SH_BANDS = 3;   // [2 3 4 5]
            
            # 💡 Интенсивность GI
            const float GI_INTENSITY = 1.0;   // [0.0 0.25 0.5 0.75 1.0 1.25 1.5 2.0]
            
            # 🌑 Мягкость теней
            const float SHADOW_SOFTNESS = 0.5;   // [0.0 0.25 0.5 0.75 1.0]
            
            # ✨ Интенсивность отражений
            const float REFLECTION_INTENSITY = 0.8;   // [0.0 0.25 0.5 0.75 1.0]
            """;
    }
    
    private String generateCommonGlsl() {
        return """
            // 🚀 VOXELCRAI - ОБЩИЕ ОПРЕДЕЛЕНИЯ
            // common.glsl
            
            #ifndef COMMON_GLSL
            #define COMMON_GLSL
            
            // 📊 КОНСТАНТЫ
            #define PI 3.14159265359
            #define TAU 6.28318530718
            #define EPSILON 0.0001
            
            // 🎨 LIGHTPATTERN1KB СТРУКТУРА (1024 байта)
            struct LightPattern1KB {
                uvec2 id;           // 8 bytes (u64)
                uvec2 _pad0;        // 8 bytes
                uvec3 directLight;  // 12 bytes (RGB fp16 packed as uint)
                uvec3 indirectLight;// 12 bytes (RGB fp16 packed as uint)
                ivec4 shCoeffs0;    // 16 bytes (SH coeffs 0-3)
                ivec4 shCoeffs1;    // 16 bytes (SH coeffs 4-7)
                int shCoeff8;       // 4 bytes (SH coeff 8)
                uint material;      // 4 bytes (roughness + metallic)
                uint flags;         // 4 bytes
                // Остальное - padding до 1024
            };
            
            // 🏴 ФЛАГИ
            #define FLAG_EMISSIVE       0x0001u
            #define FLAG_TRANSPARENT    0x0002u
            #define FLAG_WATER          0x0004u
            #define FLAG_FOLIAGE        0x0008u
            #define FLAG_DYNAMIC        0x0010u
            #define FLAG_SHADOW_CASTER  0x0020u
            #define FLAG_SHADOW_RECV    0x0040u
            #define FLAG_REFLECTIVE     0x0080u
            
            // 🔧 УТИЛИТЫ
            
            // Конвертация fp16 (half) -> float
            float halfToFloat(uint h) {
                uint sign = (h >> 15u) & 1u;
                uint exp = (h >> 10u) & 0x1Fu;
                uint mant = h & 0x3FFu;
                
                if (exp == 0u) {
                    if (mant == 0u) return sign == 0u ? 0.0 : -0.0;
                    float f = float(mant) / 1024.0;
                    return sign == 0u ? f * pow(2.0, -14.0) : -f * pow(2.0, -14.0);
                }
                if (exp == 31u) {
                    return mant == 0u ? (sign == 0u ? 1e38 : -1e38) : 0.0;
                }
                
                float f = 1.0 + float(mant) / 1024.0;
                f *= pow(2.0, float(exp) - 15.0);
                return sign == 0u ? f : -f;
            }
            
            // Извлечение RGB из packed fp16
            vec3 unpackRGBfp16(uvec3 packed) {
                return vec3(
                    halfToFloat(packed.x & 0xFFFFu),
                    halfToFloat(packed.y & 0xFFFFu),
                    halfToFloat(packed.z & 0xFFFFu)
                );
            }
            
            // Извлечение материала
            vec2 unpackMaterial(uint packed) {
                float roughness = float(packed & 0xFFu) / 255.0;
                float metallic = float((packed >> 8u) & 0xFFu) / 255.0;
                return vec2(roughness, metallic);
            }
            
            // Извлечение SH коэффициентов (i8 -> float)
            float unpackSHCoeff(int packed) {
                return float(packed) / 127.0;
            }
            
            #endif
            """;
    }
    
    private String generateShGlsl() {
        return """
            // 🌐 VOXELCRAI - SPHERICAL HARMONICS
            // sh.glsl
            
            #ifndef SH_GLSL
            #define SH_GLSL
            
            #include "common.glsl"
            
            // 🌐 SH БАЗИСНЫЕ ФУНКЦИИ (3 полосы = 9 коэффициентов)
            //
            // Band 0 (l=0): Y_0^0 = 0.282095
            // Band 1 (l=1): Y_1^-1, Y_1^0, Y_1^1
            // Band 2 (l=2): Y_2^-2, Y_2^-1, Y_2^0, Y_2^1, Y_2^2
            
            // 📊 SH КОНСТАНТЫ
            const float SH_C0 = 0.282095;      // 1/(2*sqrt(pi))
            const float SH_C1 = 0.488603;      // sqrt(3/(4*pi))
            const float SH_C2_0 = 1.092548;    // sqrt(15/(4*pi))
            const float SH_C2_1 = 0.315392;    // sqrt(5/(16*pi))
            const float SH_C2_2 = 0.546274;    // sqrt(15/(16*pi))
            
            /**
             * 🌐 Оценка SH для направления
             * 
             * @param coeffs 9 SH коэффициентов (3 полосы)
             * @param dir Нормализованное направление
             * @return Результирующий цвет/интенсивность
             */
            vec3 shEval(float coeffs[9], vec3 dir) {
                // 📊 Предвычисления
                float x = dir.x;
                float y = dir.y;
                float z = dir.z;
                
                float x2 = x * x;
                float y2 = y * y;
                float z2 = z * z;
                
                // 🌐 Band 0 (DC)
                float result = coeffs[0] * SH_C0;
                
                // 🌐 Band 1 (Linear)
                result += coeffs[1] * SH_C1 * y;
                result += coeffs[2] * SH_C1 * z;
                result += coeffs[3] * SH_C1 * x;
                
                // 🌐 Band 2 (Quadratic)
                result += coeffs[4] * SH_C2_0 * x * y;
                result += coeffs[5] * SH_C2_0 * y * z;
                result += coeffs[6] * SH_C2_1 * (3.0 * z2 - 1.0);
                result += coeffs[7] * SH_C2_0 * x * z;
                result += coeffs[8] * SH_C2_2 * (x2 - y2);
                
                return vec3(max(result, 0.0));
            }
            
            /**
             * 🌐 Оценка SH для цветного освещения (RGB)
             */
            vec3 shEvalRGB(float coeffsR[9], float coeffsG[9], float coeffsB[9], vec3 dir) {
                return vec3(
                    shEval(coeffsR, dir).r,
                    shEval(coeffsG, dir).r,
                    shEval(coeffsB, dir).r
                );
            }
            
            /**
             * 🎯 Оценка SH для теней (отрицательные коэффициенты = окклюзия)
             */
            float shEvalShadow(float coeffs[9], vec3 lightDir) {
                float illumination = shEval(coeffs, lightDir).r;
                // Отрицательные значения означают окклюзию
                return clamp(illumination, 0.0, 1.0);
            }
            
            /**
             * ✨ Оценка SH для спекулярных отражений
             */
            vec3 shEvalSpecular(float coeffs[9], vec3 reflectDir, float roughness) {
                // Более грубые поверхности - более размытые отражения
                // Аппроксимируем через более низкие полосы SH
                float lod = roughness * 2.0;  // 0-2 LOD
                
                // Простая аппроксимация: убираем высокочастотные компоненты для rough surfaces
                float tempCoeffs[9];
                for (int i = 0; i < 9; i++) {
                    float bandWeight = 1.0;
                    if (i >= 1 && i <= 3) bandWeight = mix(1.0, 0.5, lod / 2.0);
                    if (i >= 4) bandWeight = mix(1.0, 0.2, lod / 2.0);
                    tempCoeffs[i] = coeffs[i] * bandWeight;
                }
                
                return shEval(tempCoeffs, reflectDir);
            }
            
            /**
             * 🔄 Поворот SH коэффициентов (для динамического освещения)
             */
            void shRotateY(inout float coeffs[9], float angle) {
                float c = cos(angle);
                float s = sin(angle);
                
                // Band 1 rotation
                float y1m1 = coeffs[1];
                float y1p1 = coeffs[3];
                coeffs[1] = c * y1m1 - s * y1p1;
                coeffs[3] = s * y1m1 + c * y1p1;
                
                // Band 2 rotation (simplified)
                float y2m2 = coeffs[4];
                float y2p2 = coeffs[8];
                float c2 = cos(2.0 * angle);
                float s2 = sin(2.0 * angle);
                coeffs[4] = c2 * y2m2 - s2 * y2p2;
                coeffs[8] = s2 * y2m2 + c2 * y2p2;
            }
            
            #endif
            """;
    }
    
    private String generateLightingGlsl() {
        return """
            // 💡 VOXELCRAI - СИСТЕМА ОСВЕЩЕНИЯ
            // lighting.glsl
            
            #ifndef LIGHTING_GLSL
            #define LIGHTING_GLSL
            
            #include "common.glsl"
            #include "sh.glsl"
            
            // ☀️ ПАРАМЕТРЫ СОЛНЦА
            uniform vec3 sunPosition;
            uniform vec3 moonPosition;
            uniform float sunAngle;
            uniform float rainStrength;
            
            /**
             * 💡 Расчёт освещения на основе LightPattern1KB
             */
            vec3 calculatePatternLighting(
                LightPattern1KB pattern,
                vec3 normal,
                vec3 viewDir,
                vec3 lightDir
            ) {
                // 📊 Извлекаем данные паттерна
                vec3 directLight = unpackRGBfp16(pattern.directLight);
                vec3 indirectLight = unpackRGBfp16(pattern.indirectLight);
                vec2 material = unpackMaterial(pattern.material);
                float roughness = material.x;
                float metallic = material.y;
                
                // 🌐 Извлекаем SH коэффициенты
                float shCoeffs[9];
                shCoeffs[0] = unpackSHCoeff(pattern.shCoeffs0.x);
                shCoeffs[1] = unpackSHCoeff(pattern.shCoeffs0.y);
                shCoeffs[2] = unpackSHCoeff(pattern.shCoeffs0.z);
                shCoeffs[3] = unpackSHCoeff(pattern.shCoeffs0.w);
                shCoeffs[4] = unpackSHCoeff(pattern.shCoeffs1.x);
                shCoeffs[5] = unpackSHCoeff(pattern.shCoeffs1.y);
                shCoeffs[6] = unpackSHCoeff(pattern.shCoeffs1.z);
                shCoeffs[7] = unpackSHCoeff(pattern.shCoeffs1.w);
                shCoeffs[8] = unpackSHCoeff(pattern.shCoeff8);
                
                // ☀️ ПРЯМОЕ ОСВЕЩЕНИЕ
                float NdotL = max(dot(normal, lightDir), 0.0);
                vec3 direct = directLight * NdotL;
                
                // 🌐 ГЛОБАЛЬНОЕ ОСВЕЩЕНИЕ (SH)
                vec3 gi = shEval(shCoeffs, normal) * indirectLight * GI_INTENSITY;
                
                // 🌑 ТЕНИ (SH-based)
                float shadow = shEvalShadow(shCoeffs, lightDir);
                shadow = mix(shadow, 1.0, SHADOW_SOFTNESS);
                direct *= shadow;
                
                // ✨ ОТРАЖЕНИЯ (для металлических/гладких поверхностей)
                vec3 specular = vec3(0.0);
                if ((pattern.flags & FLAG_REFLECTIVE) != 0u || metallic > 0.1) {
                    vec3 reflectDir = reflect(-viewDir, normal);
                    specular = shEvalSpecular(shCoeffs, reflectDir, roughness);
                    specular *= mix(vec3(0.04), directLight, metallic);
                    specular *= REFLECTION_INTENSITY;
                }
                
                // 💡 ЭМИССИЯ
                vec3 emission = vec3(0.0);
                if ((pattern.flags & FLAG_EMISSIVE) != 0u) {
                    emission = directLight * 2.0;
                }
                
                // 🎨 ФИНАЛЬНАЯ КОМБИНАЦИЯ
                vec3 finalColor = direct + gi + specular + emission;
                
                return finalColor;
            }
            
            /**
             * 🌅 Расчёт цвета неба
             */
            vec3 getSkyColor(vec3 dir) {
                float sunHeight = sunPosition.y;
                
                // День
                vec3 dayTop = vec3(0.3, 0.5, 0.9);
                vec3 dayBottom = vec3(0.6, 0.7, 0.9);
                
                // Закат
                vec3 sunsetTop = vec3(0.2, 0.3, 0.5);
                vec3 sunsetBottom = vec3(0.9, 0.5, 0.3);
                
                // Ночь
                vec3 nightTop = vec3(0.02, 0.02, 0.05);
                vec3 nightBottom = vec3(0.05, 0.05, 0.1);
                
                float sunFactor = clamp(sunHeight * 2.0, 0.0, 1.0);
                float sunsetFactor = clamp(1.0 - abs(sunHeight * 4.0), 0.0, 1.0);
                
                vec3 top = mix(nightTop, dayTop, sunFactor);
                vec3 bottom = mix(nightBottom, dayBottom, sunFactor);
                
                top = mix(top, sunsetTop, sunsetFactor);
                bottom = mix(bottom, sunsetBottom, sunsetFactor);
                
                return mix(bottom, top, max(dir.y, 0.0));
            }
            
            #endif
            """;
    }
    
    private String generatePatternsGlsl() {
        return """
            // 🎨 VOXELCRAI - БУФЕР ПАТТЕРНОВ
            // patterns.glsl
            
            #ifndef PATTERNS_GLSL
            #define PATTERNS_GLSL
            
            #include "common.glsl"
            
            // 📦 SSBO с паттернами
            layout(std430, binding = 0) readonly buffer PatternBuffer {
                LightPattern1KB patterns[];
            };
            
            // 📊 Количество активных паттернов
            uniform int patternCount;
            
            /**
             * 🔍 Получение паттерна по индексу
             */
            LightPattern1KB getPattern(int index) {
                if (index < 0 || index >= patternCount) {
                    // Возвращаем пустой паттерн
                    LightPattern1KB empty;
                    empty.id = uvec2(0u);
                    empty.directLight = uvec3(0u);
                    empty.indirectLight = uvec3(0u);
                    empty.shCoeffs0 = ivec4(0);
                    empty.shCoeffs1 = ivec4(0);
                    empty.shCoeff8 = 0;
                    empty.material = 0x8000u;  // 0.5 roughness, 0 metallic
                    empty.flags = FLAG_SHADOW_RECV;
                    return empty;
                }
                return patterns[index];
            }
            
            /**
             * 🔍 Получение паттерна по мировым координатам
             */
            LightPattern1KB getPatternAtPosition(vec3 worldPos) {
                // Простой хеш для индексации
                int hash = int(worldPos.x) * 73856093 ^
                           int(worldPos.y) * 19349663 ^
                           int(worldPos.z) * 83492791;
                int index = abs(hash) % patternCount;
                return getPattern(index);
            }
            
            /**
             * 🔍 Получение паттерна по UV и глубине
             */
            LightPattern1KB getPatternFromUV(vec2 uv, float depth) {
                int x = int(uv.x * 256.0);
                int y = int(uv.y * 256.0);
                int z = int(depth * 256.0);
                int index = (x + y * 256 + z * 65536) % patternCount;
                return getPattern(index);
            }
            
            #endif
            """;
    }
    
    private String generateGbuffersTerrainVsh() {
        return """
            // 🏔️ VOXELCRAI - GBUFFERS TERRAIN VERTEX SHADER
            // gbuffers_terrain.vsh
            
            #version 330 core
            
            // 📥 Входные атрибуты
            in vec3 vaPosition;
            in vec2 vaUV0;
            in vec3 vaNormal;
            in vec4 vaColor;
            in ivec2 vaUV2;  // Lightmap UV
            
            // 📤 Выходные для фрагментного шейдера
            out vec2 texcoord;
            out vec2 lmcoord;
            out vec3 normal;
            out vec4 color;
            out vec3 worldPos;
            out vec3 viewPos;
            
            // 🎯 Uniform матрицы
            uniform mat4 modelViewMatrix;
            uniform mat4 projectionMatrix;
            uniform mat4 gbufferModelViewInverse;
            uniform vec3 cameraPosition;
            
            void main() {
                // 📍 Позиция в view space
                vec4 viewPosition = modelViewMatrix * vec4(vaPosition, 1.0);
                viewPos = viewPosition.xyz;
                
                // 📍 Мировая позиция
                vec4 worldPosition = gbufferModelViewInverse * viewPosition;
                worldPos = worldPosition.xyz + cameraPosition;
                
                // 📐 Нормаль
                normal = mat3(gbufferModelViewInverse) * mat3(modelViewMatrix) * vaNormal;
                
                // 🎨 Текстурные координаты
                texcoord = vaUV0;
                lmcoord = vec2(vaUV2) / 256.0;
                
                // 🎨 Vertex color
                color = vaColor;
                
                // 📍 Финальная позиция
                gl_Position = projectionMatrix * viewPosition;
            }
            """;
    }
    
    private String generateGbuffersTerrainFsh() {
        return """
            // 🏔️ VOXELCRAI - GBUFFERS TERRAIN FRAGMENT SHADER
            // gbuffers_terrain.fsh
            
            #version 330 core
            #extension GL_ARB_shader_storage_buffer_object : enable
            
            #include "lib/common.glsl"
            #include "lib/sh.glsl"
            #include "lib/lighting.glsl"
            #include "lib/patterns.glsl"
            
            // 📥 Входные данные
            in vec2 texcoord;
            in vec2 lmcoord;
            in vec3 normal;
            in vec4 color;
            in vec3 worldPos;
            in vec3 viewPos;
            
            // 📤 Выходные буферы
            layout(location = 0) out vec4 outColor;
            layout(location = 1) out vec4 outNormal;
            layout(location = 2) out vec4 outSpecular;
            
            // 🎨 Текстуры
            uniform sampler2D gtexture;
            uniform sampler2D lightmap;
            
            // 🎯 Uniforms
            uniform vec3 sunPosition;
            uniform vec3 cameraPosition;
            uniform float frameTimeCounter;
            
            void main() {
                // 🎨 Базовый цвет
                vec4 albedo = texture(gtexture, texcoord) * color;
                
                if (albedo.a < 0.1) discard;
                
                // 📐 Нормализованные векторы
                vec3 N = normalize(normal);
                vec3 V = normalize(cameraPosition - worldPos);
                vec3 L = normalize(sunPosition);
                
                // 🎨 Получаем паттерн для этого блока
                LightPattern1KB pattern = getPatternAtPosition(worldPos);
                
                // 💡 Рассчитываем освещение на основе паттерна
                vec3 lighting = calculatePatternLighting(pattern, N, V, L);
                
                // 🗺️ Lightmap
                vec2 lm = lmcoord;
                float blockLight = lm.x;
                float skyLight = lm.y;
                
                // 🎨 Комбинируем
                vec3 finalColor = albedo.rgb * lighting;
                finalColor += albedo.rgb * blockLight * 0.3;  // Torch light
                
                // 📤 Выходные данные
                outColor = vec4(finalColor, albedo.a);
                outNormal = vec4(N * 0.5 + 0.5, 1.0);
                outSpecular = vec4(unpackMaterial(pattern.material), 0.0, 1.0);
            }
            """;
    }
    
    private String generateCompositeVsh() {
        return """
            // 🖼️ VOXELCRAI - COMPOSITE VERTEX SHADER
            // composite.vsh
            
            #version 330 core
            
            out vec2 texcoord;
            
            void main() {
                // Fullscreen quad
                const vec2 positions[4] = vec2[](
                    vec2(-1.0, -1.0),
                    vec2( 1.0, -1.0),
                    vec2(-1.0,  1.0),
                    vec2( 1.0,  1.0)
                );
                
                gl_Position = vec4(positions[gl_VertexID], 0.0, 1.0);
                texcoord = positions[gl_VertexID] * 0.5 + 0.5;
            }
            """;
    }
    
    private String generateCompositeFsh() {
        return """
            // 🖼️ VOXELCRAI - COMPOSITE FRAGMENT SHADER
            // composite.fsh - Постобработка с SH GI
            
            #version 330 core
            
            #include "lib/common.glsl"
            #include "lib/sh.glsl"
            
            in vec2 texcoord;
            
            layout(location = 0) out vec4 fragColor;
            
            uniform sampler2D colortex0;  // Color
            uniform sampler2D colortex1;  // Normal
            uniform sampler2D colortex2;  // Specular
            uniform sampler2D depthtex0;  // Depth
            
            uniform mat4 gbufferProjectionInverse;
            uniform mat4 gbufferModelViewInverse;
            uniform vec3 cameraPosition;
            uniform vec3 sunPosition;
            
            vec3 getWorldPos(vec2 uv, float depth) {
                vec4 clipPos = vec4(uv * 2.0 - 1.0, depth * 2.0 - 1.0, 1.0);
                vec4 viewPos = gbufferProjectionInverse * clipPos;
                viewPos /= viewPos.w;
                vec4 worldPos = gbufferModelViewInverse * viewPos;
                return worldPos.xyz + cameraPosition;
            }
            
            void main() {
                vec4 color = texture(colortex0, texcoord);
                vec3 normal = texture(colortex1, texcoord).rgb * 2.0 - 1.0;
                vec2 specular = texture(colortex2, texcoord).rg;
                float depth = texture(depthtex0, texcoord).r;
                
                if (depth >= 1.0) {
                    // Sky
                    fragColor = color;
                    return;
                }
                
                vec3 worldPos = getWorldPos(texcoord, depth);
                
                // 🌐 Дополнительный GI проход с SH
                // (основной расчёт уже в gbuffers)
                
                // ✨ Screen-space reflections approximation
                float roughness = specular.r;
                float metallic = specular.g;
                
                if (metallic > 0.1 || roughness < 0.3) {
                    vec3 V = normalize(cameraPosition - worldPos);
                    vec3 R = reflect(-V, normal);
                    
                    // Простая SSR аппроксимация через смещение UV
                    vec2 reflectUV = texcoord + R.xy * 0.1 * (1.0 - roughness);
                    reflectUV = clamp(reflectUV, 0.0, 1.0);
                    
                    vec3 reflectColor = texture(colortex0, reflectUV).rgb;
                    float fresnel = pow(1.0 - max(dot(V, normal), 0.0), 5.0);
                    fresnel = mix(0.04, 1.0, fresnel);
                    
                    color.rgb = mix(color.rgb, reflectColor, fresnel * (1.0 - roughness) * REFLECTION_INTENSITY);
                }
                
                fragColor = color;
            }
            """;
    }
    
    private String generateFinalVsh() {
        return """
            // 🎬 VOXELCRAI - FINAL VERTEX SHADER
            // final.vsh
            
            #version 330 core
            
            out vec2 texcoord;
            
            void main() {
                const vec2 positions[4] = vec2[](
                    vec2(-1.0, -1.0),
                    vec2( 1.0, -1.0),
                    vec2(-1.0,  1.0),
                    vec2( 1.0,  1.0)
                );
                
                gl_Position = vec4(positions[gl_VertexID], 0.0, 1.0);
                texcoord = positions[gl_VertexID] * 0.5 + 0.5;
            }
            """;
    }
    
    private String generateFinalFsh() {
        return """
            // 🎬 VOXELCRAI - FINAL FRAGMENT SHADER
            // final.fsh - Тонмаппинг и финальная коррекция
            
            #version 330 core
            
            in vec2 texcoord;
            
            out vec4 fragColor;
            
            uniform sampler2D colortex0;
            
            // 🎨 ACES Tonemapping
            vec3 ACESFilm(vec3 x) {
                float a = 2.51;
                float b = 0.03;
                float c = 2.43;
                float d = 0.59;
                float e = 0.14;
                return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
            }
            
            void main() {
                vec3 color = texture(colortex0, texcoord).rgb;
                
                // 🎨 Экспозиция
                color *= 1.2;
                
                // 🎨 Тонмаппинг ACES
                color = ACESFilm(color);
                
                // 🎨 Гамма коррекция
                color = pow(color, vec3(1.0 / 2.2));
                
                // 🎨 Небольшая виньетка
                vec2 uv = texcoord * 2.0 - 1.0;
                float vignette = 1.0 - dot(uv, uv) * 0.15;
                color *= vignette;
                
                fragColor = vec4(color, 1.0);
            }
            """;
    }
    
    public Path getShaderpackPath() {
        return shaderpackPath;
    }
}

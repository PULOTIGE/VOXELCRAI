// Pattern Lighting System for Unity 6
// Common types and enums

using System;
using UnityEngine;

namespace PatternLighting
{
    /// <summary>
    /// Light pattern animation types
    /// </summary>
    public enum LightPattern
    {
        Steady = 0,
        Pulse = 1,
        Flicker = 2,
        Strobe = 3,
        Candle = 4,
        Fluorescent = 5,
        Lightning = 6,
        Fire = 7,
        Alarm = 8,
        Underwater = 9,
        Heartbeat = 10,
        Breathing = 11,
        Custom = 12
    }

    /// <summary>
    /// Reflection quality levels
    /// </summary>
    public enum ReflectionQuality
    {
        Low,
        Medium,
        High,
        Ultra
    }

    /// <summary>
    /// Shadow quality levels
    /// </summary>
    public enum ShadowQuality
    {
        Low,
        Medium,
        High,
        Ultra
    }

    /// <summary>
    /// Water quality levels
    /// </summary>
    public enum WaterQuality
    {
        Simple,
        Medium,
        High,
        Ultra
    }

    /// <summary>
    /// Pattern light settings
    /// </summary>
    [Serializable]
    public class PatternLightSettings
    {
        [Tooltip("Pattern animation type")]
        public LightPattern pattern = LightPattern.Steady;

        [Tooltip("Animation speed multiplier")]
        [Range(0.01f, 10f)]
        public float speed = 1f;

        [Tooltip("Phase offset for synchronization")]
        [Range(0f, 1f)]
        public float phaseOffset = 0f;

        [Tooltip("Minimum intensity")]
        [Range(0f, 1f)]
        public float minIntensity = 0f;

        [Tooltip("Maximum intensity")]
        [Range(0f, 10f)]
        public float maxIntensity = 1f;

        [Tooltip("Custom animation curve")]
        public AnimationCurve customCurve = AnimationCurve.Linear(0, 0, 1, 1);

        [Tooltip("Enable color shifting")]
        public bool enableColorShift = false;

        [Tooltip("Color shift gradient")]
        public Gradient colorGradient = new Gradient();
    }

    /// <summary>
    /// Shadow settings
    /// </summary>
    [Serializable]
    public class PatternShadowSettings
    {
        public ShadowQuality quality = ShadowQuality.High;

        [Range(0f, 1f)]
        public float intensity = 1f;

        [Range(0f, 10f)]
        public float softness = 1f;

        [Range(0f, 10f)]
        public float bias = 0.5f;

        public bool contactShadows = true;

        [Range(0f, 1f)]
        public float contactShadowLength = 0.1f;

        [Range(1, 8)]
        public int cascadeCount = 4;
    }

    /// <summary>
    /// Water settings
    /// </summary>
    [Serializable]
    public class WaterSettings
    {
        public WaterQuality quality = WaterQuality.High;

        [Header("Waves")]
        [Range(0f, 2f)]
        public float waveHeight = 0.5f;

        [Range(0.1f, 10f)]
        public float waveSpeed = 1f;

        [Range(0.1f, 50f)]
        public float waveScale = 10f;

        [Header("Appearance")]
        public Color shallowColor = new Color(0.2f, 0.6f, 0.8f, 0.8f);
        public Color deepColor = new Color(0.05f, 0.2f, 0.4f, 1f);

        [Range(0f, 1f)]
        public float transparency = 0.7f;

        [Range(0f, 2f)]
        public float refractionStrength = 0.5f;

        [Header("Reflections")]
        public bool enableReflections = true;

        [Range(0f, 1f)]
        public float reflectionIntensity = 0.8f;

        [Range(1f, 10f)]
        public float fresnelPower = 5f;

        [Header("Foam")]
        public bool enableFoam = true;

        [Range(0f, 2f)]
        public float foamAmount = 0.5f;

        public Color foamColor = Color.white;

        [Header("Caustics")]
        public bool enableCaustics = true;

        [Range(0f, 2f)]
        public float causticIntensity = 0.5f;

        [Range(0.1f, 5f)]
        public float causticSpeed = 1f;
    }

    /// <summary>
    /// Global system configuration
    /// </summary>
    [Serializable]
    public class PatternLightingConfig
    {
        public bool enabled = true;

        [Range(0f, 2f)]
        public float globalIntensity = 1f;

        [Range(0.1f, 5f)]
        public float globalSpeed = 1f;

        public bool enablePBR = true;
        public bool enableSSR = true;
        public bool enableVolumetrics = false;

        [Range(0f, 1f)]
        public float volumetricDensity = 0.1f;
    }

    /// <summary>
    /// Pattern evaluation utilities
    /// </summary>
    public static class PatternEvaluator
    {
        public static float Evaluate(LightPattern pattern, float time, Vector3 worldPos = default)
        {
            float value = 1f;

            switch (pattern)
            {
                case LightPattern.Steady:
                    value = 1f;
                    break;

                case LightPattern.Pulse:
                    value = 0.5f + 0.5f * Mathf.Sin(time * Mathf.PI * 2f);
                    break;

                case LightPattern.Flicker:
                    value = 0.7f + 0.3f * Mathf.Sin(time * 20f) * Mathf.Sin(time * 7.3f);
                    break;

                case LightPattern.Strobe:
                    value = Mathf.Sin(time * 10f) > 0f ? 1f : 0f;
                    break;

                case LightPattern.Candle:
                    value = 0.8f + 0.2f * Mathf.Sin(time * 12f) * Mathf.Sin(time * 5.7f) * Mathf.Sin(time * 3.1f);
                    break;

                case LightPattern.Fluorescent:
                    float startup = Mathf.Clamp01((time % 5f) / 2f);
                    float buzz = 0.05f * Mathf.Sin(time * 120f);
                    value = startup + buzz * startup;
                    break;

                case LightPattern.Lightning:
                    value = Mathf.Pow(Mathf.Max(0f, Mathf.Sin(time * 0.5f)), 20f);
                    break;

                case LightPattern.Fire:
                    value = 0.7f + 0.3f * Mathf.Sin(time * 8f) * Mathf.Sin(time * 4.3f) * Mathf.Sin(time * 2.1f);
                    break;

                case LightPattern.Alarm:
                    value = Mathf.Sin(time * 4f) > 0f ? 1f : 0.2f;
                    break;

                case LightPattern.Underwater:
                    float caustic = Mathf.Sin(worldPos.x * 0.01f + time) * Mathf.Sin(worldPos.z * 0.01f + time * 0.7f);
                    value = 0.7f + 0.3f * caustic;
                    break;

                case LightPattern.Heartbeat:
                    float beat = Mathf.Pow(Mathf.Sin(time * 2.5f), 12f);
                    float beat2 = Mathf.Pow(Mathf.Sin(time * 2.5f + 0.3f), 12f) * 0.5f;
                    value = Mathf.Max(beat, beat2);
                    break;

                case LightPattern.Breathing:
                    value = 0.3f + 0.7f * (Mathf.Sin(time * 0.5f) * 0.5f + 0.5f);
                    break;
            }

            return Mathf.Clamp01(value);
        }
    }
}

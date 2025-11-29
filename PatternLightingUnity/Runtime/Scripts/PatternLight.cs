// Pattern Lighting System for Unity 6
// Pattern Light Component

using UnityEngine;
using UnityEngine.Rendering;

namespace PatternLighting
{
    /// <summary>
    /// Pattern-based animated light component
    /// </summary>
    [RequireComponent(typeof(Light))]
    [AddComponentMenu("Pattern Lighting/Pattern Light")]
    [ExecuteAlways]
    public class PatternLight : MonoBehaviour
    {
        [Header("Pattern Settings")]
        public PatternLightSettings settings = new PatternLightSettings();

        [Header("Base Properties")]
        public Color baseColor = Color.white;
        public float baseIntensity = 1f;

        [Header("Shadow Settings")]
        public bool castPatternShadows = true;
        public PatternShadowSettings shadowSettings = new PatternShadowSettings();

        [Header("Sync")]
        [Tooltip("Lights with same group name sync their animations")]
        public string syncGroup = "";

        // Components
        private Light _light;
        private float _currentTime;
        private float _flashTimer;
        private float _flashIntensity = 1f;

        // Cached values
        private float _currentIntensity;
        private Color _currentColor;

        public Light Light => _light;
        public float CurrentIntensity => _currentIntensity;
        public Color CurrentColor => _currentColor;

        private void Awake()
        {
            _light = GetComponent<Light>();
        }

        private void OnEnable()
        {
            if (_light == null)
                _light = GetComponent<Light>();

            // Random phase if not synced
            if (string.IsNullOrEmpty(syncGroup) && settings.phaseOffset == 0f)
            {
                settings.phaseOffset = Random.value;
            }

            _currentTime = settings.phaseOffset;

            // Register with manager
            PatternLightingManager.Instance?.RegisterLight(this);
        }

        private void OnDisable()
        {
            PatternLightingManager.Instance?.UnregisterLight(this);
        }

        private void Update()
        {
            if (_light == null) return;

            // Get delta time (works in editor too)
            float deltaTime = Application.isPlaying ? Time.deltaTime : 0.016f;

            // Update time
            _currentTime += deltaTime * settings.speed;

            // Update flash
            if (_flashTimer > 0f)
            {
                _flashTimer -= deltaTime;
                if (_flashTimer <= 0f)
                    _flashIntensity = 1f;
            }

            // Calculate pattern value
            float patternValue = EvaluatePattern();

            // Apply intensity
            _currentIntensity = baseIntensity * patternValue * _flashIntensity;
            _light.intensity = _currentIntensity * (PatternLightingManager.Instance?.Config.globalIntensity ?? 1f);

            // Apply color
            if (settings.enableColorShift && settings.colorGradient != null)
            {
                _currentColor = settings.colorGradient.Evaluate(_currentTime % 1f);
            }
            else
            {
                _currentColor = baseColor;
            }
            _light.color = _currentColor;

            // Apply shadow settings
            UpdateShadowSettings();
        }

        private float EvaluatePattern()
        {
            float rawValue;

            if (settings.pattern == LightPattern.Custom && settings.customCurve != null)
            {
                rawValue = settings.customCurve.Evaluate(_currentTime % 1f);
            }
            else
            {
                rawValue = PatternEvaluator.Evaluate(settings.pattern, _currentTime, transform.position);
            }

            // Map to min/max range
            return Mathf.Lerp(settings.minIntensity, settings.maxIntensity, rawValue);
        }

        private void UpdateShadowSettings()
        {
            if (!castPatternShadows)
            {
                _light.shadows = LightShadows.None;
                return;
            }

            _light.shadows = shadowSettings.quality >= ShadowQuality.High
                ? LightShadows.Soft
                : LightShadows.Hard;

            _light.shadowStrength = shadowSettings.intensity;
            _light.shadowBias = shadowSettings.bias * 0.01f;
        }

        /// <summary>
        /// Trigger a flash effect
        /// </summary>
        public void TriggerFlash(float duration = 0.1f, float intensity = 10f)
        {
            _flashTimer = duration;
            _flashIntensity = intensity;
        }

        /// <summary>
        /// Sync with another pattern light
        /// </summary>
        public void SyncWith(PatternLight other)
        {
            if (other != null)
            {
                _currentTime = other._currentTime;
                settings.phaseOffset = other.settings.phaseOffset;
            }
        }

        /// <summary>
        /// Set pattern at runtime
        /// </summary>
        public void SetPattern(LightPattern pattern)
        {
            settings.pattern = pattern;
        }

        /// <summary>
        /// Set pattern speed
        /// </summary>
        public void SetSpeed(float speed)
        {
            settings.speed = Mathf.Clamp(speed, 0.01f, 10f);
        }

        private void OnDrawGizmosSelected()
        {
            if (_light == null) return;

            Gizmos.color = _currentColor;

            switch (_light.type)
            {
                case LightType.Point:
                    Gizmos.DrawWireSphere(transform.position, _light.range);
                    break;

                case LightType.Spot:
                    // Draw cone
                    float angle = _light.spotAngle * Mathf.Deg2Rad * 0.5f;
                    float range = _light.range;
                    float radius = Mathf.Tan(angle) * range;
                    
                    Vector3 forward = transform.forward * range;
                    Gizmos.DrawLine(transform.position, transform.position + forward);
                    Gizmos.DrawWireSphere(transform.position + forward, radius);
                    break;
            }
        }
    }
}

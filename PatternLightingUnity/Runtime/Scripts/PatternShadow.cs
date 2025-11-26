// Pattern Lighting System for Unity 6
// Enhanced Shadow Component

using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.Rendering.Universal;

namespace PatternLighting
{
    /// <summary>
    /// Enhanced shadow controller with cascades and contact shadows
    /// </summary>
    [AddComponentMenu("Pattern Lighting/Pattern Shadow Controller")]
    [ExecuteAlways]
    public class PatternShadow : MonoBehaviour
    {
        [Header("Shadow Settings")]
        public PatternShadowSettings settings = new PatternShadowSettings();

        [Header("Cascade Settings")]
        [Range(1000f, 100000f)]
        public float shadowDistance = 50000f;

        public float[] cascadeDistances = { 0.1f, 0.2f, 0.4f, 1f };

        [Header("Shadow Color")]
        public Color shadowColor = new Color(0f, 0f, 0.1f, 1f);

        [Header("Volumetric")]
        public bool volumetricShadows = false;

        [Range(8, 64)]
        public int volumetricSamples = 16;

        private Light _directionalLight;
        private UniversalAdditionalLightData _additionalLightData;

        private void OnEnable()
        {
            _directionalLight = GetComponent<Light>();
            if (_directionalLight != null)
            {
                _additionalLightData = _directionalLight.GetComponent<UniversalAdditionalLightData>();
            }

            ApplySettings();
        }

        private void Update()
        {
            ApplySettings();
        }

        public void ApplySettings()
        {
            if (_directionalLight == null) return;

            // Basic shadow settings
            _directionalLight.shadows = settings.quality >= ShadowQuality.High
                ? LightShadows.Soft
                : LightShadows.Hard;

            _directionalLight.shadowStrength = settings.intensity;
            _directionalLight.shadowBias = settings.bias * 0.01f;
            _directionalLight.shadowNormalBias = settings.softness * 0.1f;

            // URP specific settings
            if (_additionalLightData != null)
            {
                _additionalLightData.useSoftShadows = settings.quality >= ShadowQuality.High;
            }
        }

        /// <summary>
        /// Calculate cascade split distances
        /// </summary>
        public float[] CalculateCascadeSplits(float nearPlane, float farPlane)
        {
            float[] splits = new float[settings.cascadeCount + 1];
            float lambda = 0.5f; // Blend between log and linear

            for (int i = 0; i <= settings.cascadeCount; i++)
            {
                float p = (float)i / settings.cascadeCount;

                // Logarithmic split
                float logSplit = nearPlane * Mathf.Pow(farPlane / nearPlane, p);

                // Linear split
                float linearSplit = nearPlane + (farPlane - nearPlane) * p;

                // Blend
                splits[i] = Mathf.Lerp(linearSplit, logSplit, lambda);
            }

            return splits;
        }

        /// <summary>
        /// Calculate soft shadow penumbra size
        /// </summary>
        public static float CalculatePenumbra(float lightRadius, float occluderDistance, float receiverDistance)
        {
            if (occluderDistance <= 0f || receiverDistance <= occluderDistance)
                return 0f;

            return lightRadius * (receiverDistance - occluderDistance) / occluderDistance;
        }

        private void OnDrawGizmosSelected()
        {
            if (_directionalLight == null) return;

            // Draw shadow direction
            Gizmos.color = Color.yellow;
            Gizmos.DrawRay(transform.position, transform.forward * 10f);

            // Draw cascade regions (simplified)
            Gizmos.color = new Color(1f, 0.5f, 0f, 0.3f);
            for (int i = 0; i < cascadeDistances.Length && i < settings.cascadeCount; i++)
            {
                float dist = cascadeDistances[i] * shadowDistance;
                Gizmos.DrawWireCube(transform.position + transform.forward * dist * 0.5f,
                    new Vector3(dist, dist, dist) * 0.2f);
            }
        }
    }

    /// <summary>
    /// Contact shadow volume
    /// </summary>
    [AddComponentMenu("Pattern Lighting/Contact Shadow Volume")]
    [RequireComponent(typeof(BoxCollider))]
    public class ContactShadowVolume : MonoBehaviour
    {
        [Range(0f, 1f)]
        public float intensity = 1f;

        [Range(0f, 1f)]
        public float length = 0.1f;

        [Range(0f, 1f)]
        public float fadeDistance = 0.2f;

        public bool affectTransparent = false;

        private BoxCollider _collider;

        private void Awake()
        {
            _collider = GetComponent<BoxCollider>();
            _collider.isTrigger = true;
        }

        public bool IsPointInside(Vector3 worldPoint)
        {
            return _collider.bounds.Contains(worldPoint);
        }

        private void OnDrawGizmos()
        {
            if (_collider == null)
                _collider = GetComponent<BoxCollider>();

            Gizmos.color = new Color(0.5f, 0f, 0.5f, 0.3f);
            Gizmos.matrix = transform.localToWorldMatrix;
            Gizmos.DrawCube(_collider.center, _collider.size);

            Gizmos.color = new Color(0.5f, 0f, 0.5f, 1f);
            Gizmos.DrawWireCube(_collider.center, _collider.size);
        }
    }

    /// <summary>
    /// Shadow caster component for dynamic objects
    /// </summary>
    [AddComponentMenu("Pattern Lighting/Shadow Caster")]
    [ExecuteAlways]
    public class PatternShadowCaster : MonoBehaviour
    {
        public bool castShadows = true;
        public bool receiveShadows = true;

        [Range(0f, 1f)]
        public float shadowBias = 0f;

        public ShadowCastingMode shadowCastingMode = ShadowCastingMode.On;

        private Renderer[] _renderers;

        private void OnEnable()
        {
            _renderers = GetComponentsInChildren<Renderer>();
            ApplySettings();
        }

        private void Update()
        {
            if (!Application.isPlaying)
            {
                ApplySettings();
            }
        }

        public void ApplySettings()
        {
            if (_renderers == null) return;

            foreach (var renderer in _renderers)
            {
                if (renderer == null) continue;

                renderer.shadowCastingMode = castShadows ? shadowCastingMode : ShadowCastingMode.Off;
                renderer.receiveShadows = receiveShadows;
            }
        }
    }
}

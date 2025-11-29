// Pattern Lighting System for Unity 6
// Advanced Water System

using UnityEngine;

namespace PatternLighting
{
    /// <summary>
    /// Advanced water surface with waves, reflections, and caustics
    /// </summary>
    [AddComponentMenu("Pattern Lighting/Pattern Water")]
    [RequireComponent(typeof(MeshRenderer))]
    [ExecuteAlways]
    public class PatternWater : MonoBehaviour
    {
        [Header("Water Settings")]
        public WaterSettings settings = new WaterSettings();

        [Header("References")]
        public Camera reflectionCamera;
        public RenderTexture reflectionTexture;

        [Header("Performance")]
        public bool updateEveryFrame = true;

        [Range(1, 4)]
        public int reflectionDownsample = 2;

        // Components
        private MeshRenderer _renderer;
        private MaterialPropertyBlock _propertyBlock;
        private float _time;

        // Shader property IDs
        private static readonly int ShallowColorID = Shader.PropertyToID("_ShallowColor");
        private static readonly int DeepColorID = Shader.PropertyToID("_DeepColor");
        private static readonly int WaveHeightID = Shader.PropertyToID("_WaveHeight");
        private static readonly int WaveSpeedID = Shader.PropertyToID("_WaveSpeed");
        private static readonly int WaveScaleID = Shader.PropertyToID("_WaveScale");
        private static readonly int TransparencyID = Shader.PropertyToID("_Transparency");
        private static readonly int RefractionStrengthID = Shader.PropertyToID("_RefractionStrength");
        private static readonly int ReflectionIntensityID = Shader.PropertyToID("_ReflectionIntensity");
        private static readonly int FresnelPowerID = Shader.PropertyToID("_FresnelPower");
        private static readonly int FoamAmountID = Shader.PropertyToID("_FoamAmount");
        private static readonly int FoamColorID = Shader.PropertyToID("_FoamColor");
        private static readonly int CausticIntensityID = Shader.PropertyToID("_CausticIntensity");
        private static readonly int CausticSpeedID = Shader.PropertyToID("_CausticSpeed");
        private static readonly int TimeID = Shader.PropertyToID("_Time");
        private static readonly int ReflectionTexID = Shader.PropertyToID("_ReflectionTex");

        private void Awake()
        {
            _renderer = GetComponent<MeshRenderer>();
            _propertyBlock = new MaterialPropertyBlock();
        }

        private void OnEnable()
        {
            PatternLightingManager.Instance?.RegisterWater(this);
            SetupReflectionCamera();
        }

        private void OnDisable()
        {
            PatternLightingManager.Instance?.UnregisterWater(this);
            CleanupReflection();
        }

        private void Update()
        {
            if (_renderer == null) return;

            float deltaTime = Application.isPlaying ? Time.deltaTime : 0.016f;
            _time += deltaTime;

            UpdateMaterialProperties();

            if (updateEveryFrame && settings.enableReflections)
            {
                UpdateReflection();
            }
        }

        private void UpdateMaterialProperties()
        {
            _renderer.GetPropertyBlock(_propertyBlock);

            // Colors
            _propertyBlock.SetColor(ShallowColorID, settings.shallowColor);
            _propertyBlock.SetColor(DeepColorID, settings.deepColor);

            // Waves
            _propertyBlock.SetFloat(WaveHeightID, settings.waveHeight);
            _propertyBlock.SetFloat(WaveSpeedID, settings.waveSpeed);
            _propertyBlock.SetFloat(WaveScaleID, settings.waveScale);

            // Appearance
            _propertyBlock.SetFloat(TransparencyID, settings.transparency);
            _propertyBlock.SetFloat(RefractionStrengthID, settings.refractionStrength);

            // Reflections
            _propertyBlock.SetFloat(ReflectionIntensityID, settings.enableReflections ? settings.reflectionIntensity : 0f);
            _propertyBlock.SetFloat(FresnelPowerID, settings.fresnelPower);

            // Foam
            _propertyBlock.SetFloat(FoamAmountID, settings.enableFoam ? settings.foamAmount : 0f);
            _propertyBlock.SetColor(FoamColorID, settings.foamColor);

            // Caustics
            _propertyBlock.SetFloat(CausticIntensityID, settings.enableCaustics ? settings.causticIntensity : 0f);
            _propertyBlock.SetFloat(CausticSpeedID, settings.causticSpeed);

            // Time
            _propertyBlock.SetFloat(TimeID, _time);

            // Reflection texture
            if (reflectionTexture != null)
            {
                _propertyBlock.SetTexture(ReflectionTexID, reflectionTexture);
            }

            _renderer.SetPropertyBlock(_propertyBlock);
        }

        private void SetupReflectionCamera()
        {
            if (!settings.enableReflections) return;

            if (reflectionCamera == null)
            {
                var go = new GameObject("Water Reflection Camera");
                go.transform.SetParent(transform);
                reflectionCamera = go.AddComponent<Camera>();
                reflectionCamera.enabled = false;
            }

            // Setup render texture
            int width = Screen.width / reflectionDownsample;
            int height = Screen.height / reflectionDownsample;

            if (reflectionTexture == null || reflectionTexture.width != width || reflectionTexture.height != height)
            {
                if (reflectionTexture != null)
                    reflectionTexture.Release();

                reflectionTexture = new RenderTexture(width, height, 16, RenderTextureFormat.ARGB32);
                reflectionTexture.name = "Water Reflection";
            }

            reflectionCamera.targetTexture = reflectionTexture;
        }

        private void UpdateReflection()
        {
            if (reflectionCamera == null || Camera.main == null) return;

            var mainCam = Camera.main;

            // Mirror camera position
            Vector3 pos = mainCam.transform.position;
            Vector3 normal = transform.up;
            float d = -Vector3.Dot(normal, transform.position);
            Vector4 reflectionPlane = new Vector4(normal.x, normal.y, normal.z, d);

            Matrix4x4 reflection = Matrix4x4.zero;
            CalculateReflectionMatrix(ref reflection, reflectionPlane);

            reflectionCamera.worldToCameraMatrix = mainCam.worldToCameraMatrix * reflection;

            // Oblique projection matrix for clipping
            Vector4 clipPlane = CameraSpacePlane(reflectionCamera, transform.position, normal, 1.0f);
            reflectionCamera.projectionMatrix = mainCam.CalculateObliqueMatrix(clipPlane);

            reflectionCamera.cullingMask = ~(1 << 4); // Exclude water layer
            reflectionCamera.Render();
        }

        private void CleanupReflection()
        {
            if (reflectionTexture != null)
            {
                reflectionTexture.Release();
                DestroyImmediate(reflectionTexture);
            }

            if (reflectionCamera != null)
            {
                DestroyImmediate(reflectionCamera.gameObject);
            }
        }

        private static void CalculateReflectionMatrix(ref Matrix4x4 reflectionMat, Vector4 plane)
        {
            reflectionMat.m00 = 1f - 2f * plane.x * plane.x;
            reflectionMat.m01 = -2f * plane.x * plane.y;
            reflectionMat.m02 = -2f * plane.x * plane.z;
            reflectionMat.m03 = -2f * plane.w * plane.x;

            reflectionMat.m10 = -2f * plane.y * plane.x;
            reflectionMat.m11 = 1f - 2f * plane.y * plane.y;
            reflectionMat.m12 = -2f * plane.y * plane.z;
            reflectionMat.m13 = -2f * plane.w * plane.y;

            reflectionMat.m20 = -2f * plane.z * plane.x;
            reflectionMat.m21 = -2f * plane.z * plane.y;
            reflectionMat.m22 = 1f - 2f * plane.z * plane.z;
            reflectionMat.m23 = -2f * plane.w * plane.z;

            reflectionMat.m30 = 0f;
            reflectionMat.m31 = 0f;
            reflectionMat.m32 = 0f;
            reflectionMat.m33 = 1f;
        }

        private Vector4 CameraSpacePlane(Camera cam, Vector3 pos, Vector3 normal, float sideSign)
        {
            Vector3 offsetPos = pos + normal * 0.07f;
            Matrix4x4 m = cam.worldToCameraMatrix;
            Vector3 cpos = m.MultiplyPoint(offsetPos);
            Vector3 cnormal = m.MultiplyVector(normal).normalized * sideSign;
            return new Vector4(cnormal.x, cnormal.y, cnormal.z, -Vector3.Dot(cpos, cnormal));
        }

        /// <summary>
        /// Get wave height at world position
        /// </summary>
        public float GetWaveHeightAt(Vector3 worldPosition)
        {
            float x = worldPosition.x / settings.waveScale;
            float z = worldPosition.z / settings.waveScale;
            float t = _time * settings.waveSpeed;

            // Simple Gerstner-like wave approximation
            float wave1 = Mathf.Sin(x + t) * 0.5f;
            float wave2 = Mathf.Sin(z * 0.7f + t * 1.3f) * 0.3f;
            float wave3 = Mathf.Sin((x + z) * 0.5f + t * 0.8f) * 0.2f;

            return transform.position.y + (wave1 + wave2 + wave3) * settings.waveHeight;
        }

        /// <summary>
        /// Check if a point is underwater
        /// </summary>
        public bool IsUnderwater(Vector3 worldPosition)
        {
            return worldPosition.y < GetWaveHeightAt(worldPosition);
        }

        private void OnDrawGizmosSelected()
        {
            Gizmos.color = new Color(0.2f, 0.6f, 1f, 0.3f);
            Gizmos.matrix = transform.localToWorldMatrix;

            // Draw water plane
            Gizmos.DrawCube(Vector3.zero, new Vector3(10, 0.1f, 10));

            // Draw wave direction
            Gizmos.color = Color.cyan;
            Gizmos.DrawRay(Vector3.zero, Vector3.right * 2f);
        }
    }
}

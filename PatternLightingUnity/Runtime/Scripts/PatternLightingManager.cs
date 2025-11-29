// Pattern Lighting System for Unity 6
// Global Manager

using System.Collections.Generic;
using System.Linq;
using UnityEngine;

namespace PatternLighting
{
    /// <summary>
    /// Global manager for pattern lighting system
    /// </summary>
    [DefaultExecutionOrder(-100)]
    public class PatternLightingManager : MonoBehaviour
    {
        private static PatternLightingManager _instance;
        public static PatternLightingManager Instance
        {
            get
            {
                if (_instance == null)
                {
                    _instance = FindFirstObjectByType<PatternLightingManager>();
                    if (_instance == null)
                    {
                        var go = new GameObject("Pattern Lighting Manager");
                        _instance = go.AddComponent<PatternLightingManager>();
                        DontDestroyOnLoad(go);
                    }
                }
                return _instance;
            }
        }

        [Header("Global Configuration")]
        public PatternLightingConfig Config = new PatternLightingConfig();

        [Header("Debug")]
        public bool showDebugInfo = false;

        // Registered components
        private readonly List<PatternLight> _lights = new List<PatternLight>();
        private readonly List<PatternWater> _waterSurfaces = new List<PatternWater>();
        private readonly Dictionary<string, List<PatternLight>> _syncGroups = new Dictionary<string, List<PatternLight>>();

        // Master time for synced animations
        public float MasterTime { get; private set; }

        private void Awake()
        {
            if (_instance != null && _instance != this)
            {
                Destroy(gameObject);
                return;
            }

            _instance = this;
            DontDestroyOnLoad(gameObject);
        }

        private void Update()
        {
            if (!Config.enabled) return;

            // Update master time
            MasterTime += Time.deltaTime * Config.globalSpeed;

            // Clean up destroyed references
            CleanupStaleReferences();
        }

        // ====================================================================
        // Registration
        // ====================================================================

        public void RegisterLight(PatternLight light)
        {
            if (light != null && !_lights.Contains(light))
            {
                _lights.Add(light);

                // Add to sync group
                if (!string.IsNullOrEmpty(light.syncGroup))
                {
                    if (!_syncGroups.ContainsKey(light.syncGroup))
                        _syncGroups[light.syncGroup] = new List<PatternLight>();

                    _syncGroups[light.syncGroup].Add(light);
                }
            }
        }

        public void UnregisterLight(PatternLight light)
        {
            _lights.Remove(light);

            if (light != null && !string.IsNullOrEmpty(light.syncGroup))
            {
                if (_syncGroups.TryGetValue(light.syncGroup, out var group))
                {
                    group.Remove(light);
                }
            }
        }

        public void RegisterWater(PatternWater water)
        {
            if (water != null && !_waterSurfaces.Contains(water))
            {
                _waterSurfaces.Add(water);
            }
        }

        public void UnregisterWater(PatternWater water)
        {
            _waterSurfaces.Remove(water);
        }

        // ====================================================================
        // Queries
        // ====================================================================

        public List<PatternLight> GetLightsInSyncGroup(string groupName)
        {
            if (_syncGroups.TryGetValue(groupName, out var group))
                return group.Where(l => l != null).ToList();

            return new List<PatternLight>();
        }

        public List<PatternLight> GetLightsAtPosition(Vector3 position, float radius = 100f)
        {
            return _lights
                .Where(l => l != null && Vector3.Distance(l.transform.position, position) <= radius)
                .ToList();
        }

        public float GetCombinedIntensityAt(Vector3 position)
        {
            float total = 0f;

            foreach (var light in _lights)
            {
                if (light == null || light.Light == null) continue;

                float distance = Vector3.Distance(position, light.transform.position);
                float range = light.Light.range;

                if (distance < range)
                {
                    float falloff = 1f - (distance / range);
                    total += light.CurrentIntensity * falloff * falloff;
                }
            }

            return total * Config.globalIntensity;
        }

        public Color GetCombinedColorAt(Vector3 position)
        {
            Color totalColor = Color.black;
            float totalWeight = 0f;

            foreach (var light in _lights)
            {
                if (light == null || light.Light == null) continue;

                float distance = Vector3.Distance(position, light.transform.position);
                float range = light.Light.range;

                if (distance < range)
                {
                    float weight = (1f - distance / range) * light.CurrentIntensity;
                    totalColor += light.CurrentColor * weight;
                    totalWeight += weight;
                }
            }

            if (totalWeight > 0f)
                totalColor /= totalWeight;

            return totalColor;
        }

        // ====================================================================
        // Global Control
        // ====================================================================

        public void SetGlobalIntensity(float intensity)
        {
            Config.globalIntensity = Mathf.Clamp(intensity, 0f, 2f);
        }

        public void SetGlobalSpeed(float speed)
        {
            Config.globalSpeed = Mathf.Clamp(speed, 0.1f, 5f);
        }

        public void TriggerFlashAtPosition(Vector3 position, float radius, float duration = 0.1f, float intensity = 10f)
        {
            foreach (var light in _lights)
            {
                if (light == null) continue;

                float distance = Vector3.Distance(position, light.transform.position);
                if (distance < radius)
                {
                    float falloff = 1f - distance / radius;
                    light.TriggerFlash(duration, intensity * falloff);
                }
            }
        }

        public void SyncGroup(string groupName)
        {
            var lights = GetLightsInSyncGroup(groupName);
            if (lights.Count > 1)
            {
                var master = lights[0];
                for (int i = 1; i < lights.Count; i++)
                {
                    lights[i].SyncWith(master);
                }
            }
        }

        // ====================================================================
        // Stats
        // ====================================================================

        public string GetStatsString()
        {
            return $"Pattern Lights: {_lights.Count(l => l != null)}\n" +
                   $"Water Surfaces: {_waterSurfaces.Count(w => w != null)}\n" +
                   $"Sync Groups: {_syncGroups.Count}\n" +
                   $"Master Time: {MasterTime:F2}";
        }

        private void CleanupStaleReferences()
        {
            _lights.RemoveAll(l => l == null);
            _waterSurfaces.RemoveAll(w => w == null);

            foreach (var group in _syncGroups.Values)
            {
                group.RemoveAll(l => l == null);
            }
        }

        private void OnGUI()
        {
            if (!showDebugInfo) return;

            GUILayout.BeginArea(new Rect(10, 10, 300, 200));
            GUILayout.BeginVertical("box");
            GUILayout.Label("Pattern Lighting System", GUI.skin.box);
            GUILayout.Label(GetStatsString());
            GUILayout.EndVertical();
            GUILayout.EndArea();
        }
    }
}

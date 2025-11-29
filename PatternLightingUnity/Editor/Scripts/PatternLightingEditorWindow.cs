// Pattern Lighting System for Unity 6
// Main Editor Window

using UnityEngine;
using UnityEditor;
using System.Collections.Generic;
using System.Linq;

namespace PatternLighting.Editor
{
    public class PatternLightingEditorWindow : EditorWindow
    {
        private Vector2 _scrollPos;
        private int _selectedTab;
        private readonly string[] _tabs = { "Global", "Lights", "Water", "Materials", "Debug" };

        // Global settings
        private PatternLightingConfig _config = new PatternLightingConfig();

        // Preview
        private float _previewTime;
        private LightPattern _previewPattern = LightPattern.Pulse;
        private AnimationCurve _previewCurve;
        private readonly List<float> _graphHistory = new List<float>();
        private const int MaxGraphPoints = 100;

        [MenuItem("Window/Pattern Lighting/Control Panel %#l")]
        public static void ShowWindow()
        {
            var window = GetWindow<PatternLightingEditorWindow>();
            window.titleContent = new GUIContent("Pattern Lighting", EditorGUIUtility.IconContent("Light Icon").image);
            window.minSize = new Vector2(400, 500);
        }

        private void OnEnable()
        {
            RefreshConfig();
            EditorApplication.update += OnEditorUpdate;
        }

        private void OnDisable()
        {
            EditorApplication.update -= OnEditorUpdate;
        }

        private void OnEditorUpdate()
        {
            // Update preview
            _previewTime += 0.016f;

            float value = PatternEvaluator.Evaluate(_previewPattern, _previewTime);
            _graphHistory.Add(value);
            if (_graphHistory.Count > MaxGraphPoints)
                _graphHistory.RemoveAt(0);

            Repaint();
        }

        private void RefreshConfig()
        {
            var manager = FindFirstObjectByType<PatternLightingManager>();
            if (manager != null)
            {
                _config = manager.Config;
            }
        }

        private void OnGUI()
        {
            // Header
            EditorGUILayout.BeginHorizontal(EditorStyles.toolbar);
            GUILayout.Label("Pattern Lighting System", EditorStyles.boldLabel);
            GUILayout.FlexibleSpace();
            if (GUILayout.Button("Refresh", EditorStyles.toolbarButton))
                RefreshConfig();
            EditorGUILayout.EndHorizontal();

            // Tabs
            _selectedTab = GUILayout.Toolbar(_selectedTab, _tabs);

            _scrollPos = EditorGUILayout.BeginScrollView(_scrollPos);

            switch (_selectedTab)
            {
                case 0: DrawGlobalTab(); break;
                case 1: DrawLightsTab(); break;
                case 2: DrawWaterTab(); break;
                case 3: DrawMaterialsTab(); break;
                case 4: DrawDebugTab(); break;
            }

            EditorGUILayout.EndScrollView();
        }

        private void DrawGlobalTab()
        {
            EditorGUILayout.Space(10);
            EditorGUILayout.LabelField("Global Settings", EditorStyles.boldLabel);

            EditorGUI.BeginChangeCheck();

            _config.enabled = EditorGUILayout.Toggle("Enabled", _config.enabled);
            _config.globalIntensity = EditorGUILayout.Slider("Global Intensity", _config.globalIntensity, 0f, 2f);
            _config.globalSpeed = EditorGUILayout.Slider("Global Speed", _config.globalSpeed, 0.1f, 5f);

            EditorGUILayout.Space(10);
            EditorGUILayout.LabelField("Features", EditorStyles.boldLabel);

            _config.enablePBR = EditorGUILayout.Toggle("Enable PBR", _config.enablePBR);
            _config.enableSSR = EditorGUILayout.Toggle("Enable SSR", _config.enableSSR);
            _config.enableVolumetrics = EditorGUILayout.Toggle("Enable Volumetrics", _config.enableVolumetrics);

            if (_config.enableVolumetrics)
            {
                EditorGUI.indentLevel++;
                _config.volumetricDensity = EditorGUILayout.Slider("Volumetric Density", _config.volumetricDensity, 0f, 1f);
                EditorGUI.indentLevel--;
            }

            if (EditorGUI.EndChangeCheck())
            {
                ApplyConfig();
            }

            EditorGUILayout.Space(20);

            // Pattern Preview
            EditorGUILayout.LabelField("Pattern Preview", EditorStyles.boldLabel);

            _previewPattern = (LightPattern)EditorGUILayout.EnumPopup("Pattern", _previewPattern);

            // Draw graph
            Rect graphRect = GUILayoutUtility.GetRect(GUIContent.none, GUIStyle.none, GUILayout.Height(100));
            EditorGUI.DrawRect(graphRect, new Color(0.1f, 0.1f, 0.1f));

            if (_graphHistory.Count > 1)
            {
                Handles.color = Color.cyan;
                for (int i = 1; i < _graphHistory.Count; i++)
                {
                    float x1 = graphRect.x + (i - 1) * graphRect.width / MaxGraphPoints;
                    float y1 = graphRect.yMax - _graphHistory[i - 1] * graphRect.height;
                    float x2 = graphRect.x + i * graphRect.width / MaxGraphPoints;
                    float y2 = graphRect.yMax - _graphHistory[i] * graphRect.height;

                    Handles.DrawLine(new Vector3(x1, y1), new Vector3(x2, y2));
                }
            }

            // Current value
            if (_graphHistory.Count > 0)
            {
                float currentValue = _graphHistory[_graphHistory.Count - 1];
                EditorGUILayout.LabelField($"Current Value: {currentValue:F2}");

                // Color preview
                Color previewColor = Color.Lerp(Color.black, Color.yellow, currentValue);
                Rect colorRect = GUILayoutUtility.GetRect(GUIContent.none, GUIStyle.none, GUILayout.Height(30));
                EditorGUI.DrawRect(colorRect, previewColor);
            }
        }

        private void DrawLightsTab()
        {
            EditorGUILayout.Space(10);
            EditorGUILayout.LabelField("Pattern Lights", EditorStyles.boldLabel);

            var lights = FindObjectsByType<PatternLight>(FindObjectsSortMode.None);

            EditorGUILayout.LabelField($"Total: {lights.Length}");

            EditorGUILayout.Space(5);

            // Actions
            EditorGUILayout.BeginHorizontal();
            if (GUILayout.Button("Select All"))
            {
                Selection.objects = lights.Select(l => l.gameObject).ToArray();
            }
            if (GUILayout.Button("Create New"))
            {
                CreatePatternLight();
            }
            EditorGUILayout.EndHorizontal();

            EditorGUILayout.Space(10);

            // List lights
            foreach (var light in lights)
            {
                EditorGUILayout.BeginHorizontal("box");

                EditorGUILayout.LabelField(light.name, GUILayout.Width(150));
                EditorGUILayout.LabelField(light.settings.pattern.ToString(), GUILayout.Width(100));

                if (GUILayout.Button("Select", GUILayout.Width(60)))
                {
                    Selection.activeGameObject = light.gameObject;
                    SceneView.FrameLastActiveSceneView();
                }

                if (GUILayout.Button("Flash", GUILayout.Width(50)))
                {
                    light.TriggerFlash();
                }

                EditorGUILayout.EndHorizontal();
            }

            EditorGUILayout.Space(10);

            // Sync groups
            EditorGUILayout.LabelField("Sync Groups", EditorStyles.boldLabel);

            var groups = lights
                .Where(l => !string.IsNullOrEmpty(l.syncGroup))
                .GroupBy(l => l.syncGroup);

            foreach (var group in groups)
            {
                EditorGUILayout.BeginHorizontal();
                EditorGUILayout.LabelField($"{group.Key}: {group.Count()} lights");

                if (GUILayout.Button("Sync", GUILayout.Width(60)))
                {
                    var manager = FindFirstObjectByType<PatternLightingManager>();
                    manager?.SyncGroup(group.Key);
                }

                EditorGUILayout.EndHorizontal();
            }
        }

        private void DrawWaterTab()
        {
            EditorGUILayout.Space(10);
            EditorGUILayout.LabelField("Water Surfaces", EditorStyles.boldLabel);

            var waters = FindObjectsByType<PatternWater>(FindObjectsSortMode.None);

            EditorGUILayout.LabelField($"Total: {waters.Length}");

            EditorGUILayout.Space(5);

            if (GUILayout.Button("Create Water Plane"))
            {
                CreateWaterPlane();
            }

            EditorGUILayout.Space(10);

            foreach (var water in waters)
            {
                EditorGUILayout.BeginVertical("box");

                EditorGUILayout.BeginHorizontal();
                EditorGUILayout.LabelField(water.name, EditorStyles.boldLabel);

                if (GUILayout.Button("Select", GUILayout.Width(60)))
                {
                    Selection.activeGameObject = water.gameObject;
                }
                EditorGUILayout.EndHorizontal();

                EditorGUI.indentLevel++;
                EditorGUILayout.LabelField($"Quality: {water.settings.quality}");
                EditorGUILayout.LabelField($"Wave Height: {water.settings.waveHeight:F2}");
                EditorGUILayout.LabelField($"Reflections: {(water.settings.enableReflections ? "On" : "Off")}");
                EditorGUI.indentLevel--;

                EditorGUILayout.EndVertical();
            }
        }

        private void DrawMaterialsTab()
        {
            EditorGUILayout.Space(10);
            EditorGUILayout.LabelField("Create Materials", EditorStyles.boldLabel);

            EditorGUILayout.Space(5);

            if (GUILayout.Button("Create PBR Material"))
            {
                CreateMaterial("Pattern Lighting/PBR", "New Pattern PBR Material");
            }

            if (GUILayout.Button("Create Emissive Material"))
            {
                CreateMaterial("Pattern Lighting/Emissive", "New Pattern Emissive Material");
            }

            if (GUILayout.Button("Create Water Material"))
            {
                CreateMaterial("Pattern Lighting/Water", "New Pattern Water Material");
            }

            EditorGUILayout.Space(20);

            EditorGUILayout.LabelField("Presets", EditorStyles.boldLabel);

            EditorGUILayout.BeginVertical("box");

            if (GUILayout.Button("Neon Sign"))
            {
                CreatePresetMaterial("NeonSign", Color.magenta, LightPattern.Pulse);
            }

            if (GUILayout.Button("Fire Light"))
            {
                CreatePresetMaterial("FireLight", new Color(1, 0.5f, 0.1f), LightPattern.Fire);
            }

            if (GUILayout.Button("Alarm"))
            {
                CreatePresetMaterial("Alarm", Color.red, LightPattern.Alarm);
            }

            if (GUILayout.Button("Candle"))
            {
                CreatePresetMaterial("Candle", new Color(1, 0.8f, 0.4f), LightPattern.Candle);
            }

            EditorGUILayout.EndVertical();
        }

        private void DrawDebugTab()
        {
            EditorGUILayout.Space(10);
            EditorGUILayout.LabelField("Debug Info", EditorStyles.boldLabel);

            var manager = FindFirstObjectByType<PatternLightingManager>();

            if (manager != null)
            {
                EditorGUILayout.TextArea(manager.GetStatsString(), GUILayout.Height(100));

                EditorGUILayout.Space(10);

                manager.showDebugInfo = EditorGUILayout.Toggle("Show In-Game Debug", manager.showDebugInfo);
            }
            else
            {
                EditorGUILayout.HelpBox("No PatternLightingManager in scene. One will be created automatically at runtime.", MessageType.Info);

                if (GUILayout.Button("Create Manager Now"))
                {
                    var go = new GameObject("Pattern Lighting Manager");
                    go.AddComponent<PatternLightingManager>();
                }
            }

            EditorGUILayout.Space(20);

            EditorGUILayout.LabelField("Actions", EditorStyles.boldLabel);

            if (GUILayout.Button("Flash All Lights"))
            {
                foreach (var light in FindObjectsByType<PatternLight>(FindObjectsSortMode.None))
                {
                    light.TriggerFlash();
                }
            }

            if (GUILayout.Button("Randomize All Phases"))
            {
                foreach (var light in FindObjectsByType<PatternLight>(FindObjectsSortMode.None))
                {
                    light.settings.phaseOffset = Random.value;
                    EditorUtility.SetDirty(light);
                }
            }
        }

        private void ApplyConfig()
        {
            var manager = FindFirstObjectByType<PatternLightingManager>();
            if (manager != null)
            {
                manager.Config = _config;
                EditorUtility.SetDirty(manager);
            }
        }

        private void CreatePatternLight()
        {
            var go = new GameObject("Pattern Light");
            var light = go.AddComponent<Light>();
            light.type = LightType.Point;
            light.intensity = 1;
            light.range = 10;

            go.AddComponent<PatternLight>();

            Selection.activeGameObject = go;
            SceneView.FrameLastActiveSceneView();

            Undo.RegisterCreatedObjectUndo(go, "Create Pattern Light");
        }

        private void CreateWaterPlane()
        {
            var go = GameObject.CreatePrimitive(PrimitiveType.Plane);
            go.name = "Water Plane";
            go.transform.localScale = new Vector3(10, 1, 10);

            go.AddComponent<PatternWater>();

            // Create water material
            var shader = Shader.Find("Pattern Lighting/Water");
            if (shader != null)
            {
                var mat = new Material(shader);
                go.GetComponent<Renderer>().material = mat;
            }

            Selection.activeGameObject = go;
            SceneView.FrameLastActiveSceneView();

            Undo.RegisterCreatedObjectUndo(go, "Create Water Plane");
        }

        private void CreateMaterial(string shaderName, string materialName)
        {
            var shader = Shader.Find(shaderName);
            if (shader == null)
            {
                EditorUtility.DisplayDialog("Error", $"Shader '{shaderName}' not found!", "OK");
                return;
            }

            var material = new Material(shader);
            material.name = materialName;

            string path = EditorUtility.SaveFilePanelInProject("Save Material", materialName, "mat", "Save material as");
            if (!string.IsNullOrEmpty(path))
            {
                AssetDatabase.CreateAsset(material, path);
                AssetDatabase.SaveAssets();
                Selection.activeObject = material;
            }
        }

        private void CreatePresetMaterial(string name, Color color, LightPattern pattern)
        {
            var shader = Shader.Find("Pattern Lighting/Emissive");
            if (shader == null) return;

            var material = new Material(shader);
            material.name = $"Pattern {name}";
            material.SetColor("_EmissionColor", color * 2);
            material.SetFloat("_PatternType", (float)pattern);
            material.SetFloat("_PatternSpeed", 1f);

            string path = $"Assets/Pattern {name}.mat";
            AssetDatabase.CreateAsset(material, AssetDatabase.GenerateUniqueAssetPath(path));
            AssetDatabase.SaveAssets();
            Selection.activeObject = material;
        }
    }
}

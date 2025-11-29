// Pattern Lighting System for Unity 6
// Custom Inspector for PatternWater

using UnityEngine;
using UnityEditor;

namespace PatternLighting.Editor
{
    [CustomEditor(typeof(PatternWater))]
    public class PatternWaterEditor : UnityEditor.Editor
    {
        private SerializedProperty _settings;
        private SerializedProperty _reflectionCamera;
        private SerializedProperty _reflectionTexture;
        private SerializedProperty _updateEveryFrame;
        private SerializedProperty _reflectionDownsample;

        private bool _showWaveSettings = true;
        private bool _showAppearanceSettings = true;
        private bool _showReflectionSettings = true;
        private bool _showFoamSettings = true;
        private bool _showCausticSettings = true;
        private bool _showPerformance = true;

        private void OnEnable()
        {
            _settings = serializedObject.FindProperty("settings");
            _reflectionCamera = serializedObject.FindProperty("reflectionCamera");
            _reflectionTexture = serializedObject.FindProperty("reflectionTexture");
            _updateEveryFrame = serializedObject.FindProperty("updateEveryFrame");
            _reflectionDownsample = serializedObject.FindProperty("reflectionDownsample");
        }

        public override void OnInspectorGUI()
        {
            serializedObject.Update();

            var water = target as PatternWater;

            // Header
            EditorGUILayout.LabelField("Pattern Water", EditorStyles.boldLabel);

            EditorGUILayout.Space(5);

            // Quality
            var qualityProp = _settings.FindPropertyRelative("quality");
            EditorGUILayout.PropertyField(qualityProp);

            EditorGUILayout.Space(10);

            // Waves
            _showWaveSettings = EditorGUILayout.Foldout(_showWaveSettings, "Wave Settings", true);
            if (_showWaveSettings)
            {
                EditorGUI.indentLevel++;
                EditorGUILayout.PropertyField(_settings.FindPropertyRelative("waveHeight"));
                EditorGUILayout.PropertyField(_settings.FindPropertyRelative("waveSpeed"));
                EditorGUILayout.PropertyField(_settings.FindPropertyRelative("waveScale"));
                EditorGUI.indentLevel--;
            }

            EditorGUILayout.Space(5);

            // Appearance
            _showAppearanceSettings = EditorGUILayout.Foldout(_showAppearanceSettings, "Appearance", true);
            if (_showAppearanceSettings)
            {
                EditorGUI.indentLevel++;
                EditorGUILayout.PropertyField(_settings.FindPropertyRelative("shallowColor"));
                EditorGUILayout.PropertyField(_settings.FindPropertyRelative("deepColor"));
                EditorGUILayout.PropertyField(_settings.FindPropertyRelative("transparency"));
                EditorGUILayout.PropertyField(_settings.FindPropertyRelative("refractionStrength"));
                EditorGUI.indentLevel--;
            }

            EditorGUILayout.Space(5);

            // Reflections
            _showReflectionSettings = EditorGUILayout.Foldout(_showReflectionSettings, "Reflections", true);
            if (_showReflectionSettings)
            {
                EditorGUI.indentLevel++;
                EditorGUILayout.PropertyField(_settings.FindPropertyRelative("enableReflections"));

                if (_settings.FindPropertyRelative("enableReflections").boolValue)
                {
                    EditorGUILayout.PropertyField(_settings.FindPropertyRelative("reflectionIntensity"));
                    EditorGUILayout.PropertyField(_settings.FindPropertyRelative("fresnelPower"));

                    EditorGUILayout.Space(5);
                    EditorGUILayout.PropertyField(_reflectionCamera);
                    EditorGUILayout.PropertyField(_reflectionTexture);
                }
                EditorGUI.indentLevel--;
            }

            EditorGUILayout.Space(5);

            // Foam
            _showFoamSettings = EditorGUILayout.Foldout(_showFoamSettings, "Foam", true);
            if (_showFoamSettings)
            {
                EditorGUI.indentLevel++;
                EditorGUILayout.PropertyField(_settings.FindPropertyRelative("enableFoam"));

                if (_settings.FindPropertyRelative("enableFoam").boolValue)
                {
                    EditorGUILayout.PropertyField(_settings.FindPropertyRelative("foamAmount"));
                    EditorGUILayout.PropertyField(_settings.FindPropertyRelative("foamColor"));
                }
                EditorGUI.indentLevel--;
            }

            EditorGUILayout.Space(5);

            // Caustics
            _showCausticSettings = EditorGUILayout.Foldout(_showCausticSettings, "Caustics", true);
            if (_showCausticSettings)
            {
                EditorGUI.indentLevel++;
                EditorGUILayout.PropertyField(_settings.FindPropertyRelative("enableCaustics"));

                if (_settings.FindPropertyRelative("enableCaustics").boolValue)
                {
                    EditorGUILayout.PropertyField(_settings.FindPropertyRelative("causticIntensity"));
                    EditorGUILayout.PropertyField(_settings.FindPropertyRelative("causticSpeed"));
                }
                EditorGUI.indentLevel--;
            }

            EditorGUILayout.Space(5);

            // Performance
            _showPerformance = EditorGUILayout.Foldout(_showPerformance, "Performance", true);
            if (_showPerformance)
            {
                EditorGUI.indentLevel++;
                EditorGUILayout.PropertyField(_updateEveryFrame);
                EditorGUILayout.PropertyField(_reflectionDownsample);
                EditorGUI.indentLevel--;
            }

            EditorGUILayout.Space(10);

            // Utility buttons
            EditorGUILayout.BeginHorizontal();

            if (GUILayout.Button("Create Water Material"))
            {
                CreateWaterMaterial(water);
            }

            if (GUILayout.Button("Apply Preset: Ocean"))
            {
                ApplyOceanPreset(water);
            }

            if (GUILayout.Button("Apply Preset: Pool"))
            {
                ApplyPoolPreset(water);
            }

            EditorGUILayout.EndHorizontal();

            serializedObject.ApplyModifiedProperties();
        }

        private void CreateWaterMaterial(PatternWater water)
        {
            var shader = Shader.Find("Pattern Lighting/Water");
            if (shader == null)
            {
                EditorUtility.DisplayDialog("Error", "Water shader not found!", "OK");
                return;
            }

            var material = new Material(shader);
            material.name = "New Water Material";

            string path = EditorUtility.SaveFilePanelInProject("Save Water Material", "WaterMaterial", "mat", "");
            if (!string.IsNullOrEmpty(path))
            {
                AssetDatabase.CreateAsset(material, path);
                AssetDatabase.SaveAssets();

                var renderer = water.GetComponent<Renderer>();
                if (renderer != null)
                {
                    renderer.material = material;
                }

                Selection.activeObject = material;
            }
        }

        private void ApplyOceanPreset(PatternWater water)
        {
            Undo.RecordObject(water, "Apply Ocean Preset");

            water.settings.quality = WaterQuality.High;
            water.settings.waveHeight = 1f;
            water.settings.waveSpeed = 0.8f;
            water.settings.waveScale = 20f;
            water.settings.shallowColor = new Color(0.1f, 0.4f, 0.6f, 0.8f);
            water.settings.deepColor = new Color(0.02f, 0.1f, 0.2f, 1f);
            water.settings.transparency = 0.5f;
            water.settings.enableFoam = true;
            water.settings.foamAmount = 0.8f;
            water.settings.enableCaustics = true;

            EditorUtility.SetDirty(water);
        }

        private void ApplyPoolPreset(PatternWater water)
        {
            Undo.RecordObject(water, "Apply Pool Preset");

            water.settings.quality = WaterQuality.High;
            water.settings.waveHeight = 0.1f;
            water.settings.waveSpeed = 0.5f;
            water.settings.waveScale = 5f;
            water.settings.shallowColor = new Color(0.4f, 0.8f, 0.9f, 0.6f);
            water.settings.deepColor = new Color(0.1f, 0.3f, 0.5f, 0.9f);
            water.settings.transparency = 0.8f;
            water.settings.enableFoam = false;
            water.settings.enableCaustics = true;
            water.settings.causticIntensity = 0.7f;

            EditorUtility.SetDirty(water);
        }

        [DrawGizmo(GizmoType.Selected | GizmoType.Active)]
        private static void DrawGizmos(PatternWater water, GizmoType gizmoType)
        {
            Gizmos.color = water.settings.shallowColor;

            // Draw water bounds
            var renderer = water.GetComponent<Renderer>();
            if (renderer != null)
            {
                Gizmos.DrawWireCube(renderer.bounds.center, renderer.bounds.size);
            }

            // Draw wave direction
            Gizmos.color = Color.cyan;
            Gizmos.DrawRay(water.transform.position, water.transform.right * 2f);
        }
    }
}

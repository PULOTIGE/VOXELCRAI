// Pattern Lighting System for Unity 6
// Custom Inspector for PatternLight

using UnityEngine;
using UnityEditor;

namespace PatternLighting.Editor
{
    [CustomEditor(typeof(PatternLight))]
    [CanEditMultipleObjects]
    public class PatternLightEditor : UnityEditor.Editor
    {
        private SerializedProperty _settings;
        private SerializedProperty _baseColor;
        private SerializedProperty _baseIntensity;
        private SerializedProperty _castPatternShadows;
        private SerializedProperty _shadowSettings;
        private SerializedProperty _syncGroup;

        private bool _showPatternSettings = true;
        private bool _showShadowSettings = true;
        private bool _showPreview = true;

        private float _previewTime;
        private readonly float[] _graphValues = new float[100];
        private int _graphIndex;

        private void OnEnable()
        {
            _settings = serializedObject.FindProperty("settings");
            _baseColor = serializedObject.FindProperty("baseColor");
            _baseIntensity = serializedObject.FindProperty("baseIntensity");
            _castPatternShadows = serializedObject.FindProperty("castPatternShadows");
            _shadowSettings = serializedObject.FindProperty("shadowSettings");
            _syncGroup = serializedObject.FindProperty("syncGroup");

            EditorApplication.update += OnEditorUpdate;
        }

        private void OnDisable()
        {
            EditorApplication.update -= OnEditorUpdate;
        }

        private void OnEditorUpdate()
        {
            _previewTime += 0.016f;

            var patternLight = target as PatternLight;
            if (patternLight != null)
            {
                float value = PatternEvaluator.Evaluate(patternLight.settings.pattern, _previewTime * patternLight.settings.speed);
                value = Mathf.Lerp(patternLight.settings.minIntensity, patternLight.settings.maxIntensity, value);

                _graphValues[_graphIndex] = value;
                _graphIndex = (_graphIndex + 1) % _graphValues.Length;
            }

            Repaint();
        }

        public override void OnInspectorGUI()
        {
            serializedObject.Update();

            var patternLight = target as PatternLight;

            // Header
            EditorGUILayout.BeginHorizontal();
            EditorGUILayout.LabelField("Pattern Light", EditorStyles.boldLabel);
            GUILayout.FlexibleSpace();
            if (GUILayout.Button("Flash!", GUILayout.Width(60)))
            {
                patternLight?.TriggerFlash();
            }
            EditorGUILayout.EndHorizontal();

            EditorGUILayout.Space(5);

            // Pattern Settings
            _showPatternSettings = EditorGUILayout.Foldout(_showPatternSettings, "Pattern Settings", true);
            if (_showPatternSettings)
            {
                EditorGUI.indentLevel++;

                var patternProp = _settings.FindPropertyRelative("pattern");
                EditorGUILayout.PropertyField(patternProp);

                var speedProp = _settings.FindPropertyRelative("speed");
                EditorGUILayout.PropertyField(speedProp);

                var phaseProp = _settings.FindPropertyRelative("phaseOffset");
                EditorGUILayout.PropertyField(phaseProp);

                EditorGUILayout.Space(5);

                var minIntProp = _settings.FindPropertyRelative("minIntensity");
                var maxIntProp = _settings.FindPropertyRelative("maxIntensity");

                EditorGUILayout.LabelField("Intensity Range");
                EditorGUILayout.BeginHorizontal();
                EditorGUILayout.PropertyField(minIntProp, GUIContent.none, GUILayout.Width(60));
                EditorGUILayout.MinMaxSlider(ref minIntProp.floatValue, ref maxIntProp.floatValue, 0f, 10f);
                EditorGUILayout.PropertyField(maxIntProp, GUIContent.none, GUILayout.Width(60));
                EditorGUILayout.EndHorizontal();

                // Custom curve
                if (patternProp.enumValueIndex == (int)LightPattern.Custom)
                {
                    EditorGUILayout.Space(5);
                    var curveProp = _settings.FindPropertyRelative("customCurve");
                    EditorGUILayout.PropertyField(curveProp);
                }

                // Color shift
                EditorGUILayout.Space(5);
                var colorShiftProp = _settings.FindPropertyRelative("enableColorShift");
                EditorGUILayout.PropertyField(colorShiftProp);

                if (colorShiftProp.boolValue)
                {
                    var gradientProp = _settings.FindPropertyRelative("colorGradient");
                    EditorGUILayout.PropertyField(gradientProp);
                }

                EditorGUI.indentLevel--;
            }

            EditorGUILayout.Space(10);

            // Base Properties
            EditorGUILayout.LabelField("Base Properties", EditorStyles.boldLabel);
            EditorGUILayout.PropertyField(_baseColor);
            EditorGUILayout.PropertyField(_baseIntensity);

            EditorGUILayout.Space(10);

            // Shadow Settings
            _showShadowSettings = EditorGUILayout.Foldout(_showShadowSettings, "Shadow Settings", true);
            if (_showShadowSettings)
            {
                EditorGUI.indentLevel++;
                EditorGUILayout.PropertyField(_castPatternShadows);

                if (_castPatternShadows.boolValue)
                {
                    EditorGUILayout.PropertyField(_shadowSettings, true);
                }
                EditorGUI.indentLevel--;
            }

            EditorGUILayout.Space(10);

            // Sync
            EditorGUILayout.LabelField("Synchronization", EditorStyles.boldLabel);
            EditorGUILayout.PropertyField(_syncGroup);

            EditorGUILayout.Space(10);

            // Preview
            _showPreview = EditorGUILayout.Foldout(_showPreview, "Live Preview", true);
            if (_showPreview)
            {
                // Graph
                Rect graphRect = GUILayoutUtility.GetRect(GUIContent.none, GUIStyle.none, GUILayout.Height(60));
                EditorGUI.DrawRect(graphRect, new Color(0.15f, 0.15f, 0.15f));

                // Draw grid
                Handles.color = new Color(0.3f, 0.3f, 0.3f);
                Handles.DrawLine(new Vector3(graphRect.x, graphRect.center.y), new Vector3(graphRect.xMax, graphRect.center.y));

                // Draw graph
                Handles.color = patternLight != null ? patternLight.baseColor : Color.cyan;
                for (int i = 1; i < _graphValues.Length; i++)
                {
                    int idx1 = (_graphIndex + i - 1) % _graphValues.Length;
                    int idx2 = (_graphIndex + i) % _graphValues.Length;

                    float x1 = graphRect.x + (i - 1) * graphRect.width / _graphValues.Length;
                    float y1 = graphRect.yMax - _graphValues[idx1] * graphRect.height;
                    float x2 = graphRect.x + i * graphRect.width / _graphValues.Length;
                    float y2 = graphRect.yMax - _graphValues[idx2] * graphRect.height;

                    Handles.DrawLine(new Vector3(x1, y1), new Vector3(x2, y2));
                }

                // Current value
                if (patternLight != null)
                {
                    EditorGUILayout.BeginHorizontal();
                    EditorGUILayout.LabelField($"Current: {patternLight.CurrentIntensity:F2}");

                    // Color preview
                    Rect colorRect = GUILayoutUtility.GetRect(40, 20);
                    EditorGUI.DrawRect(colorRect, patternLight.CurrentColor);
                    EditorGUILayout.EndHorizontal();
                }
            }

            serializedObject.ApplyModifiedProperties();
        }

        [DrawGizmo(GizmoType.Selected | GizmoType.Active)]
        private static void DrawGizmos(PatternLight light, GizmoType gizmoType)
        {
            if (light.Light == null) return;

            Gizmos.color = light.CurrentColor * light.CurrentIntensity;

            switch (light.Light.type)
            {
                case LightType.Point:
                    Gizmos.DrawWireSphere(light.transform.position, light.Light.range * 0.5f);
                    break;

                case LightType.Spot:
                    // Draw spot cone direction
                    Gizmos.DrawRay(light.transform.position, light.transform.forward * light.Light.range);
                    break;
            }
        }
    }
}

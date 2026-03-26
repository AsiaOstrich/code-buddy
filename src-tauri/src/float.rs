use serde::{Deserialize, Serialize};

/// 浮動視窗設定（持久化到 app_data_dir）
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloatSettings {
    pub enabled: bool,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub opacity: f64,
}

impl Default for FloatSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            x: None,
            y: None,
            opacity: 0.7,
        }
    }
}

#[allow(dead_code)]
impl FloatSettings {
    /// 設定透明度，clamp 到 0.3~1.0
    pub fn set_opacity(&mut self, value: f64) {
        self.opacity = value.clamp(0.3, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === AC-5: 透明度 ===

    #[test]
    fn default_opacity_is_0_7() {
        let settings = FloatSettings::default();
        assert!((settings.opacity - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn opacity_clamp_min_0_3() {
        let mut settings = FloatSettings::default();
        settings.set_opacity(0.1);
        assert!((settings.opacity - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn opacity_clamp_max_1_0() {
        let mut settings = FloatSettings::default();
        settings.set_opacity(1.5);
        assert!((settings.opacity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn opacity_normal_range() {
        let mut settings = FloatSettings::default();
        settings.set_opacity(0.5);
        assert!((settings.opacity - 0.5).abs() < f64::EPSILON);
    }

    // === AC-6: 預設位置 ===

    #[test]
    fn default_position_is_none() {
        let settings = FloatSettings::default();
        assert!(settings.x.is_none());
        assert!(settings.y.is_none());
    }

    // === AC-7: 記住位置 ===

    #[test]
    fn serialize_and_deserialize() {
        let mut settings = FloatSettings::default();
        settings.x = Some(1820.0);
        settings.y = Some(980.0);
        settings.opacity = 0.5;
        settings.enabled = true;

        let json = serde_json::to_string(&settings).unwrap();
        let loaded: FloatSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.x, Some(1820.0));
        assert_eq!(loaded.y, Some(980.0));
        assert!((loaded.opacity - 0.5).abs() < f64::EPSILON);
        assert!(loaded.enabled);
    }

    // === AC-1: 啟用/關閉 ===

    #[test]
    fn default_disabled() {
        let settings = FloatSettings::default();
        assert!(!settings.enabled);
    }
}

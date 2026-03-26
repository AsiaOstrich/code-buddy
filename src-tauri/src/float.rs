use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

/// 浮動視窗設定（持久化到 app_data_dir）
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

impl FloatSettings {
    /// 設定透明度，clamp 到 0.3~1.0
    pub fn set_opacity(&mut self, value: f64) {
        self.opacity = value.clamp(0.3, 1.0);
    }
}

const FLOAT_WINDOW_LABEL: &str = "float";
const FLOAT_SIZE: f64 = 86.0; // 80px 動畫 + 6px 邊框
const EDGE_MARGIN: f64 = 20.0;

/// 切換浮動視窗顯示/隱藏
pub fn toggle_float_window(app: &AppHandle) -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window(FLOAT_WINDOW_LABEL) {
        // 已存在 → 關閉
        window.close()?;
        tracing::info!("浮動視窗已關閉");
        Ok(false)
    } else {
        // 不存在 → 建立
        create_float_window(app)?;
        tracing::info!("浮動視窗已開啟");
        Ok(true)
    }
}

/// 建立浮動視窗
fn create_float_window(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 讀取設定（位置、透明度）
    let settings = load_settings(app);

    // 計算位置：有儲存位置就用，沒有就預設右下角
    let (x, y) = match (settings.x, settings.y) {
        (Some(x), Some(y)) => (x, y),
        _ => default_bottom_right_position(app),
    };

    let window = WebviewWindowBuilder::new(
        app,
        FLOAT_WINDOW_LABEL,
        WebviewUrl::App("float.html".into()),
    )
    .title("")
    .inner_size(FLOAT_SIZE, FLOAT_SIZE)
    .position(x, y)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .build()?;

    // macOS: 讓視窗在所有工作區可見
    #[cfg(target_os = "macos")]
    {
        let _ = window.set_visible_on_all_workspaces(true);
    }

    let _ = window;
    Ok(())
}

/// 計算螢幕右下角位置
fn default_bottom_right_position(app: &AppHandle) -> (f64, f64) {
    // 嘗試取得主螢幕尺寸
    if let Ok(Some(monitor)) = app
        .get_webview_window("devpanel")
        .or_else(|| app.get_webview_window("main"))
        .map(|w| w.current_monitor())
        .unwrap_or(Ok(None))
    {
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let screen_w = size.width as f64 / scale;
        let screen_h = size.height as f64 / scale;
        (
            screen_w - FLOAT_SIZE - EDGE_MARGIN,
            screen_h - FLOAT_SIZE - EDGE_MARGIN - 80.0, // 減去 Dock 高度
        )
    } else {
        // 無法取得螢幕尺寸時的預設值
        (1800.0, 900.0)
    }
}

/// 設定浮動視窗透明度（儲存設定，實際 opacity 由前端 CSS 控制）
pub fn set_opacity(app: &AppHandle, opacity: f64) -> Result<(), Box<dyn std::error::Error>> {
    let mut settings = load_settings(app);
    settings.set_opacity(opacity);
    save_settings(app, &settings);
    Ok(())
}

/// 儲存浮動視窗位置
pub fn save_position(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window(FLOAT_WINDOW_LABEL) {
        let pos = window.outer_position()?;
        let scale = window.scale_factor()?;
        let mut settings = load_settings(app);
        settings.x = Some(pos.x as f64 / scale);
        settings.y = Some(pos.y as f64 / scale);
        save_settings(app, &settings);
    }
    Ok(())
}

// === 設定持久化 ===

fn settings_path(app: &AppHandle) -> std::path::PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("float-settings.json")
}

fn load_settings(app: &AppHandle) -> FloatSettings {
    let path = settings_path(app);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_settings(app: &AppHandle, settings: &FloatSettings) {
    let path = settings_path(app);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(&path, json);
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

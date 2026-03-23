pub struct ColorUtils;

impl ColorUtils {
    pub fn parse_hex(hex: &str) -> (u8, u8, u8) {
        let hex = hex.trim().trim_start_matches('#');
        if hex.len() < 6 {
            return (0, 0, 0);
        }
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        (r, g, b)
    }

    pub fn to_hex(r: u8, g: u8, b: u8) -> String {
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    pub fn lerp_rgb(from: (u8, u8, u8), to: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
        let t = t.clamp(0.0, 1.0);
        let r = (from.0 as f32 + (to.0 as f32 - from.0 as f32) * t).round() as u8;
        let g = (from.1 as f32 + (to.1 as f32 - from.1 as f32) * t).round() as u8;
        let b = (from.2 as f32 + (to.2 as f32 - from.2 as f32) * t).round() as u8;
        (r, g, b)
    }

    pub fn parse_rgba_opacity(rgba: &str) -> f32 {
        let rgba = rgba.trim();
        if let Some(start) = rgba.rfind(',') {
            if let Some(end) = rgba.rfind(')') {
                if start + 1 < end {
                    return rgba[start + 1..end].trim().parse().unwrap_or(1.0);
                }
            }
        }
        1.0
    }

    pub fn hex_to_rgba(hex: &str, alpha: f32) -> String {
        let (r, g, b) = Self::parse_hex(hex);
        format!("rgba({}, {}, {}, {:.2})", r, g, b, alpha.clamp(0.0, 1.0))
    }

    pub fn rgb_to_string(r: u8, g: u8, b: u8) -> String {
        format!("rgb({}, {}, {})", r, g, b)
    }
}

pub fn prefers_reduced_motion() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_valid() {
        assert_eq!(ColorUtils::parse_hex("#ff0000"), (255, 0, 0));
        assert_eq!(ColorUtils::parse_hex("#00ff00"), (0, 255, 0));
        assert_eq!(ColorUtils::parse_hex("#0000ff"), (0, 0, 255));
        assert_eq!(ColorUtils::parse_hex("30d158"), (48, 209, 88));
    }

    #[test]
    fn test_parse_hex_invalid() {
        assert_eq!(ColorUtils::parse_hex(""), (0, 0, 0));
        assert_eq!(ColorUtils::parse_hex("#"), (0, 0, 0));
        assert_eq!(ColorUtils::parse_hex("#ff"), (0, 0, 0));
    }

    #[test]
    fn test_to_hex() {
        assert_eq!(ColorUtils::to_hex(255, 0, 0), "#ff0000");
        assert_eq!(ColorUtils::to_hex(0, 255, 0), "#00ff00");
        assert_eq!(ColorUtils::to_hex(48, 209, 88), "#30d158");
    }

    #[test]
    fn test_lerp_rgb_boundaries() {
        assert_eq!(
            ColorUtils::lerp_rgb((0, 0, 0), (255, 255, 255), 0.0),
            (0, 0, 0)
        );
        assert_eq!(
            ColorUtils::lerp_rgb((0, 0, 0), (255, 255, 255), 1.0),
            (255, 255, 255)
        );
    }

    #[test]
    fn test_lerp_rgb_midpoint() {
        let result = ColorUtils::lerp_rgb((0, 0, 0), (100, 100, 100), 0.5);
        assert_eq!(result, (50, 50, 50));
    }

    #[test]
    fn test_lerp_rgb_clamp() {
        assert_eq!(
            ColorUtils::lerp_rgb((0, 0, 0), (100, 100, 100), -0.5),
            (0, 0, 0)
        );
        assert_eq!(
            ColorUtils::lerp_rgb((0, 0, 0), (100, 100, 100), 1.5),
            (100, 100, 100)
        );
    }

    #[test]
    fn test_parse_rgba_opacity() {
        assert_eq!(ColorUtils::parse_rgba_opacity("rgba(0, 0, 0, 0.7)"), 0.7);
        assert_eq!(
            ColorUtils::parse_rgba_opacity("rgba(255, 255, 255, 0.54)"),
            0.54
        );
        assert_eq!(ColorUtils::parse_rgba_opacity("rgba(0,0,0,1)"), 1.0);
    }

    #[test]
    fn test_hex_to_rgba_format() {
        let result = ColorUtils::hex_to_rgba("#ff0000", 0.5);
        assert_eq!(result, "rgba(255, 0, 0, 0.50)");
    }

    #[test]
    fn test_rgb_to_string() {
        assert_eq!(ColorUtils::rgb_to_string(255, 128, 0), "rgb(255, 128, 0)");
    }
}

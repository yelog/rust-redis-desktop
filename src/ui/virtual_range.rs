pub fn visible_range(
    total: usize,
    scroll_top: f64,
    viewport_height: f64,
    row_height: f64,
    overscan: usize,
) -> (usize, usize) {
    if total == 0 || row_height <= 0.0 {
        return (0, 0);
    }
    let first = (scroll_top.max(0.0) / row_height) as usize;
    let visible = (viewport_height.max(0.0) / row_height).ceil() as usize;
    let start = first.saturating_sub(overscan).min(total);
    let end = first
        .saturating_add(visible)
        .saturating_add(overscan)
        .min(total);
    (start, end.max(start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_visible_rows() {
        assert_eq!(visible_range(0, 0.0, 100.0, 20.0, 2), (0, 0));
        assert_eq!(visible_range(100, 200.0, 100.0, 20.0, 2), (8, 17));
        assert_eq!(visible_range(10, 500.0, 100.0, 20.0, 2), (10, 10));
    }
}

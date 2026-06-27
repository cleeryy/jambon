//! Consistent embed colours for all bot responses.

/// Informational embeds — blue.
pub const COLOR_INFO: u32 = 0x00aaff;
/// Success embeds — green.
pub const COLOR_SUCCESS: u32 = 0x00ff00;
/// Warning embeds — yellow/amber.
pub const COLOR_WARNING: u32 = 0xffaa00;
/// Error embeds — red.
pub const COLOR_ERROR: u32 = 0xff0000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colors_have_expected_values() {
        assert_eq!(COLOR_INFO, 0x00aaff);
        assert_eq!(COLOR_SUCCESS, 0x00ff00);
        assert_eq!(COLOR_WARNING, 0xffaa00);
        assert_eq!(COLOR_ERROR, 0xff0000);
    }

    #[test]
    fn test_colors_are_distinct() {
        let mut colors = std::collections::HashSet::new();
        colors.insert(COLOR_INFO);
        colors.insert(COLOR_SUCCESS);
        colors.insert(COLOR_WARNING);
        colors.insert(COLOR_ERROR);
        assert_eq!(colors.len(), 4, "all colors should be distinct");
    }
}

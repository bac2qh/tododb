use ratatui::style::Color;

/// Catppuccin Frappe color palette
/// A warm, cozy color scheme perfect for terminal applications
pub struct CatppuccinFrappe;

impl CatppuccinFrappe {
    // Base colors
    pub const BASE: Color = Color::Rgb(48, 52, 70);      // #303446

    // Text colors
    pub const TEXT: Color = Color::Rgb(198, 208, 245);   // #c6d0f5
    pub const SUBTEXT1: Color = Color::Rgb(181, 191, 226); // #b5bfe2
    pub const SUBTEXT0: Color = Color::Rgb(165, 173, 203); // #a5adcb

    // Surface colors
    pub const SURFACE2: Color = Color::Rgb(87, 96, 134); // #575e86
    pub const SURFACE0: Color = Color::Rgb(54, 58, 79);  // #363a4f

    // Accent colors
    pub const LAVENDER: Color = Color::Rgb(186, 187, 241); // #babbf1
    pub const BLUE: Color = Color::Rgb(140, 170, 238);     // #8caaee
    pub const SAPPHIRE: Color = Color::Rgb(133, 193, 220); // #85c1dc
    pub const TEAL: Color = Color::Rgb(129, 200, 190);     // #81c8be
    pub const GREEN: Color = Color::Rgb(166, 209, 137);    // #a6d189
    pub const YELLOW: Color = Color::Rgb(229, 200, 144);   // #e5c890
    pub const PEACH: Color = Color::Rgb(239, 159, 118);    // #ef9f76
    pub const RED: Color = Color::Rgb(231, 130, 132);      // #e78284
    pub const MAUVE: Color = Color::Rgb(202, 158, 230);    // #ca9ee6
    pub const PINK: Color = Color::Rgb(244, 184, 228);     // #f4b8e4

    // UI-specific colors
    pub const SELECTED: Color = Self::BLUE;
    pub const SELECTED_BG: Color = Color::Rgb(65, 72, 104); // #414968
    pub const BORDER: Color = Self::SURFACE2;
    pub const COMPLETED: Color = Self::GREEN;
    pub const INCOMPLETE: Color = Self::TEXT;
    pub const PARENT_INDICATOR: Color = Self::LAVENDER;
    pub const CREATION_TIME: Color = Self::SUBTEXT0;
    pub const ERROR: Color = Self::RED;
}
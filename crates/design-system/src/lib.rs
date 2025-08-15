//! Design System for Hodei Momentum
//! 
//! Provides design tokens, theme management, and consistent styling API
//! following the separation of concerns principle in hexagonal architecture.

use serde::{Deserialize, Serialize};

// Public modules
pub mod icons;
pub mod toolbar;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::System
    }
}

impl Theme {
    pub fn css_class(&self) -> &'static str {
        match self {
            Theme::Light => "theme-light",
            Theme::Dark => "theme-dark", 
            Theme::System => "theme-system",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Theme::Light => "Claro",
            Theme::Dark => "Oscuro",
            Theme::System => "Sistema",
        }
    }
}

/// Design tokens for the Hodei Momentum application
pub mod tokens {
    /// Color palette following design system principles
    pub mod colors {
        // Primary palette (brand colors)
        pub const PRIMARY_50: &str = "rgb(239 246 255)";   // Lightest blue
        pub const PRIMARY_500: &str = "rgb(59 130 246)";   // Main blue
        pub const PRIMARY_600: &str = "rgb(37 99 235)";    // Darker blue
        pub const PRIMARY_900: &str = "rgb(30 58 138)";    // Darkest blue

        // Neutral palette (grays)
        pub const NEUTRAL_50: &str = "rgb(248 250 252)";   // Almost white
        pub const NEUTRAL_100: &str = "rgb(241 245 249)";  // Very light gray
        pub const NEUTRAL_200: &str = "rgb(226 232 240)";  // Light gray
        pub const NEUTRAL_300: &str = "rgb(203 213 225)";  // Medium light gray
        pub const NEUTRAL_400: &str = "rgb(148 163 184)";  // Medium gray
        pub const NEUTRAL_500: &str = "rgb(100 116 139)";  // True gray
        pub const NEUTRAL_600: &str = "rgb(71 85 105)";    // Medium dark gray
        pub const NEUTRAL_700: &str = "rgb(51 65 85)";     // Dark gray
        pub const NEUTRAL_800: &str = "rgb(30 41 59)";     // Very dark gray
        pub const NEUTRAL_900: &str = "rgb(15 23 42)";     // Almost black

        // Semantic colors
        pub const SUCCESS: &str = "rgb(34 197 94)";        // Green
        pub const WARNING: &str = "rgb(245 158 11)";       // Orange  
        pub const ERROR: &str = "rgb(239 68 68)";          // Red
        pub const INFO: &str = "rgb(59 130 246)";          // Blue
    }

    /// Spacing scale following 8px grid system
    pub mod spacing {
        pub const XS: &str = "0.25rem";    // 4px
        pub const SM: &str = "0.5rem";     // 8px  
        pub const MD: &str = "0.75rem";    // 12px
        pub const LG: &str = "1rem";       // 16px
        pub const XL: &str = "1.5rem";     // 24px
        pub const XXL: &str = "2rem";      // 32px
        pub const XXXL: &str = "3rem";     // 48px
    }

    /// Border radius values
    pub mod border_radius {
        pub const SM: &str = "0.25rem";    // 4px
        pub const MD: &str = "0.375rem";   // 6px
        pub const LG: &str = "0.5rem";     // 8px
        pub const XL: &str = "0.75rem";    // 12px
        pub const FULL: &str = "9999px";   // Fully rounded
    }

    /// Shadow definitions
    pub mod shadows {
        pub const SM: &str = "0 1px 2px 0 rgb(0 0 0 / 0.05)";
        pub const MD: &str = "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)";
        pub const LG: &str = "0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)";
        pub const XL: &str = "0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)";
    }
}

/// Utility functions for theme-aware styling
pub mod utils {
    use super::Theme;

    /// Get appropriate text color for given background
    pub fn text_color_for_theme(theme: Theme) -> &'static str {
        match theme {
            Theme::Light => "text-neutral-900",
            Theme::Dark => "text-neutral-100", 
            Theme::System => "text-neutral-900 dark:text-neutral-100",
        }
    }

    /// Get appropriate background color for given theme
    pub fn bg_color_for_theme(theme: Theme) -> &'static str {
        match theme {
            Theme::Light => "bg-white",
            Theme::Dark => "bg-neutral-900",
            Theme::System => "bg-white dark:bg-neutral-900",
        }
    }

    /// Get appropriate border color for given theme
    pub fn border_color_for_theme(theme: Theme) -> &'static str {
        match theme {
            Theme::Light => "border-neutral-200",
            Theme::Dark => "border-neutral-700",
            Theme::System => "border-neutral-200 dark:border-neutral-700",
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod theme_provider {
    use super::Theme;
    use leptos::prelude::*;
    use web_sys::window;

    const STORAGE_KEY: &str = "hodei_theme";

    #[derive(Clone)]
    pub struct ThemeProvider {
        theme: RwSignal<Theme>,
        system_theme: RwSignal<Theme>,
    }

    impl ThemeProvider {
        pub fn new() -> Self {
            let stored_theme = get_stored_theme().unwrap_or(Theme::System);
            let system_theme = detect_system_theme();
            
            let provider = Self {
                theme: RwSignal::new(stored_theme),
                system_theme: RwSignal::new(system_theme),
            };

            // Apply initial theme
            provider.apply_theme();
            
            // Setup system theme detection
            provider.setup_system_theme_listener();
            
            provider
        }

        pub fn current_theme(&self) -> ReadSignal<Theme> {
            self.theme.read_only()
        }

        pub fn effective_theme(&self) -> Theme {
            match self.theme.get() {
                Theme::System => self.system_theme.get(),
                theme => theme,
            }
        }

        pub fn set_theme(&self, theme: Theme) {
            self.theme.set(theme);
            store_theme(theme);
            self.apply_theme();
        }

        fn apply_theme(&self) {
            if let Some(document) = window().and_then(|w| w.document()) {
                if let Some(html) = document.document_element() {
                    // Remover todas las clases de tema previas
                    let _ = html.class_list().remove_1("theme-light");
                    let _ = html.class_list().remove_1("theme-dark");
                    let _ = html.class_list().remove_1("theme-system");
                    
                    // Agregar la clase base si no existe
                    let _ = html.class_list().add_1("hodei-app");
                    
                    // Agregar la nueva clase de tema
                    let _ = html.class_list().add_1(self.effective_theme().css_class());
                }
            }
        }

        fn setup_system_theme_listener(&self) {
            // Setup media query listener for system theme changes
            if let Some(window) = window() {
                let system_theme = self.system_theme;
                let theme = self.theme;
                
                // This is a simplified version - in a real implementation,
                // you'd set up a proper media query listener
                Effect::new(move |_| {
                    let new_system_theme = detect_system_theme();
                    if new_system_theme != system_theme.get() {
                        system_theme.set(new_system_theme);
                        // Re-apply theme if currently using system
                        if theme.get() == Theme::System {
                            if let Some(document) = window.document() {
                                if let Some(html) = document.document_element() {
                                    // Remover todas las clases de tema previas
                                    let _ = html.class_list().remove_1("theme-light");
                                    let _ = html.class_list().remove_1("theme-dark");
                                    let _ = html.class_list().remove_1("theme-system");
                                    
                                    // Agregar la clase base si no existe
                                    let _ = html.class_list().add_1("hodei-app");
                                    
                                    // Agregar la nueva clase de tema
                                    let _ = html.class_list().add_1(new_system_theme.css_class());
                                }
                            }
                        }
                    }
                });
            }
        }
    }

    fn get_stored_theme() -> Option<Theme> {
        window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|storage| storage.get_item(STORAGE_KEY).ok().flatten())
            .and_then(|theme_str| match theme_str.as_str() {
                "light" => Some(Theme::Light),
                "dark" => Some(Theme::Dark),
                "system" => Some(Theme::System),
                _ => None,
            })
    }

    fn store_theme(theme: Theme) {
        if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
            let theme_str = match theme {
                Theme::Light => "light",
                Theme::Dark => "dark", 
                Theme::System => "system",
            };
            let _ = storage.set_item(STORAGE_KEY, theme_str);
        }
    }

    fn detect_system_theme() -> Theme {
        window()
            .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
            .map(|media| if media.matches() { Theme::Dark } else { Theme::Light })
            .unwrap_or(Theme::Light)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod theme_provider {
    use super::Theme;

    pub struct ThemeProvider;

    impl ThemeProvider {
        pub fn new() -> Self {
            Self
        }
    }
}
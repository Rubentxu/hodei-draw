//! Toolbar components for Hodei Momentum Design System
//! Excalidraw-style toolbar implementation

use leptos::*;
use leptos::prelude::*;
use crate::icons::{Icon, IconType, IconSize};

/// Floating toolbar component similar to Excalidraw
#[component]
pub fn FloatingToolbar(children: Children) -> impl IntoView {
    view! {
        <div class="floating-toolbar">
            {children()}
        </div>
    }
}

/// Toolbar button with icon and tooltip
#[component]
pub fn ToolbarButton(
    #[prop(into)] icon: IconType,
    #[prop(default = String::new(), into)] tooltip: String,
    #[prop(optional)] selected: Option<Box<dyn Fn() -> bool + Send + 'static>>,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_click: Option<Box<dyn Fn() + 'static>>,
) -> impl IntoView {
    let button_class = move || {
        let mut classes = vec!["toolbar-button"];
        if let Some(ref sel_fn) = selected {
            if sel_fn() {
                classes.push("selected");
            }
        }
        classes.join(" ")
    };
    
    view! {
        <button 
            class=button_class
            disabled=disabled
            title=tooltip.clone()
            on:click=move |_| {
                if let Some(ref handler) = on_click {
                    handler();
                }
            }
        >
            <Icon icon=icon size=IconSize::Medium />
        </button>
    }
}

/// Toolbar separator line
#[component]
pub fn ToolbarSeparator() -> impl IntoView {
    view! {
        <div class="toolbar-separator"></div>
    }
}

/// Toolbar group for organizing related tools
#[component]
pub fn ToolbarGroup(children: Children) -> impl IntoView {
    view! {
        <div class="toolbar-group">
            {children()}
        </div>
    }
}

/// Sidebar for file operations and settings
#[component]
pub fn Sidebar(children: Children) -> impl IntoView {
    view! {
        <div class="sidebar">
            {children()}
        </div>
    }
}

/// Sidebar button component
#[component]
pub fn SidebarButton(
    #[prop(into)] icon: IconType,
    #[prop(default = String::new(), into)] tooltip: String,
    #[prop(optional)] on_click: Option<Box<dyn Fn() + 'static>>,
) -> impl IntoView {
    view! {
        <button 
            class="sidebar-button"
            title=tooltip.clone()
            on:click=move |_| {
                if let Some(ref handler) = on_click {
                    handler();
                }
            }
        >
            <Icon icon=icon size=IconSize::Large />
        </button>
    }
}

/// Floating island panel for properties and settings
#[component]
pub fn IslandPanel(
    #[prop(optional)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let panel_class = format!("island-panel {}", class.unwrap_or_default());
    
    view! {
        <div class=panel_class>
            {children()}
        </div>
    }
}

/// Property panel that appears at the bottom center
#[component]
pub fn PropertyPanel(
    #[prop(default = false)] visible: bool,
    children: Children,
) -> impl IntoView {
    let panel_class = move || {
        if visible {
            "island-panel property-panel"
        } else {
            "island-panel property-panel hidden"
        }
    };
    
    view! {
        <div class=panel_class>
            {children()}
        </div>
    }
}

/// Icon button with customizable styling
#[component]
pub fn IconButton(
    #[prop(into)] icon: IconType,
    #[prop(default = IconSize::Medium)] size: IconSize,
    #[prop(optional)] class: Option<String>,
    #[prop(default = String::new(), into)] tooltip: String,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] on_click: Option<Box<dyn Fn() + 'static>>,
) -> impl IntoView {
    let button_class = class.unwrap_or_default();
    
    view! {
        <button 
            class=button_class
            disabled=disabled
            title=tooltip.clone()
            on:click=move |_| {
                if let Some(ref handler) = on_click {
                    handler();
                }
            }
        >
            <Icon icon=icon size=size />
        </button>
    }
}

/// Theme toggle button
#[component]
pub fn ThemeToggle<F>(
    #[prop(into)] current_theme: Signal<crate::Theme>,
    on_theme_change: F,
) -> impl IntoView 
where
    F: Fn(crate::Theme) + 'static,
{
    let icon = move || match current_theme.get() {
        crate::Theme::Light => IconType::Sun,
        crate::Theme::Dark => IconType::Moon,
        crate::Theme::System => IconType::System,
    };
    
    let tooltip = move || match current_theme.get() {
        crate::Theme::Light => "Cambiar a tema oscuro",
        crate::Theme::Dark => "Cambiar a tema del sistema",
        crate::Theme::System => "Cambiar a tema claro",
    };
    
    view! {
        <ToolbarButton
            icon=icon()
            tooltip=tooltip().to_string()
            on_click=Box::new(move || {
                let new_theme = match current_theme.get() {
                    crate::Theme::Light => crate::Theme::Dark,
                    crate::Theme::Dark => crate::Theme::System,
                    crate::Theme::System => crate::Theme::Light,
                };
                on_theme_change(new_theme);
            })
        />
    }
}
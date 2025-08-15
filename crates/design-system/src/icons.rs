//! Sistema de íconos SVG para Hodei Momentum Design System
//! Basado en el estilo visual de Excalidraw

use leptos::*;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IconSize {
    Small,  // 16px
    Medium, // 20px
    Large,  // 24px
}

impl IconSize {
    pub fn to_pixels(&self) -> u32 {
        match self {
            IconSize::Small => 16,
            IconSize::Medium => 20,
            IconSize::Large => 24,
        }
    }
    
    pub fn to_class(&self) -> &'static str {
        match self {
            IconSize::Small => "w-4 h-4",
            IconSize::Medium => "w-5 h-5", 
            IconSize::Large => "w-6 h-6",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IconType {
    // Tools
    Select,
    Rectangle,
    Ellipse,
    Arrow,
    Line,
    Text,
    Pen,
    Eraser,
    
    // UI
    Menu,
    Settings,
    Download,
    Upload,
    Share,
    Undo,
    Redo,
    Zoom,
    Hand,
    
    // File operations
    New,
    Open,
    Save,
    Export,
    
    // Theme
    Sun,
    Moon,
    System,
}

#[component]
pub fn Icon(
    #[prop(into)] icon: IconType,
    #[prop(default = IconSize::Medium)] size: IconSize,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let size_px = size.to_pixels();
    let size_class = size.to_class();
    
    let additional_class = class.unwrap_or_default();
    let full_class = format!("{} {} currentColor", size_class, additional_class);
    
    view! {
        <svg 
            class=full_class
            width=size_px
            height=size_px
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            {match icon {
                IconType::Select => view! {
                    <path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z"/>
                }.into_any(),
                
                IconType::Rectangle => view! {
                    <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
                }.into_any(),
                
                IconType::Ellipse => view! {
                    <circle cx="12" cy="12" r="9"/>
                }.into_any(),
                
                IconType::Arrow => view! {
                    <line x1="7" y1="17" x2="17" y2="7"/>
                    <polyline points="7,7 17,7 17,17"/>
                }.into_any(),
                
                IconType::Line => view! {
                    <line x1="7" y1="17" x2="17" y2="7"/>
                }.into_any(),
                
                IconType::Text => view! {
                    <polyline points="4,7 4,4 20,4 20,7"/>
                    <line x1="9" y1="20" x2="15" y2="20"/>
                    <line x1="12" y1="4" x2="12" y2="20"/>
                }.into_any(),
                
                IconType::Pen => view! {
                    <path d="M12 19l7-7 3 3-7 7-3-3z"/>
                    <path d="M18 13l-1.5-7.5L2 2l3.5 14.5L13 18l5-5z"/>
                    <path d="M2 2l7.586 7.586"/>
                    <circle cx="11" cy="11" r="2"/>
                }.into_any(),
                
                IconType::Eraser => view! {
                    <path d="M20 20H7l-3-3 9-9 3 3 4 4v5z"/>
                    <path d="M6 6l4 4"/>
                }.into_any(),
                
                IconType::Menu => view! {
                    <line x1="3" y1="6" x2="21" y2="6"/>
                    <line x1="3" y1="12" x2="21" y2="12"/>
                    <line x1="3" y1="18" x2="21" y2="18"/>
                }.into_any(),
                
                IconType::Settings => view! {
                    <circle cx="12" cy="12" r="3"/>
                    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1 1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
                }.into_any(),
                
                IconType::Download => view! {
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                    <polyline points="7,10 12,15 17,10"/>
                    <line x1="12" y1="15" x2="12" y2="3"/>
                }.into_any(),
                
                IconType::Upload => view! {
                    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/>
                    <polyline points="17,8 12,3 7,8"/>
                    <line x1="12" y1="3" x2="12" y2="15"/>
                }.into_any(),
                
                IconType::Share => view! {
                    <circle cx="18" cy="5" r="3"/>
                    <circle cx="6" cy="12" r="3"/>
                    <circle cx="18" cy="19" r="3"/>
                    <line x1="8.59" y1="13.51" x2="15.42" y2="17.49"/>
                    <line x1="15.41" y1="6.51" x2="8.59" y2="10.49"/>
                }.into_any(),
                
                IconType::Undo => view! {
                    <polyline points="1,4 1,10 7,10"/>
                    <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/>
                }.into_any(),
                
                IconType::Redo => view! {
                    <polyline points="23,4 23,10 17,10"/>
                    <path d="M20.49 15a9 9 0 1 1-2.13-9.36L23 10"/>
                }.into_any(),
                
                IconType::Zoom => view! {
                    <circle cx="11" cy="11" r="8"/>
                    <path d="M21 21l-4.35-4.35"/>
                }.into_any(),
                
                IconType::Hand => view! {
                    <path d="M18 11V6a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v0"/>
                    <path d="M14 10V4a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v2"/>
                    <path d="M10 10.5V6a2 2 0 0 0-2-2v0a2 2 0 0 0-2 2v8"/>
                    <path d="M18 8a2 2 0 1 1 4 0v6a8 8 0 0 1-8 8h-2c-2.8 0-4.5-.86-5.99-2.34l-3.6-3.6a2 2 0 0 1 2.83-2.82L7 15"/>
                }.into_any(),
                
                IconType::New => view! {
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                    <polyline points="14,2 14,8 20,8"/>
                    <line x1="12" y1="18" x2="12" y2="12"/>
                    <line x1="9" y1="15" x2="15" y2="15"/>
                }.into_any(),
                
                IconType::Open => view! {
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                    <polyline points="14,2 14,8 20,8"/>
                    <line x1="16" y1="13" x2="8" y2="13"/>
                    <line x1="16" y1="17" x2="8" y2="17"/>
                    <polyline points="10,9 9,9 8,9"/>
                }.into_any(),
                
                IconType::Save => view! {
                    <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
                    <polyline points="17,21 17,13 7,13 7,21"/>
                    <polyline points="7,3 7,8 15,8"/>
                }.into_any(),
                
                IconType::Export => view! {
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                    <polyline points="14,2 14,8 20,8"/>
                    <line x1="16" y1="13" x2="8" y2="13"/>
                    <line x1="16" y1="17" x2="8" y2="17"/>
                    <polyline points="10,9 9,9 8,9"/>
                }.into_any(),
                
                IconType::Sun => view! {
                    <circle cx="12" cy="12" r="5"/>
                    <line x1="12" y1="1" x2="12" y2="3"/>
                    <line x1="12" y1="21" x2="12" y2="23"/>
                    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
                    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
                    <line x1="1" y1="12" x2="3" y2="12"/>
                    <line x1="21" y1="12" x2="23" y2="12"/>
                    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
                    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
                }.into_any(),
                
                IconType::Moon => view! {
                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
                }.into_any(),
                
                IconType::System => view! {
                    <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
                    <line x1="8" y1="21" x2="16" y2="21"/>
                    <line x1="12" y1="17" x2="12" y2="21"/>
                }.into_any(),
            }}
        </svg>
    }
}
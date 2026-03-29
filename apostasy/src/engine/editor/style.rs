use std::collections::BTreeMap;

use egui::epaint::Shadow;
use egui::{
    Color32, Margin, Stroke, Style, Vec2, Visuals,
    epaint::CornerRadius,
    style::{
        Interaction, ScrollStyle, Selection, Spacing, TextCursorStyle, WidgetVisuals, Widgets,
    },
};
use egui::{FontFamily, FontId, TextStyle};

// Gruvbox Dark color palette
// bg:    #282828  rgb(40,  40,  40)
// bg0:   #282828  rgb(40,  40,  40)
// bg1:   #3c3836  rgb(60,  56,  54)
// bg2:   #504945  rgb(80,  73,  69)
// bg3:   #665c54  rgb(102, 92,  84)
// bg4:   #7c6f64  rgb(124, 111, 100)
// fg4:   #a89984  rgb(168, 153, 132)
// fg3:   #bdae93  rgb(189, 174, 147)
// fg2:   #d5c4a1  rgb(213, 196, 161)
// fg1:   #ebdbb2  rgb(235, 219, 178)
// fg0:   #fbf1c7  rgb(251, 241, 199)
// red:   #cc241d  rgb(204, 36,  29)
// green: #98971a  rgb(152, 151, 26)
// yellow:#d79921  rgb(215, 153, 33)
// blue:  #458588  rgb(69,  133, 136)
// aqua:  #689d6a  rgb(104, 157, 106)
// orange:#d65d0e  rgb(214, 93,  14)
// gray:  #928374  rgb(146, 131, 116)
// bright_blue:   #83a598  rgb(131, 165, 152)
// bright_yellow: #fabd2f  rgb(250, 189, 47)
// bright_red:    #fb4934  rgb(251, 73,  52)

pub fn style() -> Style {
    let jetbrains = FontFamily::Name("jetbrains".into());

    let mut text_styles = BTreeMap::new();
    text_styles.insert(TextStyle::Small, FontId::new(10.0, jetbrains.clone()));
    text_styles.insert(TextStyle::Body, FontId::new(13.0, jetbrains.clone()));
    text_styles.insert(TextStyle::Button, FontId::new(13.0, jetbrains.clone()));
    text_styles.insert(TextStyle::Heading, FontId::new(18.0, jetbrains.clone()));
    text_styles.insert(TextStyle::Monospace, FontId::new(13.0, jetbrains.clone()));

    Style {
        text_styles,
        spacing: Spacing {
            item_spacing: Vec2 { x: 8.0, y: 3.0 },
            window_margin: Margin::same(6),
            button_padding: Vec2 { x: 4.0, y: 1.0 },
            menu_margin: Margin::same(6),
            indent: 18.0,
            interact_size: Vec2 { x: 40.0, y: 18.0 },
            slider_width: 100.0,
            combo_width: 100.0,
            text_edit_width: 280.0,
            icon_width: 14.0,
            icon_width_inner: 8.0,
            icon_spacing: 4.0,
            tooltip_width: 500.0,
            indent_ends_with_horizontal_line: false,
            combo_height: 200.0,
            scroll: ScrollStyle {
                bar_width: 10.0,
                handle_min_length: 12.0,
                bar_inner_margin: 4.0,
                bar_outer_margin: 0.0,
                ..Default::default()
            },
            ..Default::default()
        },
        interaction: Interaction {
            resize_grab_radius_side: 5.0,
            resize_grab_radius_corner: 10.0,
            show_tooltips_only_when_still: true,
            ..Default::default()
        },
        visuals: Visuals {
            dark_mode: true,
            override_text_color: None,
            widgets: Widgets {
                // bg0 — darkest background, borders use bg2
                noninteractive: WidgetVisuals {
                    bg_fill: Color32::from_rgb(40, 40, 40),      // bg0 #282828
                    weak_bg_fill: Color32::from_rgb(40, 40, 40), // bg0 #282828
                    bg_stroke: Stroke::new(1.0, Color32::from_rgb(80, 73, 69)), // bg2 #504945
                    corner_radius: CornerRadius::same(0),
                    fg_stroke: Stroke::new(1.0, Color32::from_rgb(168, 153, 132)), // fg4 #a89984
                    expansion: 0.0,
                },
                // bg1 — slightly raised surface, no border at rest
                inactive: WidgetVisuals {
                    bg_fill: Color32::from_rgb(60, 56, 54),      // bg1 #3c3836
                    weak_bg_fill: Color32::from_rgb(60, 56, 54), // bg1 #3c3836
                    bg_stroke: Stroke::NONE,
                    corner_radius: CornerRadius::same(0),
                    fg_stroke: Stroke::new(1.0, Color32::from_rgb(189, 174, 147)), // fg3 #bdae93
                    expansion: 0.0,
                },
                // bg3 fill + fg0 text + bg4 border — clearly highlighted
                hovered: WidgetVisuals {
                    bg_fill: Color32::from_rgb(102, 92, 84),      // bg3 #665c54
                    weak_bg_fill: Color32::from_rgb(102, 92, 84), // bg3 #665c54
                    bg_stroke: Stroke::new(1.0, Color32::from_rgb(124, 111, 100)), // bg4 #7c6f64
                    corner_radius: CornerRadius::same(3),
                    fg_stroke: Stroke::new(1.5, Color32::from_rgb(251, 241, 199)), // fg0 #fbf1c7
                    expansion: 1.0,
                },
                // bg2 fill, bright yellow (gruvbox accent) border and text when pressed
                active: WidgetVisuals {
                    bg_fill: Color32::from_rgb(80, 73, 69),      // bg2 #504945
                    weak_bg_fill: Color32::from_rgb(80, 73, 69), // bg2 #504945
                    bg_stroke: Stroke::new(1.0, Color32::from_rgb(250, 189, 47)), // bright_yellow #fabd2f
                    corner_radius: CornerRadius::same(0),
                    fg_stroke: Stroke::new(2.0, Color32::from_rgb(250, 189, 47)), // bright_yellow #fabd2f
                    expansion: 1.0,
                },
                // open menus/combos — bg darkens slightly, subdued border
                open: WidgetVisuals {
                    bg_fill: Color32::from_rgb(29, 32, 33), // hard dark #1d2021
                    weak_bg_fill: Color32::from_rgb(60, 56, 54), // bg1 #3c3836
                    bg_stroke: Stroke::new(1.0, Color32::from_rgb(80, 73, 69)), // bg2 #504945
                    corner_radius: CornerRadius::same(0),
                    fg_stroke: Stroke::new(1.0, Color32::from_rgb(213, 196, 161)), // fg2 #d5c4a1
                    expansion: 0.0,
                },
            },
            selection: Selection {
                // gruvbox blue tinted selection
                bg_fill: Color32::from_rgb(69, 133, 136), // blue #458588
                stroke: Stroke::new(1.0, Color32::from_rgb(131, 165, 152)), // bright_blue #83a598
            },
            hyperlink_color: Color32::from_rgb(131, 165, 152), // bright_blue #83a598
            faint_bg_color: Color32::from_rgba_premultiplied(5, 5, 5, 0),
            extreme_bg_color: Color32::from_rgb(29, 32, 33), // hard dark #1d2021
            code_bg_color: Color32::from_rgb(80, 73, 69),    // bg2 #504945
            warn_fg_color: Color32::from_rgb(250, 189, 47),  // bright_yellow #fabd2f
            error_fg_color: Color32::from_rgb(251, 73, 52),  // bright_red #fb4934
            window_corner_radius: CornerRadius::same(0),
            window_shadow: Shadow {
                color: Color32::from_rgba_premultiplied(0, 0, 0, 96),
                blur: 15,
                offset: [10, 20],
                spread: 0,
            },
            window_fill: Color32::from_rgb(40, 40, 40), // bg0 #282828
            window_stroke: Stroke::new(1.0, Color32::from_rgb(80, 73, 69)), // bg2 #504945
            menu_corner_radius: CornerRadius::same(0),
            panel_fill: Color32::from_rgb(40, 40, 40), // bg0 #282828
            popup_shadow: Shadow {
                color: Color32::from_rgba_premultiplied(0, 0, 0, 96),
                blur: 8,
                offset: [6, 10],
                spread: 0,
            },
            resize_corner_size: 12.0,
            text_cursor: TextCursorStyle {
                stroke: Stroke::new(2.0, Color32::from_rgb(250, 189, 47)), // bright_yellow #fabd2f
                preview: false,
                ..Default::default()
            },
            clip_rect_margin: 3.0,
            button_frame: true,
            collapsing_header_frame: true,
            indent_has_left_vline: true,
            striped: false,
            slider_trailing_fill: false,
            ..Default::default()
        },
        animation_time: 0.0833333358168602,
        explanation_tooltips: false,
        ..Default::default()
    }
}

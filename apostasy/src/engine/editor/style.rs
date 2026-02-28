use egui::epaint::Shadow;
use egui::{
    Color32, Margin, Stroke, Style, Vec2, Visuals,
    epaint::CornerRadius,
    style::{
        Interaction, ScrollStyle, Selection, Spacing, TextCursorStyle, WidgetVisuals, Widgets,
    },
};

pub fn style() -> Style {
    Style {
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
                noninteractive: WidgetVisuals {
                    // Slightly lighter background — matches egui default panels
                    bg_fill: Color32::from_rgba_premultiplied(32, 32, 32, 255),
                    weak_bg_fill: Color32::from_rgba_premultiplied(32, 32, 32, 255),
                    bg_stroke: Stroke::new(1.0, Color32::from_rgba_premultiplied(65, 65, 65, 255)),
                    corner_radius: CornerRadius::same(2),
                    // Brighter text — more readable, matches egui themer
                    fg_stroke: Stroke::new(
                        1.0,
                        Color32::from_rgba_premultiplied(180, 180, 180, 255),
                    ),
                    expansion: 0.0,
                },
                inactive: WidgetVisuals {
                    // Darker buttons — matches egui themer's subtle button look
                    bg_fill: Color32::from_rgba_premultiplied(40, 40, 40, 255),
                    weak_bg_fill: Color32::from_rgba_premultiplied(40, 40, 40, 255),
                    bg_stroke: Stroke::NONE,
                    corner_radius: CornerRadius::same(2),
                    fg_stroke: Stroke::new(
                        1.0,
                        Color32::from_rgba_premultiplied(180, 180, 180, 255),
                    ),
                    expansion: 0.0,
                },
                hovered: WidgetVisuals {
                    bg_fill: Color32::from_rgba_premultiplied(70, 70, 70, 255),
                    weak_bg_fill: Color32::from_rgba_premultiplied(70, 70, 70, 255),
                    bg_stroke: Stroke::new(
                        1.0,
                        Color32::from_rgba_premultiplied(150, 150, 150, 255),
                    ),
                    corner_radius: CornerRadius::same(3),
                    fg_stroke: Stroke::new(
                        1.5,
                        Color32::from_rgba_premultiplied(240, 240, 240, 255),
                    ),
                    expansion: 1.0,
                },
                active: WidgetVisuals {
                    bg_fill: Color32::from_rgba_premultiplied(55, 55, 55, 255),
                    weak_bg_fill: Color32::from_rgba_premultiplied(55, 55, 55, 255),
                    bg_stroke: Stroke::new(1.0, Color32::WHITE),
                    corner_radius: CornerRadius::same(2),
                    fg_stroke: Stroke::new(2.0, Color32::WHITE),
                    expansion: 1.0,
                },
                open: WidgetVisuals {
                    bg_fill: Color32::from_rgba_premultiplied(27, 27, 27, 255),
                    weak_bg_fill: Color32::from_rgba_premultiplied(45, 45, 45, 255),
                    bg_stroke: Stroke::new(1.0, Color32::from_rgba_premultiplied(60, 60, 60, 255)),
                    corner_radius: CornerRadius::same(2),
                    fg_stroke: Stroke::new(
                        1.0,
                        Color32::from_rgba_premultiplied(210, 210, 210, 255),
                    ),
                    expansion: 0.0,
                },
            },
            selection: Selection {
                bg_fill: Color32::from_rgba_premultiplied(0, 92, 128, 255),
                stroke: Stroke::new(1.0, Color32::from_rgba_premultiplied(192, 222, 255, 255)),
            },
            hyperlink_color: Color32::from_rgba_premultiplied(90, 170, 255, 255),
            faint_bg_color: Color32::from_rgba_premultiplied(5, 5, 5, 0),
            extreme_bg_color: Color32::from_rgba_premultiplied(10, 10, 10, 255),
            code_bg_color: Color32::from_rgba_premultiplied(64, 64, 64, 255),
            warn_fg_color: Color32::from_rgba_premultiplied(255, 143, 0, 255),
            error_fg_color: Color32::from_rgba_premultiplied(255, 0, 0, 255),
            // Reduced corner radius — matches egui themer's subtle rounding
            window_corner_radius: CornerRadius::same(6),
            window_shadow: Shadow {
                color: Color32::from_rgba_premultiplied(0, 0, 0, 96),
                blur: 15,
                offset: [10, 20],
                spread: 0,
            },
            // Slightly lighter window/panel fill — matches egui themer
            window_fill: Color32::from_rgba_premultiplied(32, 32, 32, 255),
            window_stroke: Stroke::new(1.0, Color32::from_rgba_premultiplied(65, 65, 65, 255)),
            menu_corner_radius: CornerRadius::same(6),
            panel_fill: Color32::from_rgba_premultiplied(32, 32, 32, 255),
            popup_shadow: Shadow {
                color: Color32::from_rgba_premultiplied(0, 0, 0, 96),
                blur: 8,
                offset: [6, 10],
                spread: 0,
            },
            resize_corner_size: 12.0,
            text_cursor: TextCursorStyle {
                stroke: Stroke::new(2.0, Color32::from_rgba_premultiplied(192, 222, 255, 255)),
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

use crate as apostasy;
use std::collections::VecDeque;
use std::time::Instant;

use apostasy_macros::editor_ui;
use ash::vk;
use egui::{Color32, FontId, Sense, Vec2};

use crate::engine::{editor::EditorStorage, nodes::world::World};

#[derive(Clone, Debug)]
pub struct ProfileScope {
    pub name: String,
    pub duration_ms: f64,
    pub color: Color32,
    pub depth: usize,
}

#[derive(Clone, Debug, Default)]
pub struct FrameData {
    pub frame_index: u64,
    pub frame_time_ms: f64,
    pub cpu_scopes: Vec<ProfileScope>,
    pub gpu_scopes: Vec<ProfileScope>,
    pub cpu_total_ms: f64,
    pub gpu_total_ms: f64,
}

pub struct ProfilerHistory {
    pub frames: VecDeque<FrameData>,
    capacity: usize,
}

impl ProfilerHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            frames: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, frame: FrameData) {
        if self.frames.len() == self.capacity {
            self.frames.pop_front();
        }
        self.frames.push_back(frame);
    }

    pub fn avg_frame_ms(&self) -> f64 {
        if self.frames.is_empty() {
            return 0.0;
        }
        self.frames.iter().map(|f| f.frame_time_ms).sum::<f64>() / self.frames.len() as f64
    }

    pub fn avg_fps(&self) -> f64 {
        let ms = self.avg_frame_ms();
        if ms > 0.0 { 1000.0 / ms } else { 0.0 }
    }

    pub fn peak_frame_ms(&self) -> f64 {
        self.frames
            .iter()
            .map(|f| f.frame_time_ms)
            .fold(0.0_f64, f64::max)
    }
}

const CPU_COLORS: &[Color32] = &[
    Color32::from_rgb(180, 100, 255),
    Color32::from_rgb(220, 130, 255),
    Color32::from_rgb(150, 80, 200),
    Color32::from_rgb(200, 160, 255),
];

pub struct CpuProfiler {
    stack: Vec<(String, Instant, usize)>,
    pub completed: Vec<ProfileScope>,
}

impl CpuProfiler {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            completed: Vec::new(),
        }
    }

    pub fn begin(&mut self, name: &str) {
        let depth = self.stack.len();
        self.stack.push((name.to_string(), Instant::now(), depth));
    }

    pub fn end(&mut self) {
        if let Some((name, start, depth)) = self.stack.pop() {
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            self.completed.push(ProfileScope {
                name,
                duration_ms: ms,
                color: CPU_COLORS[depth % CPU_COLORS.len()],
                depth,
            });
        }
    }

    pub fn drain(&mut self) -> Vec<ProfileScope> {
        std::mem::take(&mut self.completed)
    }
}

const MAX_TIMESTAMPS: u32 = 64;

const GPU_COLORS: &[Color32] = &[
    Color32::from_rgb(100, 200, 255),
    Color32::from_rgb(80, 230, 180),
    Color32::from_rgb(255, 180, 60),
    Color32::from_rgb(255, 100, 100),
    Color32::from_rgb(60, 210, 120),
];

pub struct GpuTimestampPool {
    pub query_pool: vk::QueryPool,
    pub timestamp_period_ns: f64,
    next_slot: u32,
    scope_stack: Vec<(String, u32)>,
    pending: Vec<(String, u32, u32)>,
    pub frame_active: bool,
}

impl GpuTimestampPool {
    pub fn new(device: &ash::Device, _queue: vk::Queue) -> Self {
        let pool_info = vk::QueryPoolCreateInfo::default()
            .query_type(vk::QueryType::TIMESTAMP)
            .query_count(MAX_TIMESTAMPS);
        let query_pool = unsafe {
            device
                .create_query_pool(&pool_info, None)
                .expect("Failed to create timestamp query pool")
        };
        Self {
            query_pool,
            timestamp_period_ns: 1.0,
            next_slot: 0,
            scope_stack: Vec::new(),
            pending: Vec::new(),
            frame_active: false,
        }
    }

    pub fn set_timestamp_period(&mut self, period_ns: f32) {
        self.timestamp_period_ns = period_ns as f64;
    }

    pub fn begin_frame(&mut self, device: &ash::Device, cmd: vk::CommandBuffer) {
        self.next_slot = 0;
        self.scope_stack.clear();
        self.pending.clear();
        self.frame_active = true;
        unsafe {
            device.cmd_reset_query_pool(cmd, self.query_pool, 0, MAX_TIMESTAMPS);
        }
    }

    pub fn begin_scope(&mut self, device: &ash::Device, cmd: vk::CommandBuffer, name: &str) {
        if self.next_slot + 2 > MAX_TIMESTAMPS {
            return;
        }
        let slot = self.next_slot;
        self.next_slot += 1;
        unsafe {
            device.cmd_write_timestamp(
                cmd,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                self.query_pool,
                slot,
            );
        }
        self.scope_stack.push((name.to_string(), slot));
    }

    pub fn end_scope(&mut self, device: &ash::Device, cmd: vk::CommandBuffer) {
        if let Some((name, start_slot)) = self.scope_stack.pop() {
            let end_slot = self.next_slot;
            self.next_slot += 1;
            unsafe {
                device.cmd_write_timestamp(
                    cmd,
                    vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                    self.query_pool,
                    end_slot,
                );
            }
            self.pending.push((name, start_slot, end_slot));
        }
    }

    pub fn resolve(&self, device: &ash::Device) -> Vec<ProfileScope> {
        let count = self.next_slot as usize;
        if count == 0 {
            return Vec::new();
        }
        let mut raw = vec![0u64; count];
        let ok = unsafe {
            device.get_query_pool_results(
                self.query_pool,
                0,
                &mut raw,
                vk::QueryResultFlags::TYPE_64 | vk::QueryResultFlags::WAIT,
            )
        };
        if ok.is_err() {
            return Vec::new();
        }
        self.pending
            .iter()
            .enumerate()
            .map(|(i, (name, start, end))| {
                let ticks = raw[*end as usize].saturating_sub(raw[*start as usize]);
                let duration_ms = (ticks as f64 * self.timestamp_period_ns) / 1_000_000.0;
                ProfileScope {
                    name: name.clone(),
                    duration_ms,
                    color: GPU_COLORS[i % GPU_COLORS.len()],
                    depth: 0,
                }
            })
            .collect()
    }

    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.destroy_query_pool(self.query_pool, None);
        }
    }
}

pub struct ProfilerState {
    pub history: ProfilerHistory,
    pub visible: bool,
    pub paused: bool,
    pub target_fps: f64,
    pub show_cpu: bool,
    pub show_gpu: bool,
    pub pinned_frame: Option<usize>,
}

impl Default for ProfilerState {
    fn default() -> Self {
        Self {
            history: ProfilerHistory::new(256),
            visible: false,
            paused: false,
            target_fps: 60.0,
            show_cpu: true,
            show_gpu: true,
            pinned_frame: None,
        }
    }
}

#[editor_ui]
pub fn render_profiler(ctx: &mut egui::Context, _world: &mut World, editor: &mut EditorStorage) {
    ctx.input(|i| {
        if i.key_pressed(egui::Key::F3) {
            editor.profiler.visible = !editor.profiler.visible;
        }
    });

    if !editor.profiler.visible {
        return;
    }

    egui::Window::new("⏱  Profiler")
        .default_width(680.0)
        .default_height(300.0)
        .min_height(160.0)
        .min_width(400.0)
        .resizable(true)
        .show(ctx, |ui| {
            draw_toolbar(ui, &mut editor.profiler);
            ui.separator();
            draw_stats_bar(ui, &editor.profiler);
            ui.separator();
            draw_graph(ui, &mut editor.profiler);
            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt("profiler_breakdown_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let pinned = editor.profiler.pinned_frame;
                    let idx = pinned.or_else(|| {
                        let len = editor.profiler.history.frames.len();
                        if len == 0 { None } else { Some(len - 1) }
                    });

                    if let Some(idx) = idx {
                        if let Some(frame) = editor.profiler.history.frames.get(idx).cloned() {
                            draw_breakdown(ui, &editor.profiler, &frame);
                        }
                    }
                });
        });
}

fn draw_breakdown(ui: &mut egui::Ui, state: &ProfilerState, frame: &FrameData) {
    let total_ms = frame.frame_time_ms.max(0.001);

    // Use a stable id so the open/closed state persists across frame updates
    let breakdown_id = ui.make_persistent_id("profiler_breakdown");
    let breakdown_state = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        breakdown_id,
        true,
    );

    let header = breakdown_state.show_header(ui, |ui| {
        ui.label(
            egui::RichText::new(format!("Frame #{}  breakdown", frame.frame_index))
                .strong()
                .size(12.0),
        );
    });
    header.body(|ui| {
        egui::CollapsingHeader::new("Flame")
            .default_open(true)
            .show(ui, |ui| {
                let draw_scopes = |ui: &mut egui::Ui, scopes: &[ProfileScope], kind: &str| {
                    for scope in scopes {
                        let indent = scope.depth as f32 * 14.0;

                        let row_w = (ui.available_width() - indent).max(4.0);
                        let (rect, resp) = ui.allocate_exact_size(
                            egui::Vec2::new(row_w, 18.0),
                            egui::Sense::hover(),
                        );

                        // Shift the rect right by indent amount
                        let rect = egui::Rect::from_min_size(
                            rect.min + egui::Vec2::new(indent, 0.0),
                            egui::Vec2::new(row_w - indent, 18.0),
                        );

                        let painter = ui.painter_at(rect);

                        // Background track
                        painter.rect_filled(rect, 2.0, egui::Color32::from_black_alpha(60));

                        // Filled portion — fraction of the row width
                        let frac = ((scope.duration_ms / total_ms) as f32).clamp(0.0, 1.0);
                        let bar_rect = egui::Rect::from_min_size(
                            rect.min,
                            egui::Vec2::new((rect.width() * frac).max(3.0), rect.height()),
                        );
                        painter.rect_filled(bar_rect, 2.0, scope.color);

                        // Label drawn over the bar
                        painter.text(
                            rect.left_center() + egui::Vec2::new(4.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            format!(
                                "{} [{kind}]  {:.3} ms  ({:.1}%)",
                                scope.name,
                                scope.duration_ms,
                                (scope.duration_ms / total_ms) * 100.0,
                            ),
                            egui::FontId::proportional(11.0),
                            egui::Color32::WHITE,
                        );

                        if resp.hovered() {
                            egui::Tooltip::always_open(
                                ui.ctx().clone(),
                                ui.layer_id(),
                                egui::Id::new("profiler_tooltip"),
                                resp.rect,
                            )
                            .at_pointer()
                            .show(|ui| {
                                ui.label(format!(
                                    "[{kind}] {}  {:.3} ms  ({:.1}%)",
                                    scope.name,
                                    scope.duration_ms,
                                    (scope.duration_ms / total_ms) * 100.0,
                                ));
                            });
                        }
                    }
                };

                if state.show_cpu {
                    draw_scopes(ui, &frame.cpu_scopes, "CPU");
                }
                if state.show_gpu {
                    draw_scopes(ui, &frame.gpu_scopes, "GPU");
                }
            });

        egui::CollapsingHeader::new("Table")
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("profiler_table")
                    .striped(true)
                    .min_col_width(90.0)
                    .spacing([12.0, 2.0])
                    .show(ui, |ui| {
                        for h in &["Scope", "Type", "ms", "%"] {
                            ui.label(egui::RichText::new(*h).strong().size(11.0));
                        }
                        ui.end_row();

                        let show_scopes = |ui: &mut egui::Ui,
                                           scopes: &[ProfileScope],
                                           kind: &str| {
                            for scope in scopes {
                                ui.colored_label(scope.color, &scope.name);
                                ui.label(kind);
                                ui.label(format!("{:.3}", scope.duration_ms));
                                ui.label(format!("{:.1}%", (scope.duration_ms / total_ms) * 100.0));
                                ui.end_row();
                            }
                        };

                        if state.show_cpu {
                            show_scopes(ui, &frame.cpu_scopes, "CPU");
                        }
                        if state.show_gpu {
                            show_scopes(ui, &frame.gpu_scopes, "GPU");
                        }
                    });
            });
    });
}

fn draw_toolbar(ui: &mut egui::Ui, state: &mut ProfilerState) {
    ui.horizontal(|ui| {
        let label = if state.paused {
            "▶  Resume"
        } else {
            "⏸  Pause"
        };
        if ui.button(label).clicked() {
            state.paused = !state.paused;
            if !state.paused {
                state.pinned_frame = None;
            }
        }
        ui.separator();
        ui.checkbox(&mut state.show_cpu, "CPU");
        ui.checkbox(&mut state.show_gpu, "GPU");
        ui.separator();
        ui.label("Target:");
        ui.add(
            egui::DragValue::new(&mut state.target_fps)
                .range(15.0..=360.0)
                .suffix(" fps")
                .speed(1.0),
        );
        ui.separator();
        ui.label(
            egui::RichText::new("F3 toggle")
                .color(Color32::from_gray(120))
                .size(10.0),
        );
    });
}

fn draw_stats_bar(ui: &mut egui::Ui, state: &ProfilerState) {
    let avg_ms = state.history.avg_frame_ms();
    let avg_fps = state.history.avg_fps();
    let peak_ms = state.history.peak_frame_ms();
    let target_ms = 1000.0 / state.target_fps;

    let ms_color = if avg_ms <= target_ms {
        Color32::from_rgb(80, 220, 80)
    } else if avg_ms <= target_ms * 1.5 {
        Color32::from_rgb(255, 200, 50)
    } else {
        Color32::from_rgb(255, 70, 70)
    };

    ui.horizontal(|ui| {
        ui.label("Avg:");
        ui.colored_label(ms_color, format!("{avg_ms:.2} ms"));
        ui.separator();
        ui.label("FPS:");
        ui.colored_label(ms_color, format!("{avg_fps:.1}"));
        ui.separator();
        ui.label("Peak:");
        ui.colored_label(Color32::from_rgb(255, 140, 60), format!("{peak_ms:.2} ms"));

        if let Some(last) = state.history.frames.back() {
            if state.show_cpu {
                ui.separator();
                ui.label("CPU:");
                ui.colored_label(
                    Color32::from_rgb(180, 100, 255),
                    format!("{:.2} ms", last.cpu_total_ms),
                );
            }
            if state.show_gpu {
                ui.separator();
                ui.label("GPU:");
                ui.colored_label(
                    Color32::from_rgb(100, 200, 255),
                    format!("{:.2} ms", last.gpu_total_ms),
                );
            }
        }
    });
}

fn draw_graph(ui: &mut egui::Ui, state: &mut ProfilerState) {
    let target_ms = 1000.0 / state.target_fps;
    let history_len = state.history.frames.len();
    if history_len == 0 {
        return;
    }

    let desired = Vec2::new(ui.available_width(), 110.0);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::click());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 2.0, Color32::from_rgb(18, 18, 18));

    let max_ms = (state.history.peak_frame_ms() * 1.2).max(target_ms * 2.0);
    let x_step = rect.width() / history_len as f32;

    let to_y = |ms: f64| -> f32 { rect.bottom() - (ms / max_ms) as f32 * rect.height() };

    // Target line
    let ty = to_y(target_ms);
    painter.line_segment(
        [egui::pos2(rect.left(), ty), egui::pos2(rect.right(), ty)],
        egui::Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 240, 80, 100)),
    );
    painter.text(
        egui::pos2(rect.right() - 4.0, ty - 11.0),
        egui::Align2::RIGHT_BOTTOM,
        format!("{:.1} ms", target_ms),
        FontId::proportional(9.0),
        Color32::from_rgba_unmultiplied(255, 240, 80, 160),
    );

    // Polyline helper
    let frames: Vec<&FrameData> = state.history.frames.iter().collect();
    let draw_line =
        |painter: &egui::Painter, get: &dyn Fn(&FrameData) -> f64, color: Color32, width: f32| {
            let pts: Vec<egui::Pos2> = frames
                .iter()
                .enumerate()
                .map(|(i, f)| egui::pos2(rect.left() + i as f32 * x_step, to_y(get(f))))
                .collect();
            for w in pts.windows(2) {
                painter.line_segment([w[0], w[1]], egui::Stroke::new(width, color));
            }
        };

    draw_line(
        &painter,
        &|f| f.frame_time_ms,
        Color32::from_rgb(200, 200, 200),
        1.5,
    );
    if state.show_cpu {
        draw_line(
            &painter,
            &|f| f.cpu_total_ms,
            Color32::from_rgb(180, 100, 255),
            1.0,
        );
    }
    if state.show_gpu {
        draw_line(
            &painter,
            &|f| f.gpu_total_ms,
            Color32::from_rgb(100, 200, 255),
            1.0,
        );
    }

    // Y-axis labels
    for &ms in &[target_ms, max_ms * 0.5, max_ms] {
        let y = to_y(ms);
        if y >= rect.top() && y <= rect.bottom() {
            painter.text(
                egui::pos2(rect.left() + 4.0, y),
                egui::Align2::LEFT_TOP,
                format!("{:.1}", ms),
                FontId::proportional(9.0),
                Color32::from_gray(110),
            );
        }
    }

    // Legend
    let legend: &[(&str, Color32)] = &[
        ("Frame", Color32::from_rgb(200, 200, 200)),
        ("CPU", Color32::from_rgb(180, 100, 255)),
        ("GPU", Color32::from_rgb(100, 200, 255)),
    ];
    let mut lx = rect.left() + 6.0;
    for (label, color) in legend {
        painter.line_segment(
            [
                egui::pos2(lx, rect.top() + 8.0),
                egui::pos2(lx + 14.0, rect.top() + 8.0),
            ],
            egui::Stroke::new(2.0, *color),
        );
        painter.text(
            egui::pos2(lx + 18.0, rect.top() + 2.0),
            egui::Align2::LEFT_TOP,
            *label,
            FontId::proportional(10.0),
            *color,
        );
        lx += 58.0;
    }

    // Click to pin frame
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let norm = (pos.x - rect.left()) / rect.width();
            let idx = (norm * history_len as f32) as usize;
            state.pinned_frame = Some(idx.min(history_len.saturating_sub(1)));
            state.paused = true;
        }
    }
}

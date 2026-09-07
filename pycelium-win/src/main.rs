mod config;
mod cutter;
mod export;
mod export_settings;
mod gpu;
mod hud_font;
mod memory;
mod model;
mod species;
mod teach;
mod types;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use anyhow::Result;
use clap::Parser;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use crate::config::{Args, LabelDensity, MemoryPlan};
use crate::cutter::{ray_cube_midpoint, Cutter};
use crate::export::{apply_export_layout, extract_slice, utc_stamp, write_exports_with_meta};
use crate::export_settings::ExportSettings;
use crate::gpu::{GpuDevice, MyceliumGpu};
use crate::memory::HostWorld;
use crate::species::{strip_hit, BrushState, PaintChannel, StripHit, ToolMode};
use crate::teach::{HoverId, HudSlider, NudgeKind, NudgeToast};
use crate::types::SimUniforms;

enum AppAction {
    Ready(Box<Runtime>),
}

struct Orbit {
    yaw: f32,
    pitch: f32,
    radius: f32,
    dragging: bool,
    last: Option<(f64, f64)>,
}

impl Orbit {
    fn eye_target(&self) -> ([f32; 3], [f32; 3]) {
        let target = [0.50, 0.50, 0.36];
        let cp = self.pitch.cos();
        let eye = [
            target[0] + self.radius * self.yaw.cos() * cp,
            target[1] + self.radius * self.yaw.sin() * cp,
            target[2] + self.radius * self.pitch.sin(),
        ];
        (eye, target)
    }
}

struct Runtime {
    window: Arc<Window>,
    gpu: GpuDevice,
    sim: MyceliumGpu,
    world: HostWorld,
    plan: MemoryPlan,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    paused: bool,
    frames: u32,
    fps_mark: Instant,
    fps: f32,
    orbit: Orbit,
    pending_pick: bool,
    has_picked: bool,
    cursor: (f32, f32),
    shift: bool,
    labels: LabelDensity,
    label_fade: f32,
    cutter: Cutter,
    export: ExportSettings,
    export_dir: PathBuf,
    scheme_visible: bool,
    scheme_fade: f32,
    tip_fade: f32,
    last_hover: HoverId,
    nudge: Option<LiveNudge>,
    hud_drag: Option<HudDrag>,
    brush: BrushState,
    alt: bool,
}

struct LiveNudge {
    kind: NudgeKind,
    title: String,
    old: String,
    new: String,
    last: Instant,
}

#[derive(Clone)]
struct HudDrag {
    slider: HudSlider,
    start: String,
}

struct App {
    proxy: EventLoopProxy<AppAction>,
    plan: MemoryPlan,
    world: Option<HostWorld>,
    runtime: Option<Runtime>,
    labels: LabelDensity,
    export_dir: PathBuf,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let args = Args::parse();
    let plan = MemoryPlan::from_args(&args);
    println!("Pycelium 3D mesocosm");
    println!("{}", plan.describe());

    let world = HostWorld::commit(&plan)?;
    let labels = args.labels;
    let export_dir = PathBuf::from(&args.export_dir);

    if let Some(frames) = args.bench {
        return run_bench(plan, world, frames);
    }

    let event_loop = EventLoop::<AppAction>::with_user_event().build()?;
    let mut app = App {
        proxy: event_loop.create_proxy(),
        plan,
        world: Some(world),
        runtime: None,
        labels,
        export_dir,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}

fn run_bench(plan: MemoryPlan, world: HostWorld, frames: u32) -> Result<()> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let gpu = pollster::block_on(GpuDevice::request(instance, None))?;
    println!("adapter: {}", gpu.describe());
    let mut sim = MyceliumGpu::new(
        &gpu.device,
        &gpu.queue,
        &plan,
        &world,
        wgpu::TextureFormat::Rgba8UnormSrgb,
    )?;
    println!("warming up...");
    sim.step(&gpu.device, &gpu.queue, 4, false);
    gpu.device.poll(wgpu::PollType::wait_indefinitely())?;
    let started = Instant::now();
    sim.step(&gpu.device, &gpu.queue, frames, false);
    gpu.device.poll(wgpu::PollType::wait_indefinitely())?;
    let elapsed = started.elapsed().as_secs_f64().max(1e-6);
    println!(
        "bench: {frames} steps in {elapsed:.3}s | {:.1} steps/s | {:.2} M tip-slots/s | host {:.2} GiB | brick {}x{}x{}",
        frames as f64 / elapsed,
        plan.gpu_agents as f64 * (frames as f64 / elapsed) / 1_000_000.0,
        world.committed_bytes() as f64 / 1024.0 / 1024.0 / 1024.0,
        plan.gpu_width,
        plan.gpu_height,
        plan.gpu_depth
    );
    Ok(())
}

impl ApplicationHandler<AppAction> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.runtime.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Pycelium mesocosm")
                        .with_inner_size(LogicalSize::new(1600.0, 900.0)),
                )
                .expect("window"),
        );
        let display_handle = event_loop.owned_display_handle();
        let proxy = self.proxy.clone();
        let plan = self.plan.clone();
        let world = self.world.take().expect("host world");
        let vsync = plan.vsync;
        let labels = self.labels;
        let export_dir = self.export_dir.clone();
        let cutter_depth = plan.gpu_depth;

        pollster::block_on(async move {
            let instance = wgpu::Instance::new(
                wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display_handle)),
            );
            let surface = instance.create_surface(window.clone()).expect("surface");
            let gpu = GpuDevice::request(instance, Some(&surface))
                .await
                .expect("gpu device");
            let caps = surface.get_capabilities(&gpu.adapter);
            let format = caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(caps.formats[0]);
            let present_mode = choose_present_mode(&caps.present_modes, vsync);
            let size = window.inner_size();
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                color_space: wgpu::SurfaceColorSpace::Auto,
                width: size.width.max(1),
                height: size.height.max(1),
                present_mode,
                desired_maximum_frame_latency: 2,
                alpha_mode: caps.alpha_modes[0],
                view_formats: vec![],
            };
            surface.configure(&gpu.device, &config);
            println!("adapter: {}", gpu.describe());
            let sim = MyceliumGpu::new(&gpu.device, &gpu.queue, &plan, &world, format)
                .expect("sim pipelines");
            let _ = proxy.send_event(AppAction::Ready(Box::new(Runtime {
                window,
                gpu,
                sim,
                world,
                plan,
                surface,
                config,
                paused: false,
                frames: 0,
                fps_mark: Instant::now(),
                fps: 0.0,
                orbit: Orbit {
                    yaw: 0.85,
                    pitch: 0.38,
                    radius: 1.85,
                    dragging: false,
                    last: None,
                },
                pending_pick: false,
                has_picked: false,
                cursor: (0.5, 0.5),
                shift: false,
                labels,
                label_fade: if labels == LabelDensity::Off { 0.0 } else { 1.0 },
                cutter: Cutter::new(cutter_depth),
                export: ExportSettings::default(),
                export_dir,
                scheme_visible: false,
                scheme_fade: 0.0,
                tip_fade: 0.0,
                last_hover: HoverId::None,
                nudge: None,
                hud_drag: None,
                brush: BrushState::default(),
                alt: false,
            })));
        });
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppAction) {
        match event {
            AppAction::Ready(runtime) => {
                event_loop.set_control_flow(if runtime.plan.vsync {
                    ControlFlow::Wait
                } else {
                    ControlFlow::Poll
                });
                runtime.window.request_redraw();
                self.runtime = Some(*runtime);
                println!(
                    "drag orbit | wheel zoom | click pick tip | 1-8 param | -/= tune"
                );
                println!("[ ] depth | ; ' thickness | , . XY field | arrows pan slab | Shift = coarse");
                println!("Space pause | R reseed | F litter | D drift | Esc quit");
                println!(
                    "Tab HUD labels ({}) | white names + color boxes | hover in sparse",
                    self.runtime.as_ref().map(|r| r.labels.as_str()).unwrap_or("rich")
                );
                println!(
                    "E capture | C 2D/rich | S export settings | hover cube face to snap | drag slider handles | Enter export | Esc leave"
                );
                println!(
                    "HUD: FPS, tips, fusions, branches, C:N | bars | then slice Z / thickness / zoom% | selected tip"
                );
                println!("H control scheme | hover a meter or scheme row for a teach callout");
                println!(
                    "T specimen/paint | click a species square | 1-8 slot | 9/0 brush | Alt erase | right-drag orbit"
                );
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(rt) = self.runtime.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                rt.config.width = size.width.max(1);
                rt.config.height = size.height.max(1);
                rt.surface.configure(&rt.gpu.device, &rt.config);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let x = position.x;
                let y = position.y;
                if let Some((lx, ly)) = rt.orbit.last {
                    if rt.orbit.dragging {
                        rt.orbit.yaw += (x - lx) as f32 * 0.005;
                        rt.orbit.pitch =
                            (rt.orbit.pitch + (y - ly) as f32 * 0.005).clamp(-1.2, 1.2);
                    }
                }
                if rt.orbit.dragging {
                    rt.orbit.last = Some((x, y));
                }
                rt.cursor = (
                    (x as f32 / rt.config.width as f32).clamp(0.0, 1.0),
                    (y as f32 / rt.config.height as f32).clamp(0.0, 1.0),
                );
                if let Some(drag) = rt.hud_drag.clone() {
                    apply_hud_slider(rt, drag.slider, rt.cursor.0, &drag.start);
                    rt.window.request_redraw();
                }
                if rt.brush.stroking && !rt.cutter.active && rt.hud_drag.is_none() {
                    try_stamp_brush(rt);
                    rt.window.request_redraw();
                }
                if rt.cutter.active {
                    let axis_n = rt.cutter.axis.size(rt.sim.width, rt.sim.height, rt.sim.depth)
                        as f32;
                    if rt.cutter.dragging.is_some() {
                        rt.cutter.drag_handle(rt.cursor, axis_n);
                    } else if !rt.orbit.dragging
                        && rt.hud_drag.is_none()
                        && !rt.export.contains_cursor(rt.cursor)
                    {
                        let (eye, target) = rt.orbit.eye_target();
                        let rd = click_dir(
                            rt.cursor,
                            rt.config.width,
                            rt.config.height,
                            eye,
                            target,
                        );
                        rt.cutter.track_pointer(eye, rd, axis_n);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Right {
                    rt.orbit.dragging = state == ElementState::Pressed;
                    rt.orbit.last = None;
                    if state == ElementState::Released {
                        rt.brush.stroking = false;
                    }
                    return;
                }
                if button == MouseButton::Left {
                    let scheme_rect = teach::scheme_rect(rt.export.panel_open && rt.cutter.active);
                    if rt.scheme_visible && teach::in_rect(rt.cursor, scheme_rect) {
                        rt.orbit.dragging = false;
                        rt.orbit.last = None;
                        return;
                    }
                    if state == ElementState::Pressed {
                        if let Some(hit) = strip_hit(rt.cursor) {
                            apply_strip_hit(rt, hit);
                            rt.orbit.dragging = false;
                            rt.orbit.last = None;
                            rt.window.request_redraw();
                            return;
                        }
                        if let Some(slider) = teach::hud_slider_at(rt.cursor) {
                            let start = match slider {
                                HudSlider::SliceZ => fmt_slice_z(rt.sim.slice.z),
                                HudSlider::Thick => fmt_thick(rt.sim.slice.thickness),
                                HudSlider::Zoom => fmt_zoom(rt.sim.slice.zoom),
                                HudSlider::Param(slot) => {
                                    rt.sim.param_slot = slot as u32;
                                    fmt_param(
                                        rt.sim.param_slot,
                                        rt.sim.uniforms.param_value(rt.sim.param_slot),
                                    )
                                }
                            };
                            if rt.cursor.0 >= teach::SLIDER_TRACK_X0 {
                                apply_hud_slider(rt, slider, rt.cursor.0, &start);
                            } else {
                                match slider {
                                    HudSlider::SliceZ => fire_nudge(
                                        rt,
                                        NudgeKind::SliceZ,
                                        "SLICE Z".into(),
                                        start.clone(),
                                        start.clone(),
                                    ),
                                    HudSlider::Thick => fire_nudge(
                                        rt,
                                        NudgeKind::Thick,
                                        "THICK".into(),
                                        start.clone(),
                                        start.clone(),
                                    ),
                                    HudSlider::Zoom => fire_nudge(
                                        rt,
                                        NudgeKind::Zoom,
                                        "ZOOM".into(),
                                        start.clone(),
                                        start.clone(),
                                    ),
                                    HudSlider::Param(_) => fire_nudge(
                                        rt,
                                        NudgeKind::Param,
                                        param_title(rt.sim.param_slot),
                                        start.clone(),
                                        start.clone(),
                                    ),
                                }
                            }
                            rt.hud_drag = Some(HudDrag { slider, start });
                            rt.orbit.dragging = false;
                            rt.orbit.last = None;
                            rt.window.request_redraw();
                            return;
                        }
                    }
                    if state == ElementState::Released && rt.hud_drag.is_some() {
                        rt.hud_drag = None;
                        rt.orbit.dragging = false;
                        rt.orbit.last = None;
                        return;
                    }
                    if state == ElementState::Pressed && rt.cutter.active {
                        if rt.export.contains_cursor(rt.cursor) {
                            if let Some(hit) = rt.export.hit(rt.cursor) {
                                rt.export.apply_hit(hit);
                                rt.cutter.export_heightmap = rt.export.write_heightmap;
                                println!("{}", rt.export.describe());
                            }
                            rt.orbit.dragging = false;
                            rt.orbit.last = None;
                            rt.window.request_redraw();
                            return;
                        }
                        let axis_n = rt.cutter.axis.size(rt.sim.width, rt.sim.height, rt.sim.depth)
                            as f32;
                        if let Some(handle) = rt.cutter.hit_handle(rt.cursor, axis_n) {
                            rt.cutter.dragging = Some(handle);
                            rt.orbit.dragging = false;
                            rt.orbit.last = None;
                            return;
                        }
                    }
                    if state == ElementState::Released && rt.cutter.dragging.is_some() {
                        rt.cutter.dragging = None;
                        rt.orbit.dragging = false;
                        rt.orbit.last = None;
                        return;
                    }
                    let tool_draw = !rt.cutter.active
                        && rt.brush.mode != ToolMode::View
                        && rt.hud_drag.is_none();
                    if tool_draw {
                        if state == ElementState::Pressed {
                            rt.brush.stroking = true;
                            rt.brush.last_stamp = None;
                            rt.orbit.dragging = false;
                            rt.orbit.last = None;
                            try_stamp_brush(rt);
                        } else {
                            rt.brush.stroking = false;
                            rt.brush.last_stamp = None;
                            rt.orbit.dragging = false;
                            rt.orbit.last = None;
                        }
                        rt.window.request_redraw();
                        return;
                    }
                    rt.orbit.dragging = state == ElementState::Pressed;
                    rt.orbit.last = None;
                    if state == ElementState::Released && !rt.cutter.active {
                        rt.pending_pick = true;
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let steps = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 80.0,
                };
                if rt.cutter.active && rt.export.contains_cursor(rt.cursor) {
                    rt.export.cycle_preset(if steps > 0.0 { 1 } else { -1 });
                    println!("{}", rt.export.describe());
                    rt.window.request_redraw();
                } else if rt.cutter.active && rt.shift {
                    let axis_n = rt.cutter.axis.size(rt.sim.width, rt.sim.height, rt.sim.depth)
                        as f32;
                    rt.cutter.nudge_half(steps * if rt.shift { 2.0 } else { 1.0 }, axis_n);
                } else if !rt.cutter.active
                    && (rt.brush.mode != ToolMode::View || strip_hit(rt.cursor).is_some())
                {
                    let step = if rt.shift { 2.4 } else { 0.8 };
                    rt.brush.nudge_radius(steps * step);
                    fire_brush_toast(rt);
                    rt.window.request_redraw();
                } else {
                    rt.orbit.radius = (rt.orbit.radius - steps * 0.12).clamp(0.55, 4.5);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key,
                        state,
                        repeat,
                        ..
                    },
                ..
            } => {
                if matches!(logical_key, Key::Named(NamedKey::Shift)) {
                    rt.shift = state == ElementState::Pressed;
                    return;
                }
                if matches!(logical_key, Key::Named(NamedKey::Alt)) {
                    rt.alt = state == ElementState::Pressed;
                    rt.brush.erase = rt.alt;
                    if rt.brush.mode != ToolMode::View {
                        fire_brush_toast(rt);
                        rt.window.request_redraw();
                    }
                    return;
                }
                if state != ElementState::Pressed {
                    return;
                }
                let max_z = rt.sim.depth as f32;
                let step = if rt.shift { 8.0 } else { 1.0 };
                let old_z = fmt_slice_z(rt.sim.slice.z);
                let old_thick = fmt_thick(rt.sim.slice.thickness);
                let old_zoom = fmt_zoom(rt.sim.slice.zoom);
                let old_pan = fmt_pan(rt.sim.slice.ox, rt.sim.slice.oy);
                let slice_key = match &logical_key {
                    Key::Character(c) if c == "[" => {
                        rt.sim.slice.nudge_depth(-step, max_z);
                        true
                    }
                    Key::Character(c) if c == "]" => {
                        rt.sim.slice.nudge_depth(step, max_z);
                        true
                    }
                    Key::Character(c) if c == ";" => {
                        rt.sim.slice.nudge_thickness(-step, max_z);
                        true
                    }
                    Key::Character(c) if c == "'" => {
                        rt.sim.slice.nudge_thickness(step, max_z);
                        true
                    }
                    Key::Character(c) if c == "," => {
                        rt.sim.slice.nudge_zoom(-0.04 * step);
                        true
                    }
                    Key::Character(c) if c == "." => {
                        rt.sim.slice.nudge_zoom(0.04 * step);
                        true
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        rt.sim.slice.nudge_pan(-0.03 * step, 0.0);
                        true
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        rt.sim.slice.nudge_pan(0.03 * step, 0.0);
                        true
                    }
                    Key::Named(NamedKey::ArrowUp) => {
                        rt.sim.slice.nudge_pan(0.0, -0.03 * step);
                        true
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        rt.sim.slice.nudge_pan(0.0, 0.03 * step);
                        true
                    }
                    _ => false,
                };
                if slice_key {
                    rt.sim.uniforms.slice_z = rt.sim.slice.z;
                    let (kind, title, old, value) = match &logical_key {
                        Key::Character(c) if c == "[" || c == "]" => (
                            NudgeKind::SliceZ,
                            "SLICE Z".into(),
                            old_z,
                            fmt_slice_z(rt.sim.slice.z),
                        ),
                        Key::Character(c) if c == ";" || c == "'" => (
                            NudgeKind::Thick,
                            "THICK".into(),
                            old_thick,
                            fmt_thick(rt.sim.slice.thickness),
                        ),
                        Key::Character(c) if c == "," || c == "." => (
                            NudgeKind::Zoom,
                            "ZOOM".into(),
                            old_zoom,
                            fmt_zoom(rt.sim.slice.zoom),
                        ),
                        _ => (
                            NudgeKind::Pan,
                            "PAN".into(),
                            old_pan,
                            fmt_pan(rt.sim.slice.ox, rt.sim.slice.oy),
                        ),
                    };
                    fire_nudge(rt, kind, title, old, value);
                    rt.window.set_title(&format!(
                        "Pycelium 3D | slice Z {:.0}  thick {:.0}  field {:.0}%",
                        rt.sim.slice.z,
                        rt.sim.slice.thickness,
                        rt.sim.slice.zoom * 100.0
                    ));
                    rt.window.request_redraw();
                    return;
                }
                let param_nudge = match &logical_key {
                    Key::Character(c) if c == "-" || c == "_" => Some(-0.04),
                    Key::Character(c) if c == "=" || c == "+" => Some(0.04),
                    _ => None,
                };
                if let Some(delta) = param_nudge {
                    let slot = rt.sim.param_slot;
                    let old = fmt_param(slot, rt.sim.uniforms.param_value(slot));
                    rt.sim.adjust_param(delta);
                    let new = fmt_param(slot, rt.sim.uniforms.param_value(slot));
                    fire_nudge(rt, NudgeKind::Param, param_title(slot), old, new);
                    rt.window.request_redraw();
                    return;
                }
                let radius_nudge = match &logical_key {
                    Key::Character(c) if c == "9" => Some(if rt.shift { -2.4 } else { -0.8 }),
                    Key::Character(c) if c == "0" => Some(if rt.shift { 2.4 } else { 0.8 }),
                    _ => None,
                };
                if let Some(delta) = radius_nudge {
                    rt.brush.nudge_radius(delta);
                    fire_brush_toast(rt);
                    rt.window.request_redraw();
                    return;
                }
                if repeat {
                    return;
                }
                match logical_key {
                    Key::Named(NamedKey::Escape) => {
                        if rt.cutter.active && rt.export.panel_open {
                            rt.export.close_panel();
                            println!("export settings closed");
                            rt.window.request_redraw();
                        } else if rt.cutter.active {
                            rt.cutter.leave();
                            rt.export.close_panel();
                            println!("capture off");
                            rt.window.request_redraw();
                        } else {
                            event_loop.exit();
                        }
                    }
                    Key::Named(NamedKey::Enter) => {
                        if rt.cutter.active {
                            if let Err(err) = export_slice(rt) {
                                eprintln!("slice export failed: {err:#}");
                            }
                        }
                    }
                    Key::Named(NamedKey::Tab) => {
                        rt.labels = rt.labels.cycle();
                        println!("HUD labels: {}", rt.labels.as_str());
                        rt.window.request_redraw();
                    }
                    Key::Character(c) if c.eq_ignore_ascii_case("e") => {
                        rt.cutter.toggle(rt.sim.slice.z, rt.sim.depth);
                        if rt.cutter.active {
                            println!("{}", rt.cutter.describe());
                            println!("{}", rt.export.describe());
                            println!(
                                "C cycle 2D/rich | S settings | X axis | hover face center/mid-edge to snap | M heightmap | Y height axis | H scheme | Shift+wheel thickness"
                            );
                        } else {
                            rt.export.close_panel();
                            println!("capture off");
                        }
                        rt.window.request_redraw();
                    }
                    Key::Character(c) if c.eq_ignore_ascii_case("s") && rt.cutter.active => {
                        rt.export.toggle_panel();
                        println!(
                            "export settings {}  {}",
                            if rt.export.panel_open { "open" } else { "closed" },
                            rt.export.describe()
                        );
                        rt.window.request_redraw();
                    }
                    Key::Character(c) if c.eq_ignore_ascii_case("p") && rt.cutter.active => {
                        rt.export.fit = rt.export.fit.cycle();
                        println!("{}", rt.export.describe());
                        rt.window.request_redraw();
                    }
                    Key::Character(c) if c.eq_ignore_ascii_case("a") && rt.cutter.active => {
                        rt.export.aspect = rt.export.aspect.cycle();
                        println!("{}", rt.export.describe());
                        rt.window.request_redraw();
                    }
                    Key::Character(c) if c.eq_ignore_ascii_case("c") && rt.cutter.active => {
                        rt.cutter.cycle_mode();
                        println!("{}", rt.cutter.describe());
                        rt.window.request_redraw();
                    }
                    Key::Character(c) if c.eq_ignore_ascii_case("x") && rt.cutter.active => {
                        rt.cutter.cycle_axis();
                        println!("{}", rt.cutter.describe());
                        rt.window.request_redraw();
                    }
                    Key::Character(c) if c.eq_ignore_ascii_case("h") => {
                        rt.scheme_visible = !rt.scheme_visible;
                        println!(
                            "control scheme {}",
                            if rt.scheme_visible { "on" } else { "off" }
                        );
                        rt.window.request_redraw();
                    }
                    Key::Character(c) if c.eq_ignore_ascii_case("m") && rt.cutter.active => {
                        rt.export.toggle_format(crate::export_settings::ExportFormat::Heightmap);
                        rt.cutter.export_heightmap = rt.export.write_heightmap;
                        println!("{}", rt.cutter.describe());
                        println!("{}", rt.export.describe());
                    }
                    Key::Character(c) if c.eq_ignore_ascii_case("y") && rt.cutter.active => {
                        rt.cutter.cycle_heightmap_axis();
                        rt.export.write_heightmap = true;
                        rt.cutter.export_heightmap = true;
                        println!("{}", rt.cutter.describe());
                    }
                    Key::Character(c) if c.eq_ignore_ascii_case("t") && !rt.cutter.active => {
                        rt.brush.cycle_mode();
                        fire_brush_toast(rt);
                        println!("tool {}", rt.brush.mode.as_str());
                        rt.window.request_redraw();
                    }
                    Key::Named(NamedKey::Space) => rt.paused = !rt.paused,
                    Key::Character(c) if c.eq_ignore_ascii_case("r") => {
                        rt.sim.reseed(&rt.gpu.queue, &rt.world);
                    }
                    Key::Character(c) if c.eq_ignore_ascii_case("f") => {
                        rt.world.spawn_food();
                        rt.sim.upload_brick(&rt.gpu.queue, &rt.world);
                    }
                    Key::Character(c) if c.eq_ignore_ascii_case("d") => {
                        rt.plan.drift = !rt.plan.drift;
                    }
                    Key::Character(c) => {
                        if let Some(d) = c.chars().next().and_then(|ch| ch.to_digit(10)) {
                            if (1..=8).contains(&d) {
                                let slot = (d - 1) as u8;
                                match rt.brush.mode {
                                    ToolMode::Specimen => {
                                        rt.brush.select_species(slot, &mut rt.sim.uniforms);
                                        fire_brush_toast(rt);
                                        println!(
                                            "species {} lineage {}",
                                            rt.brush.species_preset().name,
                                            rt.brush.species_preset().lineage
                                        );
                                    }
                                    ToolMode::Paint => {
                                        rt.brush.select_channel(PaintChannel::from_index(slot));
                                        fire_brush_toast(rt);
                                        println!("paint {}", rt.brush.channel.name());
                                    }
                                    ToolMode::View => {
                                        rt.sim.param_slot = slot as u32;
                                        let v = fmt_param(
                                            rt.sim.param_slot,
                                            rt.sim.uniforms.param_value(rt.sim.param_slot),
                                        );
                                        fire_nudge(
                                            rt,
                                            NudgeKind::Param,
                                            param_title(rt.sim.param_slot),
                                            v.clone(),
                                            v.clone(),
                                        );
                                        println!(
                                            "param {} = {:.4}",
                                            SimUniforms::param_name(rt.sim.param_slot),
                                            rt.sim.uniforms.param_value(rt.sim.param_slot)
                                        );
                                    }
                                }
                                rt.window.request_redraw();
                            }
                        }
                    }
                    _ => {}
                }
            }
            WindowEvent::RedrawRequested => {
                if rt.pending_pick {
                    rt.has_picked = true;
                    let (eye, target) = rt.orbit.eye_target();
                    let rd = click_dir(rt.cursor, rt.config.width, rt.config.height, eye, target);
                    let vol = [
                        rt.sim.width as f32,
                        rt.sim.height as f32,
                        rt.sim.depth as f32,
                    ];
                    rt.sim.set_pick_ray(
                        [eye[0] * vol[0], eye[1] * vol[1], eye[2] * vol[2]],
                        rd,
                    );
                }

                if !rt.paused {
                    rt.sim.step(
                        &rt.gpu.device,
                        &rt.gpu.queue,
                        rt.plan.steps_per_frame,
                        rt.pending_pick,
                    );
                    if rt.plan.drift && rt.sim.tick() % 24 == 0 {
                        rt.world.drift(rt.sim.width, rt.sim.height, rt.sim.depth);
                        rt.sim.upload_brick(&rt.gpu.queue, &rt.world);
                    }
                } else if rt.pending_pick {
                    rt.sim.step(&rt.gpu.device, &rt.gpu.queue, 1, true);
                }
                rt.pending_pick = false;

                match acquire_frame(&rt.gpu, &mut rt.surface, &rt.config, &rt.window) {
                    Some(frame) => {
                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());
                        let (eye, target) = rt.orbit.eye_target();
                        let fade_target = if rt.labels == LabelDensity::Off {
                            0.0
                        } else {
                            1.0
                        };
                        rt.label_fade += (fade_target - rt.label_fade) * 0.22;
                        if (rt.label_fade - fade_target).abs() < 0.01 {
                            rt.label_fade = fade_target;
                        }
                        if rt.cutter.active {
                            rt.window.set_title(&format!(
                                "Pycelium 3D | {}  {}",
                                rt.cutter.describe(),
                                rt.export.describe()
                            ));
                        }
                        let help_target = if rt.scheme_visible { 1.0 } else { 0.0 };
                        rt.scheme_fade += (help_target - rt.scheme_fade) * 0.28;
                        if (rt.scheme_fade - help_target).abs() < 0.01 {
                            rt.scheme_fade = help_target;
                        }
                        let (hover, anchor) = teach::hit_test(
                            rt.cursor,
                            rt.scheme_visible,
                            rt.cutter.active,
                            rt.export.panel_open,
                        );
                        if hover != HoverId::None && hover != rt.last_hover {
                            rt.tip_fade *= 0.18;
                            rt.last_hover = hover;
                        } else if hover != HoverId::None {
                            rt.last_hover = hover;
                        }
                        let tip_target = if hover == HoverId::None { 0.0 } else { 1.0 };
                        rt.tip_fade += (tip_target - rt.tip_fade) * 0.30;
                        if (rt.tip_fade - tip_target).abs() < 0.01 {
                            rt.tip_fade = tip_target;
                        }
                        if rt.tip_fade <= 0.01 && hover == HoverId::None {
                            rt.last_hover = HoverId::None;
                        }
                        let toast = current_nudge(rt);
                        if toast.is_none() {
                            rt.nudge = None;
                        }
                        let overlay = teach::pack_overlay(
                            rt.scheme_visible || rt.scheme_fade > 0.01,
                            rt.export.panel_open,
                            rt.cutter.active,
                            if rt.tip_fade > 0.01 {
                                rt.last_hover
                            } else {
                                HoverId::None
                            },
                            rt.cursor,
                            anchor,
                            rt.scheme_fade,
                            rt.tip_fade,
                            toast.as_ref(),
                        );
                        let brush_hit = brush_hit_vec(rt, eye, target);
                        rt.sim.render(
                            &rt.gpu.device,
                            &rt.gpu.queue,
                            &view,
                            rt.config.width,
                            rt.config.height,
                            eye,
                            target,
                            rt.fps,
                            [rt.cursor.0, rt.cursor.1],
                            rt.labels.as_f32(),
                            rt.label_fade,
                            rt.has_picked,
                            rt.cutter.present_vec(),
                            rt.export.present_vec(),
                            &overlay,
                            rt.brush.present_vec(),
                            brush_hit,
                        );
                        rt.window.pre_present_notify();
                        rt.gpu.queue.present(frame);
                    }
                    None => {
                        rt.window.request_redraw();
                        return;
                    }
                }

                rt.frames += 1;
                let dt = rt.fps_mark.elapsed().as_secs_f32();
                if dt >= 0.4 {
                    rt.fps = rt.frames as f32 / dt;
                    rt.frames = 0;
                    rt.fps_mark = Instant::now();
                    if !rt.cutter.active {
                        rt.window.set_title(&format!(
                            "Pycelium 3D | {:.0} FPS | {}x{}x{} | {:.2} GiB | {}",
                            rt.fps,
                            rt.plan.gpu_width,
                            rt.plan.gpu_height,
                            rt.plan.gpu_depth,
                            rt.world.committed_bytes() as f64 / 1024.0 / 1024.0 / 1024.0,
                            rt.gpu.describe()
                        ));
                    }
                }
                rt.window.request_redraw();
            }
            WindowEvent::Occluded(false) => rt.window.request_redraw(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(rt) = &self.runtime {
            if !rt.paused
                || rt.nudge.is_some()
                || rt.scheme_fade > 0.01
                || rt.tip_fade > 0.01
                || rt.brush.stroking
                || rt.brush.mode != crate::species::ToolMode::View
            {
                rt.window.request_redraw();
            }
        }
    }
}

fn export_slice(rt: &Runtime) -> anyhow::Result<()> {
    let vol = rt
        .sim
        .read_volume_fields(&rt.gpu.device, &rt.gpu.queue)?;
    let plane = extract_slice(&vol, &rt.cutter, rt.cutter.mode == crate::cutter::CaptureMode::RichBox);
    let composed = apply_export_layout(&plane, &rt.export);
    let stamp = utc_stamp(SystemTime::now());
    let paths = write_exports_with_meta(
        &rt.export_dir,
        &composed,
        &rt.cutter,
        (rt.sim.width, rt.sim.height, rt.sim.depth),
        &stamp,
        Some(&rt.export),
        (plane.width, plane.height),
    )?;
    println!(
        "exported {}  {}  {}×{}  {}",
        paths.png.display(),
        rt.export.describe(),
        composed.width,
        composed.height,
        paths.json.display()
    );
    if let Some(h) = paths.height {
        println!("heightmap {}", h.display());
    }
    Ok(())
}

fn fmt_param(slot: u32, value: f32) -> String {
    match slot % 8 {
        4 | 5 => format!("{value:.4}"),
        _ => format!("{value:.2}"),
    }
}

fn fmt_slice_z(z: f32) -> String {
    format!("{z:.0}")
}

fn fmt_thick(t: f32) -> String {
    format!("{t:.0}")
}

fn fmt_zoom(z: f32) -> String {
    format!("{:.0}", z * 100.0)
}

fn fmt_pan(ox: f32, oy: f32) -> String {
    format!("{:.0}  {:.0}", ox * 100.0, oy * 100.0)
}

fn param_title(slot: u32) -> String {
    format!(
        "{} {}",
        slot % 8 + 1,
        SimUniforms::param_name(slot).replace('_', " ")
    )
}

fn fire_nudge(rt: &mut Runtime, kind: NudgeKind, title: String, old: String, new: String) {
    rt.nudge = Some(LiveNudge {
        kind,
        title,
        old,
        new,
        last: Instant::now(),
    });
}

fn current_nudge(rt: &Runtime) -> Option<NudgeToast> {
    let n = rt.nudge.as_ref()?;
    let idle = n.last.elapsed().as_secs_f32();
    let fade = if idle < 1.15 {
        1.0
    } else {
        (1.0 - (idle - 1.15) / 0.45).clamp(0.0, 1.0)
    };
    if fade <= 0.004 {
        return None;
    }
    let value = if n.old == n.new {
        n.new.clone()
    } else {
        format!("{} TO {}", n.old, n.new)
    };
    let row_y = match n.kind {
        NudgeKind::Param => teach::param_row_mid(rt.sim.param_slot),
        other => other.row_y(),
    };
    Some(NudgeToast {
        kind: n.kind,
        title: n.title.to_ascii_uppercase(),
        value,
        keys: n.kind.keys().to_string(),
        fade,
        row_y,
    })
}

fn apply_hud_slider(rt: &mut Runtime, slider: HudSlider, x: f32, start: &str) {
    let t = teach::slider_t(x);
    let max_z = rt.sim.depth as f32;
    match slider {
        HudSlider::SliceZ => {
            rt.sim.slice.set_depth_normalized(t, max_z);
            rt.sim.uniforms.slice_z = rt.sim.slice.z;
            fire_nudge(
                rt,
                NudgeKind::SliceZ,
                "SLICE Z".into(),
                start.to_string(),
                fmt_slice_z(rt.sim.slice.z),
            );
        }
        HudSlider::Thick => {
            rt.sim.slice.set_thickness_normalized(t, max_z);
            fire_nudge(
                rt,
                NudgeKind::Thick,
                "THICK".into(),
                start.to_string(),
                fmt_thick(rt.sim.slice.thickness),
            );
        }
        HudSlider::Zoom => {
            rt.sim.slice.set_zoom_normalized(t);
            fire_nudge(
                rt,
                NudgeKind::Zoom,
                "ZOOM".into(),
                start.to_string(),
                fmt_zoom(rt.sim.slice.zoom),
            );
        }
        HudSlider::Param(slot) => {
            rt.sim.set_param_normalized(slot as u32, t);
            fire_nudge(
                rt,
                NudgeKind::Param,
                param_title(rt.sim.param_slot),
                start.to_string(),
                fmt_param(rt.sim.param_slot, rt.sim.uniforms.param_value(rt.sim.param_slot)),
            );
        }
    }
}

fn apply_strip_hit(rt: &mut Runtime, hit: StripHit) {
    match hit {
        StripHit::Species(id) => {
            rt.brush.select_species(id, &mut rt.sim.uniforms);
            fire_brush_toast(rt);
            println!(
                "species {} lineage {}",
                rt.brush.species_preset().name,
                rt.brush.species_preset().lineage
            );
        }
        StripHit::SpecimenMode => {
            rt.brush.mode = ToolMode::Specimen;
            rt.brush.apply_species(&mut rt.sim.uniforms);
            rt.brush.stroking = false;
            rt.brush.last_stamp = None;
            fire_brush_toast(rt);
        }
        StripHit::PaintMode => {
            rt.brush.mode = ToolMode::Paint;
            rt.brush.stroking = false;
            rt.brush.last_stamp = None;
            fire_brush_toast(rt);
        }
    }
}

fn fire_brush_toast(rt: &mut Runtime) {
    fire_nudge(
        rt,
        NudgeKind::Brush,
        rt.brush.toast_title(),
        rt.brush.toast_value(),
        rt.brush.toast_value(),
    );
}

fn try_stamp_brush(rt: &mut Runtime) {
    if rt.cutter.active || rt.brush.mode == ToolMode::View {
        return;
    }
    if teach::hud_slider_at(rt.cursor).is_some() || strip_hit(rt.cursor).is_some() {
        return;
    }
    if rt.scheme_visible && teach::in_rect(rt.cursor, teach::scheme_rect(false)) {
        return;
    }
    let (eye, target) = rt.orbit.eye_target();
    let rd = click_dir(rt.cursor, rt.config.width, rt.config.height, eye, target);
    let Some(mid) = ray_cube_midpoint(eye, rd) else {
        return;
    };
    let vol = [
        rt.sim.width as f32,
        rt.sim.height as f32,
        rt.sim.depth as f32,
    ];
    let center = [mid[0] * vol[0], mid[1] * vol[1], mid[2] * vol[2]];
    if !rt.brush.should_stamp(center) {
        return;
    }
    rt.brush.last_stamp = Some(center);
    rt.brush.erase = rt.alt;
    let lineage = rt.brush.species_preset().lineage;
    match rt.brush.mode {
        ToolMode::Specimen => {
            let mode = if rt.brush.erase { 2 } else { 1 };
            rt.sim.stamp_brush(
                &rt.gpu.device,
                &rt.gpu.queue,
                center,
                rt.brush.radius,
                rt.brush.strength,
                0,
                lineage,
                mode,
                false,
            );
        }
        ToolMode::Paint => {
            let subtract = rt.brush.erase || rt.brush.channel == PaintChannel::Void;
            let mode = if subtract { 4 } else { 3 };
            let kill = rt.brush.channel == PaintChannel::Void;
            rt.sim.stamp_brush(
                &rt.gpu.device,
                &rt.gpu.queue,
                center,
                rt.brush.radius,
                rt.brush.strength,
                rt.brush.channel.index() as u32,
                0,
                mode,
                kill,
            );
        }
        ToolMode::View => {}
    }
    fire_brush_toast(rt);
}

fn brush_hit_vec(rt: &Runtime, eye: [f32; 3], target: [f32; 3]) -> [f32; 4] {
    if rt.brush.mode == ToolMode::View || rt.cutter.active {
        return [0.0, 0.0, 0.0, 0.0];
    }
    let rd = click_dir(rt.cursor, rt.config.width, rt.config.height, eye, target);
    match ray_cube_midpoint(eye, rd) {
        Some(p) => [p[0], p[1], p[2], 1.0],
        None => [0.0, 0.0, 0.0, 0.0],
    }
}

fn click_dir(
    cursor: (f32, f32),
    out_w: u32,
    out_h: u32,
    eye: [f32; 3],
    target: [f32; 3],
) -> [f32; 3] {
    let aspect = out_w as f32 / out_h.max(1) as f32;
    let ndc = [(cursor.0 * 2.0 - 1.0) * aspect, -(cursor.1 * 2.0 - 1.0)];
    let fwd = norm([
        target[0] - eye[0],
        target[1] - eye[1],
        target[2] - eye[2],
    ]);
    let right = norm(cross(fwd, [0.0, 0.0, 1.0]));
    let up = norm(cross(right, fwd));
    norm([
        fwd[0] + right[0] * ndc[0] * 0.7 + up[0] * ndc[1] * 0.7,
        fwd[1] + right[1] * ndc[0] * 0.7 + up[1] * ndc[1] * 0.7,
        fwd[2] + right[2] * ndc[0] * 0.7 + up[2] * ndc[1] * 0.7,
    ])
}

fn norm(v: [f32; 3]) -> [f32; 3] {
    let m = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
    [v[0] / m, v[1] / m, v[2] / m]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn choose_present_mode(modes: &[wgpu::PresentMode], vsync: bool) -> wgpu::PresentMode {
    if vsync {
        return wgpu::PresentMode::AutoVsync;
    }
    if modes.contains(&wgpu::PresentMode::Immediate) {
        wgpu::PresentMode::Immediate
    } else if modes.contains(&wgpu::PresentMode::Mailbox) {
        wgpu::PresentMode::Mailbox
    } else {
        wgpu::PresentMode::AutoNoVsync
    }
}

fn acquire_frame(
    gpu: &GpuDevice,
    surface: &mut wgpu::Surface<'static>,
    config: &wgpu::SurfaceConfiguration,
    window: &Arc<Window>,
) -> Option<wgpu::SurfaceTexture> {
    match surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(frame) => Some(frame),
        wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => None,
        wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
            drop(frame);
            surface.configure(&gpu.device, config);
            None
        }
        wgpu::CurrentSurfaceTexture::Outdated => {
            surface.configure(&gpu.device, config);
            None
        }
        wgpu::CurrentSurfaceTexture::Lost => {
            if let Ok(next) = gpu.instance.create_surface(window.clone()) {
                *surface = next;
                surface.configure(&gpu.device, config);
            }
            None
        }
        wgpu::CurrentSurfaceTexture::Validation => panic!("surface validation error"),
    }
}

mod config;
mod cutter;
mod export;
mod gpu;
mod memory;
mod model;
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
use crate::cutter::Cutter;
use crate::export::{extract_slice, utc_stamp, write_exports};
use crate::gpu::{GpuDevice, MyceliumGpu};
use crate::memory::HostWorld;
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
    export_dir: PathBuf,
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
                export_dir,
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
                    "Tab HUD labels ({}) | hover a meter to fade English | glyphs stay",
                    self.runtime.as_ref().map(|r| r.labels.as_str()).unwrap_or("rich")
                );
                println!(
                    "E capture | C 2D/rich | hover cube face to snap | drag slider handles | Enter export | Esc leave"
                );
                println!(
                    "HUD: FPS, tips, fusions, branches, C:N | bars | then slice Z / thickness / zoom% | selected tip"
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
                if rt.cutter.active {
                    let axis_n = rt.cutter.axis.size(rt.sim.width, rt.sim.height, rt.sim.depth)
                        as f32;
                    if rt.cutter.dragging.is_some() {
                        rt.cutter.drag_handle(rt.cursor, axis_n);
                    } else if !rt.orbit.dragging {
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
                if button == MouseButton::Left {
                    if state == ElementState::Pressed && rt.cutter.active {
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
                if rt.cutter.active && rt.shift {
                    let axis_n = rt.cutter.axis.size(rt.sim.width, rt.sim.height, rt.sim.depth)
                        as f32;
                    rt.cutter.nudge_half(steps * if rt.shift { 2.0 } else { 1.0 }, axis_n);
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
                if state != ElementState::Pressed {
                    return;
                }
                let max_z = rt.sim.depth as f32;
                let step = if rt.shift { 8.0 } else { 1.0 };
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
                    rt.window.set_title(&format!(
                        "Pycelium 3D | slice Z {:.0}  thick {:.0}  field {:.0}%",
                        rt.sim.slice.z,
                        rt.sim.slice.thickness,
                        rt.sim.slice.zoom * 100.0
                    ));
                    rt.window.request_redraw();
                    return;
                }
                if repeat {
                    return;
                }
                match logical_key {
                    Key::Named(NamedKey::Escape) => {
                        if rt.cutter.active {
                            rt.cutter.leave();
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
                            println!(
                                "C cycle 2D/rich | X axis | hover face center/mid-edge to snap | H heightmap | Y height axis | Shift+wheel thickness"
                            );
                        } else {
                            println!("capture off");
                        }
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
                    Key::Character(c) if c.eq_ignore_ascii_case("h") && rt.cutter.active => {
                        rt.cutter.export_heightmap = !rt.cutter.export_heightmap;
                        println!("{}", rt.cutter.describe());
                    }
                    Key::Character(c) if c.eq_ignore_ascii_case("y") && rt.cutter.active => {
                        rt.cutter.cycle_heightmap_axis();
                        println!("{}", rt.cutter.describe());
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
                    Key::Character(c) if c == "-" || c == "_" => rt.sim.adjust_param(-0.04),
                    Key::Character(c) if c == "=" || c == "+" => rt.sim.adjust_param(0.04),
                    Key::Character(c) => {
                        if let Some(d) = c.chars().next().and_then(|ch| ch.to_digit(10)) {
                            if (1..=8).contains(&d) {
                                rt.sim.param_slot = d - 1;
                                println!(
                                    "param {} = {:.4}",
                                    SimUniforms::param_name(rt.sim.param_slot),
                                    rt.sim.uniforms.param_value(rt.sim.param_slot)
                                );
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
                                "Pycelium 3D | {}",
                                rt.cutter.describe()
                            ));
                        }
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
            if !rt.paused {
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
    let stamp = utc_stamp(SystemTime::now());
    let paths = write_exports(
        &rt.export_dir,
        &plane,
        &rt.cutter,
        (rt.sim.width, rt.sim.height, rt.sim.depth),
        &stamp,
    )?;
    println!(
        "exported {}  {}  {}  {}",
        paths.png.display(),
        paths.json.display(),
        paths.svg.display(),
        paths.mask.display()
    );
    if let Some(h) = paths.height {
        println!("heightmap {}", h.display());
    }
    Ok(())
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

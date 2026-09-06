use anyhow::{Context, Result};
use wgpu::util::DeviceExt;

use crate::config::MemoryPlan;
use crate::export::VolumeFields;
use crate::hud_font;
use crate::memory::HostWorld;
use crate::types::{PresentUniforms, SimUniforms, SliceView, TEL_COUNT, Tip};

const TIP_WG: u32 = 64;
const VOL_X: u32 = 8;
const VOL_Y: u32 = 8;
const VOL_Z: u32 = 4;

pub struct GpuDevice {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub info: wgpu::AdapterInfo,
}

impl GpuDevice {
    pub async fn request(
        instance: wgpu::Instance,
        surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: surface,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .context("no compatible GPU adapter")?;
        let info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("pycelium-win"),
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .context("failed to create GPU device")?;
        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            info,
        })
    }

    pub fn describe(&self) -> String {
        format!("{} ({:?})", self.info.name, self.info.backend)
    }
}

pub struct MyceliumGpu {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub tip_count: u32,
    pub uniforms: SimUniforms,
    pub slice: SliceView,
    pub param_slot: u32,
    uniform_buf: wgpu::Buffer,
    present_buf: wgpu::Buffer,
    tip_buf: wgpu::Buffer,
    organic: wgpu::Buffer,
    soluble_c: wgpu::Buffer,
    soluble_n: wgpu::Buffer,
    moisture: wgpu::Buffer,
    autocrine: wgpu::Buffer,
    biomass: wgpu::Buffer,
    internal_c: wgpu::Buffer,
    enzyme: wgpu::Buffer,
    #[allow(dead_code)]
    scratch: wgpu::Buffer,
    tel: wgpu::Buffer,
    kinetics: wgpu::ComputePipeline,
    diffuse: wgpu::ComputePipeline,
    scatter_field: wgpu::ComputePipeline,
    translocate: wgpu::ComputePipeline,
    scatter_internal: wgpu::ComputePipeline,
    grow: wgpu::ComputePipeline,
    hud_reduce: wgpu::ComputePipeline,
    pick: wgpu::ComputePipeline,
    sim_bg: wgpu::BindGroup,
    present_pipeline: wgpu::RenderPipeline,
    present_bg: wgpu::BindGroup,
    #[allow(dead_code)]
    font_tex: wgpu::Texture,
    #[allow(dead_code)]
    font_samp: wgpu::Sampler,
    tick: u32,
    last_pick: u32,
}

impl MyceliumGpu {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        plan: &MemoryPlan,
        world: &HostWorld,
        surface_format: wgpu::TextureFormat,
    ) -> Result<Self> {
        let width = plan.gpu_width;
        let height = plan.gpu_height;
        let depth = plan.gpu_depth;
        let tip_count = plan.gpu_agents;
        let cells = plan.gpu_cells() as u64;
        let uniforms = SimUniforms::new(width, height, depth, tip_count);

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sim-uniforms"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let present_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("present-uniforms"),
            size: std::mem::size_of::<PresentUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let tips = gpu_tip_slice(&world.spores, tip_count as usize);
        let tip_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("tips"),
            contents: bytemuck::cast_slice(&tips),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let brick = world.extract_brick(width, height, depth);
        let field = |name: &'static str, data: &[f32]| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(name),
                contents: bytemuck::cast_slice(data),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            })
        };
        let zeros = vec![0f32; cells as usize];
        let organic = field("organic", &brick.organic);
        let soluble_c = field("soluble_c", &brick.soluble_c);
        let soluble_n = field("soluble_n", &brick.soluble_n);
        let moisture = field("moisture", &brick.moisture);
        let autocrine = field("autocrine", &zeros);
        let biomass = field("biomass", &zeros);
        let internal_c = field("internal_c", &zeros);
        let enzyme = field("enzyme", &zeros);
        let scratch = field("scratch", &zeros);

        let tel = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("telemetry"),
            size: (TEL_COUNT * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sim-layout"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::COMPUTE),
                storage_entry(1, wgpu::ShaderStages::COMPUTE, false),
                storage_entry(2, wgpu::ShaderStages::COMPUTE, false),
                storage_entry(3, wgpu::ShaderStages::COMPUTE, false),
                storage_entry(4, wgpu::ShaderStages::COMPUTE, false),
                storage_entry(5, wgpu::ShaderStages::COMPUTE, false),
                storage_entry(6, wgpu::ShaderStages::COMPUTE, false),
                storage_entry(7, wgpu::ShaderStages::COMPUTE, false),
                storage_entry(8, wgpu::ShaderStages::COMPUTE, false),
                storage_entry(9, wgpu::ShaderStages::COMPUTE, false),
                storage_entry(10, wgpu::ShaderStages::COMPUTE, false),
                storage_entry(11, wgpu::ShaderStages::COMPUTE, false),
            ],
        });
        let present_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("present-layout"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::FRAGMENT),
                storage_entry(1, wgpu::ShaderStages::FRAGMENT, true),
                storage_entry(2, wgpu::ShaderStages::FRAGMENT, true),
                storage_entry(3, wgpu::ShaderStages::FRAGMENT, true),
                storage_entry(4, wgpu::ShaderStages::FRAGMENT, true),
                storage_entry(5, wgpu::ShaderStages::FRAGMENT, true),
                storage_entry(6, wgpu::ShaderStages::FRAGMENT, true),
                storage_entry(7, wgpu::ShaderStages::FRAGMENT, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let atlas = hud_font::rasterize_atlas();
        let font_tex = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("hud-font"),
                size: wgpu::Extent3d {
                    width: atlas.width,
                    height: atlas.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &atlas.pixels,
        );
        let font_view = font_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let font_samp = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("hud-font-samp"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let sim_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sim"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sim.wgsl").into()),
        });
        let present_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("present"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/present.wgsl").into()),
        });

        let sim_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sim-pl"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let present_pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("present-pl"),
            bind_group_layouts: &[Some(&present_layout)],
            immediate_size: 0,
        });

        let compute = |entry: &'static str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(&sim_pl),
                module: &sim_shader,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        };

        let present_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("present"),
            layout: Some(&present_pl),
            vertex: wgpu::VertexState {
                module: &present_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &present_shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(surface_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let sim_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sim-bg"),
            layout: &layout,
            entries: &[
                bind(&uniform_buf, 0),
                bind(&tip_buf, 1),
                bind(&organic, 2),
                bind(&soluble_c, 3),
                bind(&soluble_n, 4),
                bind(&moisture, 5),
                bind(&autocrine, 6),
                bind(&biomass, 7),
                bind(&internal_c, 8),
                bind(&enzyme, 9),
                bind(&tel, 10),
                bind(&scratch, 11),
            ],
        });
        let present_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("present-bg"),
            layout: &present_layout,
            entries: &[
                bind(&present_buf, 0),
                bind(&biomass, 1),
                bind(&internal_c, 2),
                bind(&soluble_c, 3),
                bind(&enzyme, 4),
                bind(&organic, 5),
                bind(&tel, 6),
                bind(&tip_buf, 7),
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(&font_view),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::Sampler(&font_samp),
                },
            ],
        });

        let _ = queue;
        Ok(Self {
            width,
            height,
            depth,
            tip_count,
            uniforms,
            slice: SliceView::new(depth),
            param_slot: 0,
            uniform_buf,
            present_buf,
            tip_buf,
            organic,
            soluble_c,
            soluble_n,
            moisture,
            autocrine,
            biomass,
            internal_c,
            enzyme,
            scratch,
            tel,
            kinetics: compute("kinetics"),
            diffuse: compute("diffuse"),
            scatter_field: compute("scatter_field"),
            translocate: compute("translocate"),
            scatter_internal: compute("scatter_internal"),
            grow: compute("grow"),
            hud_reduce: compute("hud_reduce"),
            pick: compute("pick"),
            sim_bg,
            present_pipeline,
            present_bg,
            font_tex,
            font_samp,
            tick: 0,
            last_pick: 0,
        })
    }

    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn step(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, steps: u32, do_pick: bool) {
        for _ in 0..steps {
            self.uniforms.time = self.tick as f32 * 0.016;
            self.uniforms.seed = self.uniforms.seed.wrapping_add(1);
            self.uniforms.pick_active = if do_pick { 1.0 } else { 0.0 };
            self.clear_telemetry(queue);
            self.dispatch_named(device, queue, "kinetics");

            for (mode, decay) in [(0.0, 0.15), (1.0, 0.15), (2.0, 0.12), (3.0, 0.145)] {
                self.uniforms.field_decay = mode + decay;
                queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&self.uniforms));
                self.dispatch_named(device, queue, "diffuse");
                self.dispatch_named(device, queue, "scatter_field");
            }

            self.uniforms.field_decay = 1.0;
            queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&self.uniforms));
            self.dispatch_named(device, queue, "translocate");
            self.dispatch_named(device, queue, "scatter_internal");
            self.dispatch_named(device, queue, "grow");
            if self.tick % 8 == 0 {
                self.dispatch_named(device, queue, "hud_reduce");
            }
            if do_pick {
                self.dispatch_named(device, queue, "pick");
            }
            self.tick = self.tick.wrapping_add(1);
        }
    }

    pub fn set_pick_ray(&mut self, origin: [f32; 3], dir: [f32; 3]) {
        self.uniforms.pick_ox = origin[0];
        self.uniforms.pick_oy = origin[1];
        self.uniforms.pick_oz = origin[2];
        self.uniforms.pick_dx = dir[0];
        self.uniforms.pick_dy = dir[1];
        self.uniforms.pick_dz = dir[2];
    }

    pub fn adjust_param(&mut self, delta: f32) {
        self.uniforms.adjust(self.param_slot, delta);
    }

    pub fn upload_brick(&self, queue: &wgpu::Queue, world: &HostWorld) {
        let brick = world.extract_brick(self.width, self.height, self.depth);
        queue.write_buffer(&self.organic, 0, bytemuck::cast_slice(&brick.organic));
        queue.write_buffer(&self.soluble_c, 0, bytemuck::cast_slice(&brick.soluble_c));
        queue.write_buffer(&self.soluble_n, 0, bytemuck::cast_slice(&brick.soluble_n));
        queue.write_buffer(&self.moisture, 0, bytemuck::cast_slice(&brick.moisture));
    }

    pub fn reseed(&mut self, queue: &wgpu::Queue, world: &HostWorld) {
        let tips = gpu_tip_slice(&world.spores, self.tip_count as usize);
        queue.write_buffer(&self.tip_buf, 0, bytemuck::cast_slice(&tips));
        let zeros = vec![0f32; (self.width * self.height * self.depth) as usize];
        queue.write_buffer(&self.autocrine, 0, bytemuck::cast_slice(&zeros));
        queue.write_buffer(&self.biomass, 0, bytemuck::cast_slice(&zeros));
        queue.write_buffer(&self.internal_c, 0, bytemuck::cast_slice(&zeros));
        queue.write_buffer(&self.enzyme, 0, bytemuck::cast_slice(&zeros));
        self.upload_brick(queue, world);
        self.tick = 0;
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        out_w: u32,
        out_h: u32,
        eye: [f32; 3],
        target: [f32; 3],
        fps: f32,
        cursor: [f32; 2],
        label_density: f32,
        label_fade: f32,
        has_picked: bool,
        cutter: [f32; 4],
    ) {
        let present = PresentUniforms {
            width: self.width,
            height: self.height,
            depth: self.depth,
            out_w,
            out_h,
            _pad0: 0,
            slice_z: self.slice.z,
            fps,
            eye: [eye[0], eye[1], eye[2], 0.0],
            target: [target[0], target[1], target[2], 0.0],
            live_tips: 0.0,
            fusions: 0.0,
            branches: 0.0,
            cn_ratio: 0.0,
            biomass: 0.0,
            internal_c: 0.0,
            enzyme: 0.0,
            organic: 0.0,
            selected_id: if has_picked { 1.0 } else { 0.0 },
            selected_lineage: 0.0,
            selected_age: 0.0,
            selected_reserve: 0.0,
            param_slot: self.param_slot as f32,
            param_value: self.uniforms.param_value(self.param_slot),
            soluble_c: 0.0,
            soluble_n: 0.0,
            slice_thickness: self.slice.thickness,
            slice_zoom: self.slice.zoom,
            slice_ox: self.slice.ox,
            slice_oy: self.slice.oy,
            hud_ui: [cursor[0], cursor[1], label_density, label_fade],
            cutter,
        };
        queue.write_buffer(&self.present_buf, 0, bytemuck::bytes_of(&present));
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("present"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("present"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.015,
                            g: 0.018,
                            b: 0.022,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.present_pipeline);
            pass.set_bind_group(0, &self.present_bg, &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit(Some(encoder.finish()));
    }

    /// Copy biomass + soluble C back to the host for a slice bake.
    pub fn read_volume_fields(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<VolumeFields> {
        let cells = (self.width * self.height * self.depth) as u64;
        let bytes = cells * 4;
        let biomass = read_f32_buffer(device, queue, &self.biomass, bytes)?;
        let soluble_c = read_f32_buffer(device, queue, &self.soluble_c, bytes)?;
        Ok(VolumeFields {
            width: self.width,
            height: self.height,
            depth: self.depth,
            biomass,
            soluble_c,
        })
    }

    fn clear_telemetry(&self, queue: &wgpu::Queue) {
        let mut data = [0u32; TEL_COUNT];
        data[10] = u32::MAX;
        data[11] = self.last_pick;
        queue.write_buffer(&self.tel, 0, bytemuck::cast_slice(&data));
    }

    fn dispatch_named(&self, device: &wgpu::Device, queue: &wgpu::Queue, name: &str) {
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&self.uniforms));
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(name),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some(name),
                timestamp_writes: None,
            });
            pass.set_bind_group(0, &self.sim_bg, &[]);
            match name {
                "grow" | "pick" => {
                    pass.set_pipeline(if name == "grow" { &self.grow } else { &self.pick });
                    pass.dispatch_workgroups(self.tip_count.div_ceil(TIP_WG), 1, 1);
                }
                other => {
                    let pipe = match other {
                        "kinetics" => &self.kinetics,
                        "diffuse" => &self.diffuse,
                        "scatter_field" => &self.scatter_field,
                        "translocate" => &self.translocate,
                        "scatter_internal" => &self.scatter_internal,
                        _ => &self.hud_reduce,
                    };
                    pass.set_pipeline(pipe);
                    pass.dispatch_workgroups(
                        self.width.div_ceil(VOL_X),
                        self.height.div_ceil(VOL_Y),
                        self.depth.div_ceil(VOL_Z),
                    );
                }
            }
        }
        queue.submit(Some(encoder.finish()));
    }
}

fn gpu_tip_slice(spores: &[Tip], count: usize) -> Vec<Tip> {
    let mut tips = spores.iter().copied().take(count).collect::<Vec<_>>();
    if tips.len() < count {
        tips.resize(count, Tip::DORMANT);
    }
    tips
}

fn uniform_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn storage_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    read_only: bool,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

#[cfg(test)]
mod shader_tests {
    #[test]
    fn present_wgsl_parses_and_validates() {
        let src = include_str!("shaders/present.wgsl");
        let module = naga::front::wgsl::parse_str(src).expect("parse present.wgsl");
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::default(),
        );
        validator
            .validate(&module)
            .expect("present.wgsl should validate");
    }
}

fn read_f32_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    src: &wgpu::Buffer,
    bytes: u64,
) -> Result<Vec<f32>> {
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("slice-readback"),
        size: bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("slice-readback"),
    });
    encoder.copy_buffer_to_buffer(src, 0, &staging, 0, bytes);
    queue.submit(Some(encoder.finish()));
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .context("poll slice readback")?;
    rx.recv().context("readback callback")??;
    let data = slice.get_mapped_range().context("map slice range")?;
    let floats = bytemuck::cast_slice(data.as_ref()).to_vec();
    drop(data);
    staging.unmap();
    Ok(floats)
}

fn bind(buffer: &wgpu::Buffer, binding: u32) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}

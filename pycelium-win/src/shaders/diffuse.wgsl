struct Uniforms {
    width: u32,
    height: u32,
    agent_count: u32,
    time: f32,
    sensor_angle: f32,
    sensor_distance: f32,
    turn_angle: f32,
    step_size: f32,
    deposit: f32,
    decay: f32,
    nutrient_weight: f32,
    consume: f32,
    branch_threshold: f32,
    seed: u32,
    pad0: u32,
    pad1: u32,
}

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var<storage, read> trail_in: array<f32>;
@group(0) @binding(2) var<storage, read_write> trail_out: array<f32>;
@group(0) @binding(3) var<storage, read_write> deposit: array<atomic<u32>>;

fn wrap_i(v: i32, m: i32) -> i32 {
    return ((v % m) + m) % m;
}

fn cell_index(x: i32, y: i32) -> u32 {
    let w = i32(u.width);
    let h = i32(u.height);
    return u32(wrap_i(y, h) * w + wrap_i(x, w));
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= u.width || gid.y >= u.height {
        return;
    }

    let x = i32(gid.x);
    let y = i32(gid.y);
    var acc = 0.0;
    for (var oy = -1; oy <= 1; oy++) {
        for (var ox = -1; ox <= 1; ox++) {
            acc = acc + trail_in[cell_index(x + ox, y + oy)];
        }
    }
    acc = acc / 9.0;

    let i = gid.y * u.width + gid.x;
    let dep = f32(atomicLoad(&deposit[i])) * 0.001;
    atomicStore(&deposit[i], 0u);
    trail_out[i] = acc * u.decay + dep;
}

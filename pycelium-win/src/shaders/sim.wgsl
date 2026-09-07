struct Tip {
    pos: vec3<f32>,
    age: f32,
    dir: vec3<f32>,
    reserve: f32,
    state: f32,
    lineage: u32,
    parent: u32,
    flags: u32,
}

struct Uniforms {
    width: u32,
    height: u32,
    depth: u32,
    tip_count: u32,
    time: f32,
    sensor_distance: f32,
    branch_angle: f32,
    max_extension: f32,
    min_branch_age: f32,
    km_c: f32,
    km_n: f32,
    uptake_v: f32,
    enzyme_k: f32,
    maintenance: f32,
    yield_c: f32,
    auto_weight: f32,
    chemo_weight: f32,
    nitro_weight: f32,
    persist: f32,
    anastomosis_th: f32,
    branch_cost: f32,
    field_decay: f32,
    slice_z: f32,
    seed: u32,
    pick_ox: f32,
    pick_oy: f32,
    pick_oz: f32,
    pick_active: f32,
    pick_dx: f32,
    pick_dy: f32,
    pick_dz: f32,
    pad2: f32,
    brush_x: f32,
    brush_y: f32,
    brush_z: f32,
    brush_radius: f32,
    brush_strength: f32,
    brush_channel: f32,
    brush_mode: f32,
    brush_lineage: f32,
}

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var<storage, read_write> tips: array<Tip>;
@group(0) @binding(2) var<storage, read_write> organic: array<f32>;
@group(0) @binding(3) var<storage, read_write> soluble_c: array<f32>;
@group(0) @binding(4) var<storage, read_write> soluble_n: array<f32>;
@group(0) @binding(5) var<storage, read_write> moisture: array<f32>;
@group(0) @binding(6) var<storage, read_write> autocrine: array<f32>;
@group(0) @binding(7) var<storage, read_write> biomass: array<f32>;
@group(0) @binding(8) var<storage, read_write> internal_c: array<f32>;
@group(0) @binding(9) var<storage, read_write> enzyme: array<f32>;
@group(0) @binding(10) var<storage, read_write> tel: array<atomic<u32>>;
@group(0) @binding(11) var<storage, read_write> scratch: array<f32>;

fn wrap_i(v: i32, m: i32) -> i32 {
    return ((v % m) + m) % m;
}

fn idx3(x: i32, y: i32, z: i32) -> u32 {
    let w = i32(u.width);
    let h = i32(u.height);
    let d = i32(u.depth);
    let zz = clamp(z, 0, d - 1);
    return u32((zz * h + wrap_i(y, h)) * w + wrap_i(x, w));
}

fn at(p: vec3<f32>, field: u32) -> f32 {
    let i = idx3(i32(floor(p.x)), i32(floor(p.y)), i32(floor(p.z)));
    switch field {
        case 0u: { return soluble_c[i]; }
        case 1u: { return soluble_n[i]; }
        case 2u: { return autocrine[i]; }
        case 3u: { return biomass[i]; }
        default: { return 0.0; }
    }
}

fn grad(p: vec3<f32>, field: u32) -> vec3<f32> {
    let e = u.sensor_distance;
    return vec3<f32>(
        at(p + vec3<f32>(e, 0.0, 0.0), field) - at(p - vec3<f32>(e, 0.0, 0.0), field),
        at(p + vec3<f32>(0.0, e, 0.0), field) - at(p - vec3<f32>(0.0, e, 0.0), field),
        at(p + vec3<f32>(0.0, 0.0, e), field) - at(p - vec3<f32>(0.0, 0.0, e), field),
    );
}

fn pcg(n: u32) -> u32 {
    var h = n * 747796405u + 2891336453u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
    return (h >> 22u) ^ h;
}

fn safe_norm(v: vec3<f32>) -> vec3<f32> {
    let m = length(v);
    if m < 1e-4 {
        return vec3<f32>(0.0);
    }
    return v / m;
}

fn perp(d: vec3<f32>, salt: u32) -> vec3<f32> {
    var a = vec3<f32>(1.0, 0.0, 0.0);
    if abs(d.x) > 0.85 {
        a = vec3<f32>(0.0, 1.0, 0.0);
    }
    let p = safe_norm(cross(d, a));
    let q = safe_norm(cross(d, p));
    let ang = f32(salt % 628u) * 0.01;
    return safe_norm(p * cos(ang) + q * sin(ang));
}

@compute @workgroup_size(8, 8, 4)
fn kinetics(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= u.width || gid.y >= u.height || gid.z >= u.depth {
        return;
    }
    let i = idx3(i32(gid.x), i32(gid.y), i32(gid.z));
    let moist = moisture[i];
    let hydro = u.enzyme_k * enzyme[i] * moist * organic[i] / (u.km_c + organic[i] + 0.001);
    organic[i] = max(organic[i] - hydro, 0.0);
    soluble_c[i] = soluble_c[i] + hydro * 0.90;
    soluble_n[i] = soluble_n[i] + hydro * 0.05;
    enzyme[i] = enzyme[i] * 0.995;
    autocrine[i] = autocrine[i] * 0.968;
    if biomass[i] > 0.018 {
        internal_c[i] = max(internal_c[i] - u.maintenance * biomass[i], 0.0);
        if internal_c[i] < 0.0015 {
            biomass[i] = biomass[i] * 0.9988;
        }
    }
}

@compute @workgroup_size(8, 8, 4)
fn diffuse(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= u.width || gid.y >= u.height || gid.z >= u.depth {
        return;
    }
    let x = i32(gid.x);
    let y = i32(gid.y);
    let z = i32(gid.z);
    let i = idx3(x, y, z);
    var acc = 0.0;
    var wsum = 0.0;
    let pick = u32(u.field_decay) % 4u;
    for (var oz = -1; oz <= 1; oz++) {
        for (var oy = -1; oy <= 1; oy++) {
            for (var ox = -1; ox <= 1; ox++) {
                let j = idx3(x + ox, y + oy, z + oz);
                let mw = 0.18 + 0.82 * moisture[j];
                var v = 0.0;
                switch pick {
                    case 0u: { v = soluble_c[j]; }
                    case 1u: { v = soluble_n[j]; }
                    case 2u: { v = autocrine[j]; }
                    default: { v = enzyme[j]; }
                }
                acc = acc + v * mw;
                wsum = wsum + mw;
            }
        }
    }
    let decay = clamp(fract(u.field_decay) + 0.85, 0.85, 1.0);
    scratch[i] = (acc / max(wsum, 1e-4)) * decay;
}

@compute @workgroup_size(8, 8, 4)
fn scatter_field(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= u.width || gid.y >= u.height || gid.z >= u.depth {
        return;
    }
    let i = idx3(i32(gid.x), i32(gid.y), i32(gid.z));
    let pick = u32(u.field_decay) % 4u;
    let v = scratch[i];
    switch pick {
        case 0u: { soluble_c[i] = v; }
        case 1u: { soluble_n[i] = v; }
        case 2u: { autocrine[i] = v; }
        default: { enzyme[i] = v; }
    }
}

@compute @workgroup_size(8, 8, 4)
fn translocate(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= u.width || gid.y >= u.height || gid.z >= u.depth {
        return;
    }
    let x = i32(gid.x);
    let y = i32(gid.y);
    let z = i32(gid.z);
    let i = idx3(x, y, z);
    let wi = max(biomass[i] - 0.016, 0.0);
    if wi <= 0.0 {
        scratch[i] = internal_c[i] * 0.994;
        return;
    }
    var flux = 0.0;
    let offs = array<vec3<i32>, 6>(
        vec3<i32>(1, 0, 0), vec3<i32>(-1, 0, 0),
        vec3<i32>(0, 1, 0), vec3<i32>(0, -1, 0),
        vec3<i32>(0, 0, 1), vec3<i32>(0, 0, -1),
    );
    for (var n = 0; n < 6; n++) {
        let o = offs[n];
        let j = idx3(x + o.x, y + o.y, z + o.z);
        let wn = max(biomass[j] - 0.016, 0.0);
        let cond = sqrt(wi * wn);
        flux = flux + cond * (internal_c[j] - internal_c[i]);
    }
    scratch[i] = max(internal_c[i] + 0.18 * flux, 0.0);
}

@compute @workgroup_size(8, 8, 4)
fn scatter_internal(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= u.width || gid.y >= u.height || gid.z >= u.depth {
        return;
    }
    let i = idx3(i32(gid.x), i32(gid.y), i32(gid.z));
    internal_c[i] = scratch[i];
}

@compute @workgroup_size(64)
fn grow(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if id >= u.tip_count {
        return;
    }
    var t = tips[id];
    if t.state <= 0.0 {
        return;
    }
    atomicAdd(&tel[0], 1u);

    let p = t.pos;
    let g_c = grad(p, 0u);
    let g_n = grad(p, 1u);
    let g_a = grad(p, 2u);
    let g_b = grad(p, 3u);
    var dir = safe_norm(t.dir);
    if length(dir) < 0.1 {
        dir = vec3<f32>(1.0, 0.0, 0.0);
    }
    let seek_n = t.reserve / (t.reserve + 0.28);
    var steer = dir * u.persist
        + safe_norm(g_c) * u.chemo_weight
        + safe_norm(g_n) * u.nitro_weight * seek_n
        - safe_norm(g_a + g_b * 0.45) * u.auto_weight;
    dir = safe_norm(steer);
    if length(dir) < 0.1 {
        dir = safe_norm(t.dir);
    }

    let step = u.max_extension * clamp(t.reserve / (0.16 + t.reserve), 0.18, 1.0);
    var next = p + dir * step;
    let fw = f32(u.width);
    let fh = f32(u.height);
    let fd = f32(u.depth);
    next.x = next.x - fw * floor(next.x / fw);
    next.y = next.y - fh * floor(next.y / fh);
    if next.x < 0.0 { next.x = next.x + fw; }
    if next.y < 0.0 { next.y = next.y + fh; }
    if next.z < 1.0 {
        next.z = 1.0;
        dir.z = abs(dir.z);
    }
    if next.z > fd - 2.0 {
        next.z = fd - 2.0;
        dir.z = -abs(dir.z);
    }
    t.pos = next;
    t.dir = dir;
    t.age = t.age + 1.0;

    let i = idx3(i32(next.x), i32(next.y), i32(next.z));
    let uc = u.uptake_v * soluble_c[i] / (u.km_c + soluble_c[i] + 0.001);
    let un = u.uptake_v * 0.7 * soluble_n[i] / (u.km_n + soluble_n[i] + 0.001);
    let growth = min(uc, un * 12.0);
    soluble_c[i] = max(soluble_c[i] - uc, 0.0);
    soluble_n[i] = max(soluble_n[i] - un, 0.0);
    t.reserve = clamp(t.reserve + growth * u.yield_c + internal_c[i] * 0.12 - 0.003, 0.04, 3.2);
    internal_c[i] = internal_c[i] * 0.88 + growth * 0.25;
    biomass[i] = min(biomass[i] + 0.07 * step, 4.0);
    enzyme[i] = enzyme[i] + 0.018 * (1.0 - soluble_c[i] / (0.45 + soluble_c[i]));
    autocrine[i] = autocrine[i] + 0.035;
    atomicAdd(&tel[3], u32(growth * 1000.0));

    if biomass[i] > u.anastomosis_th && t.age > 18.0 && autocrine[i] > 0.08 {
        internal_c[i] = internal_c[i] + t.reserve;
        t.state = 0.0;
        t.reserve = 0.0;
        atomicAdd(&tel[1], 1u);
        tips[id] = t;
        return;
    }

    if t.age > u.min_branch_age && t.reserve > u.branch_cost && biomass[i] < 1.8 {
        let child = id + (u.tip_count / 2u);
        if child < u.tip_count && tips[child].state <= 0.0 {
            let roll = pcg(id * 1664525u + u.seed + bitcast<u32>(u.time));
            if (roll % 90u) == 0u {
                var kid = t;
                let axis = perp(dir, roll);
                let ca = cos(u.branch_angle);
                let sa = sin(u.branch_angle);
                kid.dir = safe_norm(dir * ca + axis * sa);
                kid.age = 0.0;
                kid.reserve = t.reserve * 0.42;
                kid.state = 1.0;
                kid.parent = id;
                kid.flags = 1u;
                tips[child] = kid;
                t.reserve = t.reserve * 0.55;
                atomicAdd(&tel[2], 1u);
            }
        }
    }
    tips[id] = t;
}

@compute @workgroup_size(8, 8, 4)
fn hud_reduce(@builtin(global_invocation_id) gid: vec3<u32>) {
    if gid.x >= u.width || gid.y >= u.height || gid.z >= u.depth {
        return;
    }
    // Strided census — every 4th voxel, cheap enough for a HUD.
    if ((gid.x | gid.y | gid.z) & 3u) != 0u {
        return;
    }
    let i = idx3(i32(gid.x), i32(gid.y), i32(gid.z));
    atomicAdd(&tel[4], u32(biomass[i] * 200.0));
    atomicAdd(&tel[5], u32(internal_c[i] * 200.0));
    atomicAdd(&tel[6], u32(soluble_c[i] * 200.0));
    atomicAdd(&tel[7], u32(soluble_n[i] * 200.0));
    atomicAdd(&tel[8], u32(enzyme[i] * 400.0));
    atomicAdd(&tel[9], u32(organic[i] * 120.0));
}

@compute @workgroup_size(64)
fn pick(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if id >= u.tip_count {
        return;
    }
    let t = tips[id];
    if t.state <= 0.0 {
        return;
    }
    if u.pick_active < 0.5 {
        return;
    }
    let orig = vec3<f32>(u.pick_ox, u.pick_oy, u.pick_oz);
    let dir = vec3<f32>(u.pick_dx, u.pick_dy, u.pick_dz);
    let along = max(dot(t.pos - orig, dir), 0.0);
    let d = distance(t.pos, orig + dir * along);
    let key = u32(d * 100.0);
    atomicMin(&tel[10], key);
    if atomicLoad(&tel[10]) == key {
        atomicStore(&tel[11], id);
    }
}

// Brush stamps. Mode 1 = inoculate, 2 = erase tips, 3 = paint add, 4 = paint subtract.
// Paint dispatches a small AABB: gid (0,0,0) is (round(center) - r - 1).
@compute @workgroup_size(8, 8, 4)
fn paint(@builtin(global_invocation_id) gid: vec3<u32>) {
    let mode = u32(u.brush_mode + 0.5);
    if mode != 3u && mode != 4u {
        return;
    }
    let r = i32(ceil(u.brush_radius)) + 1;
    let cx = i32(round(u.brush_x));
    let cy = i32(round(u.brush_y));
    let cz = i32(round(u.brush_z));
    let x = cx - r + i32(gid.x);
    let y = cy - r + i32(gid.y);
    let z = cz - r + i32(gid.z);
    if x < 0 || y < 0 || z < 0 {
        return;
    }
    if x >= i32(u.width) || y >= i32(u.height) || z >= i32(u.depth) {
        return;
    }
    let p = vec3<f32>(f32(x) + 0.5, f32(y) + 0.5, f32(z) + 0.5);
    let center = vec3<f32>(u.brush_x, u.brush_y, u.brush_z);
    let rad = max(u.brush_radius, 0.75);
    let d = distance(p, center);
    if d > rad {
        return;
    }
    let fall = pow(1.0 - d / rad, 1.4);
    let str = clamp(u.brush_strength, 0.05, 1.5) * fall;
    let sub = mode == 4u;
    let ch = u32(u.brush_channel + 0.5) % 8u;
    let i = idx3(x, y, z);

    if ch == 0u {
        if sub { soluble_c[i] = max(soluble_c[i] - 0.16 * str, 0.0); }
        else { soluble_c[i] = soluble_c[i] + 0.14 * str; }
        return;
    }
    if ch == 1u {
        if sub { soluble_n[i] = max(soluble_n[i] - 0.12 * str, 0.0); }
        else { soluble_n[i] = soluble_n[i] + 0.10 * str; }
        return;
    }
    if ch == 2u {
        if sub { moisture[i] = max(moisture[i] - 0.22 * str, 0.02); }
        else { moisture[i] = min(moisture[i] + 0.20 * str, 0.98); }
        return;
    }
    if ch == 3u {
        if sub { organic[i] = max(organic[i] - 0.45 * str, 0.0); }
        else { organic[i] = organic[i] + 0.40 * str; }
        return;
    }
    if ch == 4u {
        if sub { enzyme[i] = max(enzyme[i] - 0.10 * str, 0.0); }
        else { enzyme[i] = enzyme[i] + 0.085 * str; }
        return;
    }
    if ch == 5u {
        if sub {
            organic[i] = max(organic[i] - 0.80 * str, 0.0);
            soluble_c[i] = max(soluble_c[i] - 0.08 * str, 0.0);
        } else {
            organic[i] = organic[i] + 0.85 * str;
            soluble_c[i] = soluble_c[i] + 0.07 * str;
        }
        return;
    }
    if ch == 6u {
        if sub {
            organic[i] = max(organic[i] - 0.40 * str, 0.0);
            soluble_n[i] = max(soluble_n[i] - 0.14 * str, 0.0);
            soluble_c[i] = max(soluble_c[i] - 0.04 * str, 0.0);
        } else {
            organic[i] = organic[i] + 0.38 * str;
            soluble_n[i] = soluble_n[i] + 0.14 * str;
            soluble_c[i] = soluble_c[i] + 0.03 * str;
        }
        return;
    }
    // VOID: always a cutout of existing fields.
    biomass[i] = max(biomass[i] * (1.0 - 0.88 * str), 0.0);
    organic[i] = max(organic[i] * (1.0 - 0.80 * str), 0.0);
    soluble_c[i] = max(soluble_c[i] * (1.0 - 0.80 * str), 0.0);
    soluble_n[i] = max(soluble_n[i] * (1.0 - 0.80 * str), 0.0);
    enzyme[i] = max(enzyme[i] * (1.0 - 0.88 * str), 0.0);
    internal_c[i] = max(internal_c[i] * (1.0 - 0.75 * str), 0.0);
}

@compute @workgroup_size(64)
fn inoculate(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if id >= u.tip_count {
        return;
    }
    let mode = u32(u.brush_mode + 0.5);
    let center = vec3<f32>(u.brush_x, u.brush_y, u.brush_z);
    let rad = max(u.brush_radius, 1.0);

    if mode == 2u {
        var t = tips[id];
        if t.state <= 0.0 {
            return;
        }
        if distance(t.pos, center) > rad {
            return;
        }
        let want = u32(u.brush_lineage + 0.5);
        if want != 0u && t.lineage != want {
            return;
        }
        t.state = 0.0;
        t.reserve = 0.0;
        tips[id] = t;
        return;
    }

    if mode != 1u {
        return;
    }
    // Parent half only — the upper half is the grow() branch pool.
    if id >= (u.tip_count / 2u) {
        return;
    }
    if tips[id].state > 0.0 {
        return;
    }
    let cap = max(1u, min(6u, u32(rad / 3.0)));
    let claimed = atomicAdd(&tel[12], 1u);
    if claimed >= cap {
        return;
    }
    let h = pcg(id * 747796405u + u.seed + claimed * 17u);
    let a = f32(h & 1023u) * 0.006135923;
    let b = f32((h >> 10u) & 1023u) * 0.006135923;
    let spread = rad * 0.55 * f32((h >> 20u) & 255u) / 255.0;
    let off = vec3<f32>(sin(a) * cos(b), cos(a) * cos(b), sin(b) * 0.65) * spread;
    var t = tips[id];
    var pos = center + off;
    pos.x = pos.x - f32(u.width) * floor(pos.x / f32(u.width));
    pos.y = pos.y - f32(u.height) * floor(pos.y / f32(u.height));
    if pos.x < 0.0 { pos.x = pos.x + f32(u.width); }
    if pos.y < 0.0 { pos.y = pos.y + f32(u.height); }
    pos.z = clamp(pos.z, 1.0, f32(u.depth) - 2.0);
    t.pos = pos;
    t.dir = safe_norm(off + vec3<f32>(0.12, 0.04, 0.06));
    if length(t.dir) < 0.1 {
        t.dir = vec3<f32>(1.0, 0.0, 0.0);
    }
    t.age = 0.0;
    t.reserve = 1.1;
    t.state = 1.0;
    t.lineage = max(1u, u32(u.brush_lineage + 0.5));
    t.parent = 0u;
    t.flags = 2u;
    tips[id] = t;
}

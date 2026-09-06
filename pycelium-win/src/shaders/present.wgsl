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

struct PresentUniforms {
    width: u32,
    height: u32,
    depth: u32,
    out_w: u32,
    out_h: u32,
    pad0: u32,
    slice_z: f32,
    fps: f32,
    eye: vec4<f32>,
    aim: vec4<f32>,
    live_tips: f32,
    fusions: f32,
    branches: f32,
    cn_ratio: f32,
    biomass: f32,
    internal_c: f32,
    enzyme: f32,
    organic: f32,
    selected_id: f32,
    selected_lineage: f32,
    selected_age: f32,
    selected_reserve: f32,
    param_slot: f32,
    param_value: f32,
    soluble_c: f32,
    soluble_n: f32,
    slice_thickness: f32,
    slice_zoom: f32,
    slice_ox: f32,
    slice_oy: f32,
}

@group(0) @binding(0) var<uniform> u: PresentUniforms;
@group(0) @binding(1) var<storage, read> biomass: array<f32>;
@group(0) @binding(2) var<storage, read> internal_c: array<f32>;
@group(0) @binding(3) var<storage, read> soluble_c: array<f32>;
@group(0) @binding(4) var<storage, read> enzyme: array<f32>;
@group(0) @binding(5) var<storage, read> organic: array<f32>;
@group(0) @binding(6) var<storage, read> hud: array<u32>;
@group(0) @binding(7) var<storage, read> tips: array<Tip>;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
    var out: VsOut;
    let verts = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    out.clip = vec4<f32>(verts[i], 0.0, 1.0);
    return out;
}

fn wrap_i(v: i32, m: i32) -> i32 { return ((v % m) + m) % m; }

fn idx3(p: vec3<f32>) -> u32 {
    let w = i32(u.width);
    let h = i32(u.height);
    let d = i32(u.depth);
    let x = wrap_i(i32(floor(p.x)), w);
    let y = wrap_i(i32(floor(p.y)), h);
    let z = clamp(i32(floor(p.z)), 0, d - 1);
    return u32((z * h + y) * w + x);
}

fn sample3(p: vec3<f32>) -> vec4<f32> {
    let vol = vec3<f32>(f32(u.width), f32(u.height), f32(u.depth));
    let q = p * vol;
    let i = idx3(q);
    return vec4<f32>(biomass[i], internal_c[i], soluble_c[i], enzyme[i]);
}

fn hit_box(ro: vec3<f32>, rd: vec3<f32>) -> vec2<f32> {
    let inv = 1.0 / rd;
    let t0 = (vec3<f32>(0.0) - ro) * inv;
    let t1 = (vec3<f32>(1.0) - ro) * inv;
    let tmin = max(max(min(t0.x, t1.x), min(t0.y, t1.y)), min(t0.z, t1.z));
    let tmax = min(min(max(t0.x, t1.x), max(t0.y, t1.y)), max(t0.z, t1.z));
    return vec2<f32>(tmin, tmax);
}

fn digit(cell: vec2<f32>, n: i32) -> f32 {
    let p = vec2<i32>(i32(floor(cell.x * 3.0)), i32(floor((1.0 - cell.y) * 5.0)));
    if p.x < 0 || p.x > 2 || p.y < 0 || p.y > 4 { return 0.0; }
    var bits = 0u;
    switch n {
        case 0: { bits = 0x1D27u; }
        case 1: { bits = 0x12C9u; }
        case 2: { bits = 0x1EE3u; }
        case 3: { bits = 0x1E97u; }
        case 4: { bits = 0x12F4u; }
        case 5: { bits = 0x1F17u; }
        case 6: { bits = 0x1F27u; }
        case 7: { bits = 0x124Fu; }
        case 8: { bits = 0x1F2Fu; }
        case 9: { bits = 0x1F17u; }
        default: { bits = 0u; }
    }
    let bit = u32(p.y * 3 + p.x);
    return f32((bits >> bit) & 1u);
}

fn draw_number(uv: vec2<f32>, origin: vec2<f32>, value: f32, digits: i32) -> f32 {
    let local = (uv - origin) / vec2<f32>(0.014 * f32(digits), 0.028);
    if local.x < 0.0 || local.x > 1.0 || local.y < 0.0 || local.y > 1.0 {
        return 0.0;
    }
    let n = max(i32(value), 0);
    let slot = i32(floor(local.x * f32(digits)));
    let cell = vec2<f32>(fract(local.x * f32(digits)), local.y);
    var div = 1;
    for (var i = 0; i < digits - slot - 1; i++) { div = div * 10; }
    let d = (n / div) % 10;
    return digit(cell, d);
}

fn bar(uv: vec2<f32>, origin: vec2<f32>, fill: f32, rgb: vec3<f32>) -> vec3<f32> {
    let local = (uv - origin) / vec2<f32>(0.16, 0.016);
    if local.x < 0.0 || local.x > 1.0 || local.y < 0.0 || local.y > 1.0 {
        return vec3<f32>(0.0);
    }
    let edge = step(0.0, local.x) * step(local.x, 1.0) * step(0.0, local.y) * step(local.y, 1.0);
    let body = select(vec3<f32>(0.08, 0.09, 0.10), rgb, local.x < clamp(fill, 0.0, 1.0));
    return body * edge;
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let res = vec2<f32>(f32(u.out_w), f32(u.out_h));
    let uv = pos.xy / res;
    let aspect = res.x / res.y;

    let eye = u.eye.xyz;
    let tgt = u.aim.xyz;
    let fwd = normalize(tgt - eye);
    let right = normalize(cross(fwd, vec3<f32>(0.0, 0.0, 1.0)));
    let up = normalize(cross(right, fwd));
    let ndc = vec2<f32>((uv.x * 2.0 - 1.0) * aspect, -(uv.y * 2.0 - 1.0));
    let rd = normalize(fwd + right * ndc.x * 0.7 + up * ndc.y * 0.7);
    let hit = hit_box(eye, rd);

    var rgb = vec3<f32>(0.015, 0.018, 0.022);
    if hit.y > hit.x && hit.y > 0.0 {
        var t = max(hit.x, 0.0);
        let tmax = min(hit.y, t + 1.8);
        var acc = vec3<f32>(0.0);
        var alpha = 0.0;
        for (var s = 0; s < 110 && t < tmax && alpha < 0.97; s++) {
            let p = eye + rd * t;
            let f = sample3(p);
            let org = organic[idx3(p * vec3<f32>(f32(u.width), f32(u.height), f32(u.depth)))];
            let hypha = 1.0 - exp(-f.x * 3.4);
            let cord = 1.0 - exp(-f.y * 4.0);
            let food = 1.0 - exp(-f.z * 2.6);
            let enz = 1.0 - exp(-f.w * 5.0);
            var col = vec3<f32>(0.55, 0.92, 0.82) * hypha;
            col = col + vec3<f32>(0.95, 0.72, 0.28) * cord * 0.65;
            col = col + vec3<f32>(0.72, 0.38, 0.08) * food * 0.45;
            col = col + vec3<f32>(0.28, 0.62, 0.30) * enz * 0.25;
            col = col + vec3<f32>(0.12, 0.08, 0.05) * (1.0 - exp(-org * 1.4)) * 0.2;
            let dens = hypha * 0.55 + cord * 0.2 + food * 0.08 + 0.015;
            let z_vox = p.z * f32(u.depth);
            let slab = abs(z_vox - u.slice_z) <= max(u.slice_thickness * 0.5, 0.5);
            if slab {
                col = col + vec3<f32>(0.35, 0.28, 0.08);
            }
            acc = acc + (1.0 - alpha) * col * dens;
            alpha = alpha + (1.0 - alpha) * dens;
            t = t + 0.012;
        }
        rgb = mix(rgb, acc, clamp(alpha, 0.0, 1.0));
    }

    // Orthogonal slab inset — depth, thickness, and XY field.
    let inset = vec4<f32>(0.72, 0.06, 0.26, 0.28);
    let iu = (uv.x - inset.x) / inset.z;
    let iv = (uv.y - inset.y) / inset.w;
    if iu >= 0.0 && iu <= 1.0 && iv >= 0.0 && iv <= 1.0 {
        let zoom = max(u.slice_zoom, 0.12);
        let xw = (u.slice_ox + iu * zoom) * f32(u.width);
        let yw = (u.slice_oy + iv * zoom) * f32(u.height);
        let half = max(u.slice_thickness * 0.5, 0.5);
        var hypha = 0.0;
        var food = 0.0;
        let z0 = i32(floor(u.slice_z - half));
        let z1 = i32(ceil(u.slice_z + half));
        let span = max(z1 - z0, 1);
        let step_z = max(span / 48, 1);
        for (var z = z0; z <= z1; z = z + step_z) {
            let i = idx3(vec3<f32>(xw, yw, f32(z)));
            hypha = max(hypha, biomass[i]);
            food = max(food, soluble_c[i]);
        }
        hypha = 1.0 - exp(-hypha * 3.0);
        food = 1.0 - exp(-food * 3.2);
        var srgb = mix(vec3<f32>(0.04, 0.045, 0.05), vec3<f32>(0.62, 0.95, 0.86), hypha);
        srgb = srgb + vec3<f32>(0.7, 0.36, 0.08) * food;
        let frame = step(min(min(iu, iv), min(1.0 - iu, 1.0 - iv)), 0.02);
        rgb = mix(srgb, vec3<f32>(0.8, 0.8, 0.75), frame);
    }

    // Telemetry panel.
    if uv.x < 0.24 {
        rgb = mix(rgb, vec3<f32>(0.03, 0.035, 0.04), 0.78);
        var ink = 0.0;
        let live = f32(hud[0]);
        let fusions = f32(hud[1]);
        let branches = f32(hud[2]);
        let sol_c = f32(hud[6]);
        let sol_n = f32(hud[7]);
        let cn = select(sol_c / max(sol_n, 1.0), 0.0, sol_n < 1.0);
        let sid = hud[11];
        var lineage = 0.0;
        var age = 0.0;
        var reserve = 0.0;
        if sid < 20000000u {
            let tip = tips[sid];
            lineage = f32(tip.lineage);
            age = tip.age;
            reserve = tip.reserve;
        }
        ink = ink + draw_number(uv, vec2<f32>(0.018, 0.06), u.fps, 3);
        ink = ink + draw_number(uv, vec2<f32>(0.018, 0.12), live, 6);
        ink = ink + draw_number(uv, vec2<f32>(0.018, 0.16), fusions, 5);
        ink = ink + draw_number(uv, vec2<f32>(0.018, 0.20), branches, 5);
        ink = ink + draw_number(uv, vec2<f32>(0.018, 0.24), cn, 4);
        ink = ink + draw_number(uv, vec2<f32>(0.018, 0.70), f32(sid), 6);
        ink = ink + draw_number(uv, vec2<f32>(0.018, 0.74), lineage, 3);
        ink = ink + draw_number(uv, vec2<f32>(0.018, 0.78), age, 5);
        ink = ink + draw_number(uv, vec2<f32>(0.018, 0.82), reserve * 100.0, 4);
        ink = ink + draw_number(uv, vec2<f32>(0.018, 0.56), u.slice_z, 4);
        ink = ink + draw_number(uv, vec2<f32>(0.018, 0.60), u.slice_thickness, 3);
        ink = ink + draw_number(uv, vec2<f32>(0.018, 0.64), u.slice_zoom * 100.0, 3);
        ink = ink + draw_number(uv, vec2<f32>(0.018, 0.90), u.param_slot, 1);
        ink = ink + draw_number(uv, vec2<f32>(0.050, 0.90), u.param_value * 100.0, 4);
        rgb = rgb + vec3<f32>(0.82, 0.88, 0.80) * ink;
        rgb = rgb + bar(uv, vec2<f32>(0.03, 0.32), f32(hud[4]) / 80000.0, vec3<f32>(0.55, 0.92, 0.82));
        rgb = rgb + bar(uv, vec2<f32>(0.03, 0.36), f32(hud[5]) / 40000.0, vec3<f32>(0.95, 0.72, 0.28));
        rgb = rgb + bar(uv, vec2<f32>(0.03, 0.40), f32(hud[6]) / 40000.0, vec3<f32>(0.78, 0.42, 0.10));
        rgb = rgb + bar(uv, vec2<f32>(0.03, 0.44), f32(hud[7]) / 25000.0, vec3<f32>(0.55, 0.40, 0.85));
        rgb = rgb + bar(uv, vec2<f32>(0.03, 0.48), f32(hud[8]) / 20000.0, vec3<f32>(0.32, 0.70, 0.34));
        rgb = rgb + bar(uv, vec2<f32>(0.03, 0.52), f32(hud[9]) / 50000.0, vec3<f32>(0.45, 0.32, 0.18));
    }

    return vec4<f32>(rgb, 1.0);
}

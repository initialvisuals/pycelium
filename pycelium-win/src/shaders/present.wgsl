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
    hud_ui: vec4<f32>,
    cutter: vec4<f32>,
}

@group(0) @binding(0) var<uniform> u: PresentUniforms;
@group(0) @binding(1) var<storage, read> biomass: array<f32>;
@group(0) @binding(2) var<storage, read> internal_c: array<f32>;
@group(0) @binding(3) var<storage, read> soluble_c: array<f32>;
@group(0) @binding(4) var<storage, read> enzyme: array<f32>;
@group(0) @binding(5) var<storage, read> organic: array<f32>;
@group(0) @binding(6) var<storage, read> hud: array<u32>;
@group(0) @binding(7) var<storage, read> tips: array<Tip>;
@group(0) @binding(8) var font_tex: texture_2d<f32>;
@group(0) @binding(9) var font_samp: sampler;

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
    let local = (uv - origin) / vec2<f32>(0.0082 * f32(digits), 0.016);
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
    let local = (uv - origin) / vec2<f32>(0.11, 0.016);
    if local.x < 0.0 || local.x > 1.0 || local.y < 0.0 || local.y > 1.0 {
        return vec3<f32>(0.0);
    }
    let edge = step(0.0, local.x) * step(local.x, 1.0) * step(0.0, local.y) * step(local.y, 1.0);
    let body = select(vec3<f32>(0.08, 0.09, 0.10), rgb, local.x < clamp(fill, 0.0, 1.0));
    return body * edge;
}

fn px() -> vec2<f32> {
    return vec2<f32>(1.0 / max(f32(u.out_w), 1.0), 1.0 / max(f32(u.out_h), 1.0));
}

// Geometric sans atlas: 16×3 cells of 32px. cell.y = 0 is the TOP of the glyph.
// Built in hud_font.rs (upright). The old 5×5 pack was sampled with 1-y and looked inverted.
fn letter(cell: vec2<f32>, ch: i32) -> f32 {
    if ch <= 0 { return 0.0; }
    if cell.x < 0.0 || cell.x > 1.0 || cell.y < 0.0 || cell.y > 1.0 {
        return 0.0;
    }
    let col = ch % 16;
    let row = ch / 16;
    let uv = (vec2<f32>(f32(col), f32(row)) + cell) / vec2<f32>(16.0, 3.0);
    return textureSampleLevel(font_tex, font_samp, uv, 0.0).r;
}

// Short English names for the left HUD. -1 terminates a string of at most 8 glyphs.
fn label_char(id: i32, slot: i32) -> i32 {
    switch id {
        case 0: { // FPS
            switch slot { case 0: { return 6; } case 1: { return 16; } case 2: { return 19; } default: { return -1; } }
        }
        case 1: { // TIPS
            switch slot { case 0: { return 20; } case 1: { return 9; } case 2: { return 16; } case 3: { return 19; } default: { return -1; } }
        }
        case 2: { // FUSIONS
            switch slot { case 0: { return 6; } case 1: { return 21; } case 2: { return 19; } case 3: { return 9; } case 4: { return 15; } case 5: { return 14; } case 6: { return 19; } default: { return -1; } }
        }
        case 3: { // BRANCHES
            switch slot { case 0: { return 2; } case 1: { return 18; } case 2: { return 1; } case 3: { return 14; } case 4: { return 3; } case 5: { return 8; } case 6: { return 5; } case 7: { return 19; } default: { return -1; } }
        }
        case 4: { // C:N
            switch slot { case 0: { return 3; } case 1: { return 27; } case 2: { return 14; } default: { return -1; } }
        }
        case 5: { // HYPHA
            switch slot { case 0: { return 8; } case 1: { return 25; } case 2: { return 16; } case 3: { return 8; } case 4: { return 1; } default: { return -1; } }
        }
        case 6: { // CORD
            switch slot { case 0: { return 3; } case 1: { return 15; } case 2: { return 18; } case 3: { return 4; } default: { return -1; } }
        }
        case 7: { // SOL C
            switch slot { case 0: { return 19; } case 1: { return 15; } case 2: { return 12; } case 3: { return 0; } case 4: { return 3; } default: { return -1; } }
        }
        case 8: { // SOL N
            switch slot { case 0: { return 19; } case 1: { return 15; } case 2: { return 12; } case 3: { return 0; } case 4: { return 14; } default: { return -1; } }
        }
        case 9: { // ENZYME
            switch slot { case 0: { return 5; } case 1: { return 14; } case 2: { return 26; } case 3: { return 25; } case 4: { return 13; } case 5: { return 5; } default: { return -1; } }
        }
        case 10: { // ORGANIC
            switch slot { case 0: { return 15; } case 1: { return 18; } case 2: { return 7; } case 3: { return 1; } case 4: { return 14; } case 5: { return 9; } case 6: { return 3; } default: { return -1; } }
        }
        case 11: { // SLICE Z
            switch slot { case 0: { return 19; } case 1: { return 12; } case 2: { return 9; } case 3: { return 3; } case 4: { return 5; } case 5: { return 0; } case 6: { return 26; } default: { return -1; } }
        }
        case 12: { // THICK
            switch slot { case 0: { return 20; } case 1: { return 8; } case 2: { return 9; } case 3: { return 3; } case 4: { return 11; } default: { return -1; } }
        }
        case 13: { // ZOOM
            switch slot { case 0: { return 26; } case 1: { return 15; } case 2: { return 15; } case 3: { return 13; } default: { return -1; } }
        }
        case 14: { // TIP
            switch slot { case 0: { return 20; } case 1: { return 9; } case 2: { return 16; } default: { return -1; } }
        }
        case 15: { // LINEAGE
            switch slot { case 0: { return 12; } case 1: { return 9; } case 2: { return 14; } case 3: { return 5; } case 4: { return 1; } case 5: { return 7; } case 6: { return 5; } default: { return -1; } }
        }
        case 16: { // AGE
            switch slot { case 0: { return 1; } case 1: { return 7; } case 2: { return 5; } default: { return -1; } }
        }
        case 17: { // RESERVE
            switch slot { case 0: { return 18; } case 1: { return 5; } case 2: { return 19; } case 3: { return 5; } case 4: { return 18; } case 5: { return 22; } case 6: { return 5; } default: { return -1; } }
        }
        case 18: { // PARAM
            switch slot { case 0: { return 16; } case 1: { return 1; } case 2: { return 18; } case 3: { return 1; } case 4: { return 13; } default: { return -1; } }
        }
        default: { return -1; }
    }
}

fn draw_label(uv: vec2<f32>, origin: vec2<f32>, id: i32) -> f32 {
    let n = 8.0;
    let local = (uv - origin) / vec2<f32>(0.0112 * n, 0.022);
    if local.x < 0.0 || local.x > 1.0 || local.y < 0.0 || local.y > 1.0 {
        return 0.0;
    }
    let slot = i32(floor(local.x * n));
    let cell = vec2<f32>(fract(local.x * n), local.y);
    let ch = label_char(id, slot);
    if ch < 0 { return 0.0; }
    return letter(cell, ch);
}

fn thin_frame(uv: vec2<f32>, r0: vec2<f32>, r1: vec2<f32>) -> f32 {
    let p = px();
    let inside = uv.x >= r0.x && uv.x <= r1.x && uv.y >= r0.y && uv.y <= r1.y;
    let inner = uv.x >= r0.x + p.x && uv.x <= r1.x - p.x && uv.y >= r0.y + p.y && uv.y <= r1.y - p.y;
    return select(0.0, 1.0, inside && !inner);
}

fn row_alpha(cursor: vec2<f32>, y0: f32, y1: f32, density: f32, fade: f32, picked: bool) -> f32 {
    if density < 0.5 || fade <= 0.0 {
        return 0.0;
    }
    let in_panel = cursor.x < 0.26;
    let mid = 0.5 * (y0 + y1);
    let half = max(0.5 * (y1 - y0), 0.012);
    let dy = abs(cursor.y - mid);
    var hover = 0.0;
    if in_panel {
        hover = 1.0 - smoothstep(half, half + 0.028, dy);
    }
    let base = select(0.0, 0.96, density > 1.5);
    let focus = select(base, max(base, 0.96), picked);
    return clamp(max(focus, hover), 0.0, 1.0) * fade;
}

fn paint_label(uv: vec2<f32>, origin: vec2<f32>, id: i32, alpha: f32, rgb: vec3<f32>) -> vec3<f32> {
    if alpha <= 0.004 {
        return rgb;
    }
    let shadow = draw_label(uv, origin + px() * 1.5, id);
    let ink = draw_label(uv, origin, id);
    var out = rgb;
    out = mix(out, vec3<f32>(0.0, 0.0, 0.0), shadow * alpha * 0.45);
    out = mix(out, vec3<f32>(1.0, 1.0, 1.0), ink * alpha);
    return out;
}

fn swatch(uv: vec2<f32>, origin: vec2<f32>, fill: f32, tint: vec3<f32>) -> vec3<f32> {
    let local = (uv - origin) / vec2<f32>(0.015, 0.018);
    if local.x < 0.0 || local.x > 1.0 || local.y < 0.0 || local.y > 1.0 {
        return vec3<f32>(0.0);
    }
    let p = px();
    let edge = local.x < p.x / 0.015 || local.x > 1.0 - p.x / 0.015 || local.y < p.y / 0.018 || local.y > 1.0 - p.y / 0.018;
    let body = mix(tint * 0.18, tint, clamp(fill, 0.22, 1.0));
    return select(body, vec3<f32>(0.92, 0.93, 0.90), edge);
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
            var dens = hypha * 0.55 + cord * 0.2 + food * 0.08 + 0.015;
            let z_vox = p.z * f32(u.depth);
            let slab = abs(z_vox - u.slice_z) <= max(u.slice_thickness * 0.5, 0.5);
            if slab {
                col = col + vec3<f32>(0.35, 0.28, 0.08);
            }
            let capture = u.cutter.w;
            if capture >= 0.5 {
                let vol = vec3<f32>(f32(u.width), f32(u.height), f32(u.depth));
                let pv = p * vol;
                let ax = i32(u.cutter.x + 0.5);
                var coord = pv.z;
                var asz = vol.z;
                if ax == 0 {
                    coord = pv.x;
                    asz = vol.x;
                } else if ax == 1 {
                    coord = pv.y;
                    asz = vol.y;
                }
                let center = u.cutter.y * asz;
                let half = max(u.cutter.z, 0.5);
                let dist = abs(coord - center);
                if dist <= half {
                    let face = abs(dist - half) < 0.65;
                    let glow = select(0.28, 0.95, face);
                    let snap = select(0.0, 0.18, fract(capture) > 0.25);
                    col = col + vec3<f32>(1.0, 0.48, 0.08) * (glow + snap);
                    dens = dens + select(0.03, 0.07, face);
                }
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

    // Capture cutter: snapped cube-face wash + two-ended thickness slider.
    if u.cutter.w >= 0.5 {
        if hit.y > hit.x && hit.y > 0.0 && fract(u.cutter.w) > 0.25 {
            let p0 = eye + rd * max(hit.x, 0.0);
            let ax = i32(u.cutter.x + 0.5);
            var face_d = min(p0.z, 1.0 - p0.z);
            if ax == 0 { face_d = min(p0.x, 1.0 - p0.x); }
            else if ax == 1 { face_d = min(p0.y, 1.0 - p0.y); }
            if face_d < 0.014 {
                rgb = mix(rgb, vec3<f32>(1.0, 0.50, 0.10), 0.22);
            }
        }
        let sx0 = 0.30;
        let sx1 = 0.70;
        let sy0 = 0.900;
        let sy1 = 0.958;
        if uv.x >= sx0 - 0.012 && uv.x <= sx1 + 0.012 && uv.y >= sy0 && uv.y <= sy1 {
            let t = clamp((uv.x - sx0) / (sx1 - sx0), 0.0, 1.0);
            let pos = u.cutter.y;
            var asz = f32(u.depth);
            let ax = i32(u.cutter.x + 0.5);
            if ax == 0 { asz = f32(u.width); }
            else if ax == 1 { asz = f32(u.height); }
            let hn = clamp(u.cutter.z / max(asz, 1.0), 0.0, 0.5);
            let lo = pos - hn;
            let hi = pos + hn;
            var srgb = vec3<f32>(0.07, 0.07, 0.08);
            if t >= lo && t <= hi {
                srgb = vec3<f32>(0.85, 0.42, 0.08);
            }
            let track = abs(uv.y - 0.5 * (sy0 + sy1)) < 0.003;
            if track {
                srgb = vec3<f32>(0.35, 0.32, 0.28);
            }
            let handle = (abs(t - lo) < 0.012 || abs(t - hi) < 0.012) && abs(uv.y - 0.5 * (sy0 + sy1)) < 0.018;
            if handle {
                srgb = vec3<f32>(1.0, 0.62, 0.18);
            }
            let mid = abs(t - pos) < 0.004;
            if mid {
                srgb = vec3<f32>(0.95, 0.90, 0.70);
            }
            let frame = step(min(min((uv.x - (sx0 - 0.008)), (sx1 + 0.008) - uv.x), min(uv.y - sy0, sy1 - uv.y)), 0.002);
            rgb = mix(srgb, vec3<f32>(0.80, 0.78, 0.70), frame);
        }
    }

    // Telemetry panel. Readable white captions + color readout boxes.
    // Optional small 3×5 digits sit to the right. Row map: docs/HUD_LEGEND.md
    if uv.x < 0.26 {
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
        let hypha_f = clamp(f32(hud[4]) / 80000.0, 0.0, 1.0);
        let cord_f = clamp(f32(hud[5]) / 40000.0, 0.0, 1.0);
        let solc_f = clamp(f32(hud[6]) / 40000.0, 0.0, 1.0);
        let soln_f = clamp(f32(hud[7]) / 25000.0, 0.0, 1.0);
        let enz_f = clamp(f32(hud[8]) / 20000.0, 0.0, 1.0);
        let org_f = clamp(f32(hud[9]) / 50000.0, 0.0, 1.0);
        let teal = vec3<f32>(0.55, 0.92, 0.82);
        let amber = vec3<f32>(0.95, 0.72, 0.28);
        let rust = vec3<f32>(0.78, 0.42, 0.10);
        let violet = vec3<f32>(0.55, 0.40, 0.85);
        let green = vec3<f32>(0.32, 0.70, 0.34);
        let brown = vec3<f32>(0.45, 0.32, 0.18);
        let ice = vec3<f32>(0.78, 0.84, 0.88);
        let gold = vec3<f32>(0.90, 0.78, 0.40);
        let warm = vec3<f32>(0.92, 0.62, 0.28);

        ink = ink + draw_number(uv, vec2<f32>(0.168, 0.058), u.fps, 3);
        ink = ink + draw_number(uv, vec2<f32>(0.168, 0.100), live, 6);
        ink = ink + draw_number(uv, vec2<f32>(0.168, 0.142), fusions, 5);
        ink = ink + draw_number(uv, vec2<f32>(0.168, 0.184), branches, 5);
        ink = ink + draw_number(uv, vec2<f32>(0.168, 0.226), cn, 4);
        ink = ink + draw_number(uv, vec2<f32>(0.168, 0.558), u.slice_z, 4);
        ink = ink + draw_number(uv, vec2<f32>(0.168, 0.600), u.slice_thickness, 3);
        ink = ink + draw_number(uv, vec2<f32>(0.168, 0.642), u.slice_zoom * 100.0, 3);
        ink = ink + draw_number(uv, vec2<f32>(0.168, 0.700), f32(sid), 6);
        ink = ink + draw_number(uv, vec2<f32>(0.168, 0.742), lineage, 3);
        ink = ink + draw_number(uv, vec2<f32>(0.168, 0.784), age, 5);
        ink = ink + draw_number(uv, vec2<f32>(0.168, 0.826), reserve * 100.0, 4);
        ink = ink + draw_number(uv, vec2<f32>(0.168, 0.896), u.param_slot, 1);
        ink = ink + draw_number(uv, vec2<f32>(0.188, 0.896), u.param_value * 100.0, 4);
        rgb = rgb + vec3<f32>(0.70, 0.76, 0.72) * ink * 0.85;
        rgb = rgb + bar(uv, vec2<f32>(0.128, 0.312), hypha_f, teal);
        rgb = rgb + bar(uv, vec2<f32>(0.128, 0.354), cord_f, amber);
        rgb = rgb + bar(uv, vec2<f32>(0.128, 0.396), solc_f, rust);
        rgb = rgb + bar(uv, vec2<f32>(0.128, 0.438), soln_f, violet);
        rgb = rgb + bar(uv, vec2<f32>(0.128, 0.480), enz_f, green);
        rgb = rgb + bar(uv, vec2<f32>(0.128, 0.522), org_f, brown);

        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.058), 0.85, ice);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.100), clamp(live / 400000.0, 0.2, 1.0), teal);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.142), clamp(fusions / 8000.0, 0.2, 1.0), amber);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.184), clamp(branches / 8000.0, 0.2, 1.0), green);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.226), clamp(cn / 12.0, 0.2, 1.0), rust);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.318), hypha_f, teal);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.360), cord_f, amber);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.402), solc_f, rust);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.444), soln_f, violet);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.486), enz_f, green);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.528), org_f, brown);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.558), 0.7, warm);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.600), 0.55, warm);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.642), 0.4, warm);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.700), select(0.25, 0.9, sid < 20000000u), teal);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.742), select(0.25, 0.75, sid < 20000000u), gold);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.784), select(0.25, 0.6, sid < 20000000u), ice);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.826), select(0.25, clamp(reserve, 0.25, 1.0), sid < 20000000u), amber);
        rgb = rgb + swatch(uv, vec2<f32>(0.012, 0.896), 0.7, ice);

        let cursor = u.hud_ui.xy;
        let density = u.hud_ui.z;
        let fade = clamp(u.hud_ui.w, 0.0, 1.0);
        let picked = sid < 20000000u && u.selected_id > 0.0;
        let chrome = row_alpha(cursor, 0.04, 0.96, density, fade, false) * 0.45;
        rgb = mix(rgb, vec3<f32>(0.72, 0.74, 0.70), thin_frame(uv, vec2<f32>(0.008, 0.048), vec2<f32>(0.250, 0.258)) * chrome);
        rgb = mix(rgb, vec3<f32>(0.72, 0.74, 0.70), thin_frame(uv, vec2<f32>(0.008, 0.300), vec2<f32>(0.250, 0.548)) * chrome);
        rgb = mix(rgb, vec3<f32>(0.72, 0.74, 0.70), thin_frame(uv, vec2<f32>(0.008, 0.548), vec2<f32>(0.250, 0.682)) * chrome);
        rgb = mix(rgb, vec3<f32>(0.72, 0.74, 0.70), thin_frame(uv, vec2<f32>(0.008, 0.688), vec2<f32>(0.250, 0.862)) * chrome);
        rgb = mix(rgb, vec3<f32>(0.72, 0.74, 0.70), thin_frame(uv, vec2<f32>(0.008, 0.878), vec2<f32>(0.250, 0.938)) * chrome);

        let a0 = row_alpha(cursor, 0.050, 0.095, density, fade, false);
        let a1 = row_alpha(cursor, 0.095, 0.137, density, fade, false);
        let a2 = row_alpha(cursor, 0.137, 0.179, density, fade, false);
        let a3 = row_alpha(cursor, 0.179, 0.221, density, fade, false);
        let a4 = row_alpha(cursor, 0.221, 0.268, density, fade, false);
        let a5 = row_alpha(cursor, 0.305, 0.350, density, fade, false);
        let a6 = row_alpha(cursor, 0.350, 0.392, density, fade, false);
        let a7 = row_alpha(cursor, 0.392, 0.434, density, fade, false);
        let a8 = row_alpha(cursor, 0.434, 0.476, density, fade, false);
        let a9 = row_alpha(cursor, 0.476, 0.518, density, fade, false);
        let a10 = row_alpha(cursor, 0.518, 0.555, density, fade, false);
        let a11 = row_alpha(cursor, 0.548, 0.592, density, fade, false);
        let a12 = row_alpha(cursor, 0.592, 0.634, density, fade, false);
        let a13 = row_alpha(cursor, 0.634, 0.682, density, fade, false);
        let a14 = row_alpha(cursor, 0.688, 0.734, density, fade, picked);
        let a15 = row_alpha(cursor, 0.734, 0.776, density, fade, picked);
        let a16 = row_alpha(cursor, 0.776, 0.818, density, fade, picked);
        let a17 = row_alpha(cursor, 0.818, 0.862, density, fade, picked);
        let a18 = row_alpha(cursor, 0.878, 0.940, density, fade, false);

        rgb = paint_label(uv, vec2<f32>(0.032, 0.054), 0, a0, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.096), 1, a1, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.138), 2, a2, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.180), 3, a3, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.222), 4, a4, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.308), 5, a5, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.350), 6, a6, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.392), 7, a7, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.434), 8, a8, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.476), 9, a9, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.518), 10, a10, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.554), 11, a11, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.596), 12, a12, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.638), 13, a13, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.696), 14, a14, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.738), 15, a15, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.780), 16, a16, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.822), 17, a17, rgb);
        rgb = paint_label(uv, vec2<f32>(0.032, 0.892), 18, a18, rgb);
    }

    return vec4<f32>(rgb, 1.0);
}

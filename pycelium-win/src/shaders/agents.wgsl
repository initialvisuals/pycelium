struct Agent {
    pos: vec2<f32>,
    angle: f32,
    energy: f32,
}

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
@group(0) @binding(1) var<storage, read_write> agents: array<Agent>;
@group(0) @binding(2) var<storage, read> trail: array<f32>;
@group(0) @binding(3) var<storage, read_write> deposit: array<atomic<u32>>;
@group(0) @binding(4) var<storage, read_write> nutrient: array<f32>;

fn wrap_i(v: i32, m: i32) -> i32 {
    return ((v % m) + m) % m;
}

fn cell_index(x: i32, y: i32) -> u32 {
    let w = i32(u.width);
    let h = i32(u.height);
    return u32(wrap_i(y, h) * w + wrap_i(x, w));
}

fn sample_field(p: vec2<f32>) -> f32 {
    let i = cell_index(i32(floor(p.x)), i32(floor(p.y)));
    return trail[i] + nutrient[i] * u.nutrient_weight;
}

fn pcg(n: u32) -> u32 {
    var h = n * 747796405u + 2891336453u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
    return (h >> 22u) ^ h;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if id >= u.agent_count {
        return;
    }

    var a = agents[id];
    if a.energy <= 0.0 {
        return;
    }

    let heading = a.angle;
    let pos = a.pos;
    let sa = u.sensor_angle;
    let sd = u.sensor_distance;

    let s_fwd = sample_field(pos + vec2<f32>(cos(heading), sin(heading)) * sd);
    let s_left = sample_field(pos + vec2<f32>(cos(heading + sa), sin(heading + sa)) * sd);
    let s_right = sample_field(pos + vec2<f32>(cos(heading - sa), sin(heading - sa)) * sd);

    if s_fwd >= s_left && s_fwd >= s_right {
        // keep heading — hyphae run toward the strongest cue
    } else if s_left > s_right {
        a.angle = heading + u.turn_angle;
    } else if s_right > s_left {
        a.angle = heading - u.turn_angle;
    } else {
        let twitch = pcg(id ^ u.seed ^ bitcast<u32>(u.time));
        if (twitch & 1u) == 0u {
            a.angle = heading + u.turn_angle;
        } else {
            a.angle = heading - u.turn_angle;
        }
    }

    let dir = vec2<f32>(cos(a.angle), sin(a.angle));
    var next = pos + dir * u.step_size * clamp(0.65 + a.energy * 0.25, 0.5, 1.8);
    let fw = f32(u.width);
    let fh = f32(u.height);
    next.x = next.x - fw * floor(next.x / fw);
    next.y = next.y - fh * floor(next.y / fh);
    if next.x < 0.0 {
        next.x = next.x + fw;
    }
    if next.y < 0.0 {
        next.y = next.y + fh;
    }
    a.pos = next;

    let di = cell_index(i32(a.pos.x), i32(a.pos.y));
    let strength = u.deposit * 1000.0 * clamp(a.energy, 0.2, 2.4);
    atomicAdd(&deposit[di], u32(max(strength, 1.0)));

    let eaten = min(nutrient[di], u.consume * (0.4 + a.energy));
    nutrient[di] = nutrient[di] - eaten;
    a.energy = clamp(a.energy + eaten * 10.0 - 0.0018, 0.08, 3.0);

    if a.energy > u.branch_threshold {
        let child = id + (u.agent_count / 2u);
        if child < u.agent_count && agents[child].energy <= 0.0 {
            let roll = pcg(id * 1664525u + u.seed + bitcast<u32>(u.time));
            if (roll % 72u) == 0u {
                let fork = 0.55 + f32(roll % 17u) * 0.04;
                var kid = a;
                kid.angle = a.angle + fork;
                kid.energy = 0.9;
                agents[child] = kid;
                a.energy = a.energy * 0.62;
            }
        }
    }

    agents[id] = a;
}

const PI: f32 = 3.14159265358979323846264338327950288;
const TAU: f32 = 6.28318530717958647692528676655900577;

struct Particle {
    @location(0) pos: vec2<f32>,
    @location(1) vel: vec2<f32>,
};

struct SimParams {
    deltaT: f32,
    rule1Distance: f32,
    rule2Distance: f32,
    rule3Distance: f32,
    rule1Scale: f32,
    rule2Scale: f32,
    rule3Scale: f32,
};

struct VertexOutput {
  @builtin(position) position: vec4f,
  @location(0) color: vec4f,
}

var<private> VERTEX_POSITIONS: array<vec2f, 3> = array(vec2f(-0.01, -0.02), vec2f(0.01, -0.02), vec2f(0.00, 0.02));

@vertex
fn main_vs(
    particle: Particle,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    let position = VERTEX_POSITIONS[vertex_index];
    let angle = -atan2(particle.vel.x, particle.vel.y);
    let pos = vec2<f32>(
        position.x * cos(angle) - position.y * sin(angle),
        position.x * sin(angle) + position.y * cos(angle)
    );

    var output: VertexOutput;
    output.position = vec4(pos + particle.pos, 0., 1.);
    output.color = vec4f(
        saturate(cos(angle)),
        saturate(cos(angle - (TAU / 3.))),
        saturate(cos(angle - (2. * TAU / 3.))),
        1.
    );
    return output;
}

@fragment
fn main_fs(@location(0) color: vec4f) -> @location(0) vec4f {
    return color;
}


@group(0) @binding(0) var<uniform> params : SimParams;
@group(0) @binding(1) var<storage, read> particlesSrc : array<Particle>;
@group(0) @binding(2) var<storage, read_write> particlesDst : array<Particle>;

// https://github.com/austinEng/Project6-Vulkan-Flocking/blob/master/data/shaders/computeparticles/particle.comp
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&particlesSrc);
    let index = global_invocation_id.x;
    if index >= total {
        return;
    }

    var vPos: vec2<f32> = particlesSrc[index].pos;
    var vVel: vec2<f32> = particlesSrc[index].vel;

    var cMass: vec2<f32> = vec2<f32>(0.0, 0.0);
    var cVel: vec2<f32> = vec2<f32>(0.0, 0.0);
    var colVel: vec2<f32> = vec2<f32>(0.0, 0.0);
    var cMassCount: i32 = 0;
    var cVelCount: i32 = 0;

    var i: u32 = 0u;
    loop {
        if i >= total {
            break;
        }
        if i == index {
            continue;
        }

        let pos = particlesSrc[i].pos;
        let vel = particlesSrc[i].vel;

        if distance(pos, vPos) < params.rule1Distance {
            cMass += pos;
            cMassCount += 1;
        }
        if distance(pos, vPos) < params.rule2Distance {
            colVel -= pos - vPos;
        }
        if distance(pos, vPos) < params.rule3Distance {
            cVel += vel;
            cVelCount += 1;
        }

        continuing {
            i = i + 1u;
        }
    }
    if cMassCount > 0 {
        cMass = cMass * (1.0 / f32(cMassCount)) - vPos;
    }
    if cVelCount > 0 {
        cVel *= 1.0 / f32(cVelCount);
    }

    vVel = vVel + (cMass * params.rule1Scale) + (colVel * params.rule2Scale) + (cVel * params.rule3Scale);

    // clamp velocity for a more pleasing simulation
    vVel = normalize(vVel) * clamp(length(vVel), 0.0, 0.1);

    // kinematic update
    vPos += vVel * params.deltaT;

    // Wrap around boundary
    if vPos.x < -1.0 {
        vPos.x = 1.0;
    }
    if vPos.x > 1.0 {
        vPos.x = -1.0;
    }
    if vPos.y < -1.0 {
        vPos.y = 1.0;
    }
    if vPos.y > 1.0 {
        vPos.y = -1.0;
    }

    // Write back
    particlesDst[index] = Particle(vPos, vVel);
}

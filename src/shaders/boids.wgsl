const PI: f32 = 3.14159265358979323846264338327950288;
const TAU: f32 = 6.28318530717958647692528676655900577;

struct Particle {
    @location(0) pos: vec2<f32>,
    @location(1) vel: vec2<f32>,
};

struct SimParams {
    delta_time: f32,
    separation_distance: f32,
    alignment_distance: f32,
    cohesion_distance: f32,
    separation_scale: f32,
    alignment_scale: f32,
    cohesion_scale: f32,
};

struct VertexOutput {
  @builtin(position) position: vec4f,
  @location(0) color: vec4f,
}

var<private> VERTEX_POSITIONS: array<vec2f, 3> = array(vec2f(-0.01, -0.02), vec2f(0.01, -0.02), vec2f(0.00, 0.02));

@vertex
fn boids_vs(
    particle: Particle,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    let position = 0.2 * VERTEX_POSITIONS[vertex_index];
    let angle = -atan2(particle.vel.x, particle.vel.y);
    let pos = vec2<f32>(
        position.x * cos(angle) - position.y * sin(angle),
        position.x * sin(angle) + position.y * cos(angle)
    );

    var output: VertexOutput;
    output.position = vec4(pos + particle.pos, 0., 1.);
    output.color = vec4f(
        saturate(2. * cos(angle)),
        saturate(2. * cos(angle - (TAU / 3.))),
        saturate(2. * cos(angle - (2. * TAU / 3.))),
        1.
    );
    return output;
}

@fragment
fn boids_fs(@location(0) color: vec4f) -> @location(0) vec4f {
    return color;
}

@group(0) @binding(0) var<uniform> params : SimParams;
@group(0) @binding(1) var<storage, read> particles_src : array<Particle>;
@group(0) @binding(2) var<storage, read_write> particles_dst : array<Particle>;

// https://github.com/austinEng/Project6-Vulkan-Flocking/blob/master/data/shaders/computeparticles/particle.comp
@compute @workgroup_size(64)
fn boids_cs(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&particles_src);
    let index = global_invocation_id.x;
    if index >= total {
        return;
    }

    let me = particles_src[index];

    var separation_vel = vec2f(0.);
    var alignment_vel = vec2f(0.);
    var alignment_count: u32 = 0;
    var center_of_mass = vec2f(0.);
    var cohesion_count: u32 = 0;

    for (var i: u32 = 0u; i < total; i++) {
        if i == index {
            continue;
        }

        let other = particles_src[i];

        if distance(me.pos, other.pos) < params.separation_distance {
            separation_vel += me.pos - other.pos;
        }
        if distance(me.pos, other.pos) < params.alignment_distance {
            alignment_vel += other.vel;
            alignment_count += 1u;
        }
        if distance(me.pos, other.pos) < params.cohesion_distance {
            center_of_mass += other.pos;
            cohesion_count += 1u;
        }
    }
    if alignment_count > 0 {
        alignment_vel /= f32(alignment_count);
    }
    var cohesion_vel = vec2f(0.);
    if cohesion_count > 0 {
        cohesion_vel = (center_of_mass / f32(cohesion_count)) - me.pos;
    }

    var new_particle: Particle = me;
    new_particle.vel += separation_vel * params.separation_scale;
    new_particle.vel += alignment_vel * params.alignment_scale;
    new_particle.vel += cohesion_vel * params.cohesion_scale;

    // clamp velocity for a more pleasing simulation
    new_particle.vel = normalize(new_particle.vel) * clamp(length(new_particle.vel), 0.0, 0.1);

    // kinematic update
    new_particle.pos += new_particle.vel * params.delta_time;

    // Wrap around boundary
    new_particle.pos = 2. * fract(0.5 + 0.5 * new_particle.pos) - 1.;

    // Write back
    particles_dst[index] = new_particle;
}

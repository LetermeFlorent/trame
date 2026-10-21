struct Globals {
    viewport: vec2<f32>,
    srgb_output: u32,
    padding: u32,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var mask_atlas: texture_2d<f32>;
@group(0) @binding(2) var color_atlas: texture_2d<f32>;

const KIND_FILL: u32 = 0u;
const KIND_BORDER: u32 = 1u;
const KIND_MASK: u32 = 2u;
const KIND_COLOR: u32 = 3u;
const KIND_LINE: u32 = 4u;
const KIND_SHADOW: u32 = 5u;

struct QuadInput {
    @location(0) rect: vec4<f32>,
    @location(1) uv: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(3) params: vec4<f32>,
    @location(4) clip: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) rect: vec4<f32>,
    @location(1) @interpolate(flat) uv: vec4<f32>,
    @location(2) @interpolate(flat) color: vec4<f32>,
    @location(3) @interpolate(flat) params: vec4<f32>,
    @location(4) @interpolate(flat) clip: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) index: u32, quad: QuadInput) -> VertexOutput {
    let corner = vec2<f32>(f32(index & 1u), f32(index >> 1u));
    var grow = 0.0;
    if u32(quad.params.x) == KIND_SHADOW {
        grow = quad.params.z * 2.0;
    }
    let origin = quad.rect.xy - vec2<f32>(grow);
    let size = quad.rect.zw + vec2<f32>(grow * 2.0);
    let pixel = origin + corner * size;
    let ndc = vec2<f32>(pixel.x / globals.viewport.x * 2.0 - 1.0, 1.0 - pixel.y / globals.viewport.y * 2.0);
    var out: VertexOutput;
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.rect = quad.rect;
    out.uv = quad.uv;
    out.color = quad.color;
    out.params = quad.params;
    out.clip = quad.clip;
    return out;
}

fn rounded_rect_distance(point: vec2<f32>, rect: vec4<f32>, radius: f32) -> f32 {
    let half = rect.zw * 0.5;
    let center = rect.xy + half;
    let r = min(radius, min(half.x, half.y));
    let q = abs(point - center) - half + vec2<f32>(r);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

fn coverage(distance: f32) -> f32 {
    return clamp(0.5 - distance, 0.0, 1.0);
}

fn segment_distance(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> f32 {
    let direction = end - start;
    let length_squared = max(dot(direction, direction), 0.0001);
    let t = clamp(dot(point - start, direction) / length_squared, 0.0, 1.0);
    return length(point - (start + direction * t));
}

fn texel(point: vec2<f32>, rect: vec4<f32>, uv: vec4<f32>) -> vec2<i32> {
    return vec2<i32>(floor(uv.xy + point - rect.xy));
}

fn text_alpha(alpha: f32, color: vec3<f32>) -> f32 {
    let luminance = dot(color, vec3<f32>(0.2126, 0.7152, 0.0722));
    return pow(alpha, mix(0.92, 0.72, luminance));
}

fn srgb_to_linear(color: vec3<f32>) -> vec3<f32> {
    let low = color / 12.92;
    let high = pow((color + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(high, low, color <= vec3<f32>(0.04045));
}

fn output(rgb: vec3<f32>, alpha: f32) -> vec4<f32> {
    var color = rgb;
    if globals.srgb_output == 1u {
        color = srgb_to_linear(color);
    }
    return vec4<f32>(color * alpha, alpha);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let point = in.position.xy;
    if point.x < in.clip.x || point.y < in.clip.y || point.x >= in.clip.z || point.y >= in.clip.w {
        discard;
    }
    let kind = u32(in.params.x);
    let radius = in.params.y;
    let width = in.params.z;
    var alpha = 1.0;
    switch kind {
        case KIND_FILL: {
            alpha = coverage(rounded_rect_distance(point, in.rect, radius));
        }
        case KIND_BORDER: {
            let outer = coverage(rounded_rect_distance(point, in.rect, radius));
            let inner_rect = vec4<f32>(in.rect.xy + vec2<f32>(width), in.rect.zw - vec2<f32>(width * 2.0));
            let inner = coverage(rounded_rect_distance(point, inner_rect, max(radius - width, 0.0)));
            alpha = clamp(outer - inner, 0.0, 1.0);
        }
        case KIND_MASK: {
            let mask = textureLoad(mask_atlas, texel(point, in.rect, in.uv), 0).r;
            alpha = text_alpha(mask, in.color.rgb);
        }
        case KIND_COLOR: {
            let sample = textureLoad(color_atlas, texel(point, in.rect, in.uv), 0);
            return output(sample.rgb, sample.a * in.color.a);
        }
        case KIND_LINE: {
            let distance = segment_distance(point, in.uv.xy, in.uv.zw) - width * 0.5;
            alpha = coverage(distance);
        }
        case KIND_SHADOW: {
            let distance = rounded_rect_distance(point, in.rect, radius);
            alpha = 1.0 - smoothstep(-width, width, distance);
        }
        default: {
            alpha = 0.0;
        }
    }
    return output(in.color.rgb, in.color.a * alpha);
}

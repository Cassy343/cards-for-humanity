precision highp float;

varying vec2 point_coord;

uniform vec4 color;
uniform vec2 dimensions;
uniform float corner_radius;
uniform float aspect_ratio;
uniform vec2 canvans_dimensions;

void main(void) {
    vec2 pixel_vec = abs(point_coord) - vec2((dimensions.x / 2.) - corner_radius / aspect_ratio, (dimensions.y / 2.) - corner_radius);
    pixel_vec.x *= aspect_ratio;

    if (pixel_vec.x > 0. && pixel_vec.y > 0.) {
        float alpha;
        if (length(pixel_vec) > corner_radius) {
            alpha = 1. - clamp((length(pixel_vec) - corner_radius) * 250., 0., 1.);
        } else {
            alpha = 1.;
        }
        gl_FragColor = vec4(color.x, color.y, color.z, alpha);
    } else {
        gl_FragColor = color;
    }
}
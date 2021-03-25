precision highp float;

varying vec2 point_coord;

uniform vec4 color;
uniform vec2 dimensions;
uniform float corner_radius;
uniform float aspect_ratio;

void main(void) {
    vec2 pixel_vec = abs(point_coord) - vec2((dimensions.x / 2.) - corner_radius / aspect_ratio, (dimensions.y / 2.) - corner_radius);

    if (pixel_vec.x > 0. && pixel_vec.y > 0.) {
        pixel_vec.x *= aspect_ratio;
        float alpha = clamp(length(pixel_vec) - corner_radius, 0., 1.);
        gl_FragColor = vec4(color.x, color.y, color.z, alpha);
    } else {
        gl_FragColor = color;
    }
}
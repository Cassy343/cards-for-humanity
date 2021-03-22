precision highp float;

varying vec2 point_coord;

uniform vec4 color;
uniform vec2 dimensions;
uniform float corner_radius;

void main(void) {

    vec2 pixel_vec = abs(point_coord) - vec2((dimensions.x/2.0)-corner_radius,(dimensions.y/2.0)-corner_radius);

    if(pixel_vec.x > 0.0 && pixel_vec.y > 0.0 && length(pixel_vec) > corner_radius) {
        discard;
    } else {
        gl_FragColor=color;
    }
}
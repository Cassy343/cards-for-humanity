// uniform vec4 vColor;

attribute vec2 a_position;

varying vec2 point_coord;

uniform vec2 center_position;
uniform vec2 dimensions;


void main() {
    point_coord = a_position;
    gl_Position = vec4((a_position * dimensions / 2.) + center_position, 0, 1);
}
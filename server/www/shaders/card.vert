// uniform vec4 vColor;

attribute vec2 vertex;

varying vec2 point_coord;

uniform vec2 position;
uniform vec2 dimensions;


void main() {
    point_coord = vertex;
    gl_Position = vec4((vertex) + position, 0, 1);
}
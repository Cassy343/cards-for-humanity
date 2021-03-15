//Version 432

precision highp float;

uniform vec4 vColor;

// uniform vec2 u_dimensions;
// uniform float u_radius;
// uniform int test;

void main(void){
    // vec2 coords = 1.0 * u_dimensions;
    // if(length(coords-vec2(0))<u_radius||
    // length(coords-vec2(0,u_dimensions.y))<u_radius||
    // length(coords-vec2(u_dimensions.x,0))<u_radius||
    // length(coords-u_dimensions)<u_radius){
    //     discard;
    // }
    // Do everything else otherwise
    gl_FragColor=vColor;
}
// uniform vec4 vColor;

attribute vec2 a_position;


void main(){
    gl_Position = vec4(a_position, 0, 1);
}
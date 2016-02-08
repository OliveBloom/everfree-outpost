precision mediump float;

uniform vec2 screenSize;
uniform vec2 sheetSize;

attribute vec2 source;
attribute vec2 dest;

varying vec2 texCoord;

void main(void) {
    vec2 pos = dest / screenSize * 2.0 - 1.0;
    pos.y = -pos.y;
    gl_Position = vec4(pos, 0.5, 1.0);
    texCoord = source / sheetSize;
}

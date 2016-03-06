precision mediump float;

uniform vec2 screenSize;
uniform vec2 sheetSize;

attribute vec2 srcPos;
attribute vec2 srcSize;
attribute vec2 srcStepPx;
attribute vec2 dest;

varying vec2 texPos;
varying vec2 texSize;
varying vec2 texSteps;

void main(void) {
    vec2 pos = dest / screenSize * 2.0 - 1.0;
    pos.y = -pos.y;
    gl_Position = vec4(pos, 0.5, 1.0);

    texPos = srcPos / sheetSize;
    texSize = srcSize / sheetSize;
    texSteps = srcStepPx / srcSize;
}

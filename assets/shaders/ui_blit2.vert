precision mediump float;

uniform vec2 screenSize;
uniform vec2 sheetSize[3];

attribute vec2 srcPos;
attribute vec2 srcSize;
attribute vec2 dest;
attribute vec2 offset_;
attribute float sheetAttr;

varying vec2 texPos;
varying vec2 texSize;
varying vec2 texSteps;
varying float sheet;

void main(void) {
    vec2 pos = (dest + offset_) / screenSize * 2.0 - 1.0;
    pos.y = -pos.y;
    gl_Position = vec4(pos, 0.5, 1.0);


    sheet = sheetAttr;
    texPos = srcPos / sheetSize[int(sheet)];
    texSize = srcSize / sheetSize[int(sheet)];
    texSteps = offset_ / srcSize;
}

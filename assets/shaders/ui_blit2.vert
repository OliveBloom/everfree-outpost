precision mediump float;

uniform vec2 screenSize;
uniform vec2 sheetSize0;
uniform vec2 sheetSize1;
uniform vec2 sheetSize2;

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

    vec2 sheetSize = vec2(0.0);
    if (int(sheetAttr) == 0) {
        // Special hack for the item sheet.  In the sheet, all items are 32x32,
        // but we render at 16x16 (many icons are upscaled 16x16 -> 32x32).
        sheetSize = sheetSize0 / 2.0;
    } else if (int(sheetAttr) == 1) {
        sheetSize = sheetSize1;
    } else if (int(sheetAttr) == 2) {
        sheetSize = sheetSize2;
    }

    sheet = sheetAttr;
    texPos = srcPos / sheetSize;
    texSize = srcSize / sheetSize;
    texSteps = offset_ / srcSize;
}

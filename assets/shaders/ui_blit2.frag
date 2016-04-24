precision mediump float;

uniform sampler2D sheet0;
uniform sampler2D sheet1;
uniform sampler2D sheet2;

varying vec2 texPos;
varying vec2 texSize;
varying vec2 texSteps;
varying float sheet;

void main(void) {
    vec2 fracPart = mod(texSteps, 1.0);

    if (sheet == 0.0) {
        gl_FragColor = texture2D(sheet0, texPos + fracPart * texSize);
    } else if (sheet == 1.0) {
        gl_FragColor = texture2D(sheet1, texPos + fracPart * texSize);
    } else {
        gl_FragColor = texture2D(sheet2, texPos + fracPart * texSize);
    }
}

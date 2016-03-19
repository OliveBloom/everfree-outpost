precision mediump float;

uniform sampler2D sheet;

varying vec2 texPos;
varying vec2 texSize;
varying vec2 texSteps;

void main(void) {
    vec2 fracPart = mod(texSteps, 1.0);

    gl_FragColor = texture2D(sheet, texPos + fracPart * texSize);
}

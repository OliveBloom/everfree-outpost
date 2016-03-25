precision mediump float;

uniform sampler2D sheets[3];

varying vec2 texPos;
varying vec2 texSize;
varying vec2 texSteps;
varying float sheet;

void main(void) {
    vec2 fracPart = mod(texSteps, 1.0);

    if (sheet == 0.0) {
        gl_FragColor = texture2D(sheets[0], texPos + fracPart * texSize);
    } else if (sheet == 1.0) {
        gl_FragColor = texture2D(sheets[1], texPos + fracPart * texSize);
    } else {
        gl_FragColor = texture2D(sheets[2], texPos + fracPart * texSize);
    }
    //gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
}

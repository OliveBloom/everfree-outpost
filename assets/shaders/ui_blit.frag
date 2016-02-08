precision mediump float;

uniform sampler2D sheet;

varying vec2 texCoord;

void main(void) {
    gl_FragColor = texture2D(sheet, texCoord);
}

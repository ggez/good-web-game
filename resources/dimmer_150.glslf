#version 100

varying lowp vec4 color;
varying lowp vec2 uv;

uniform sampler2D Texture;
uniform lowp float u_Rate;

void main() {
    gl_FragColor = texture2D(Texture, uv) * color * u_Rate;
}
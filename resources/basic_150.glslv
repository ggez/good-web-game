#version 100

attribute vec2 position;
attribute vec2 texcoord;
attribute vec4 color0;

attribute vec4 Source;
attribute vec4 Color;
attribute mat4 Model;

varying lowp vec4 color;
varying lowp vec2 uv;

uniform mat4 Projection;

uniform float depth;

void main() {
    gl_Position = Projection * Model * vec4(position, 0, 1);
    gl_Position.z = depth;
    color = Color * color0;
    uv = texcoord * Source.zw + Source.xy;
}
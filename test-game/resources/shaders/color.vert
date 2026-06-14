#version 460 core
layout (location = 0) in vec3 vert_pos;
layout (location = 1) in vec4 vert_color;

out vec4 color;

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;
uniform float scale;

void main() {
    color = vert_color;

    vec4 pos = vec4(vert_pos*scale, 1.0);

    gl_Position = proj * view * model * pos;
}
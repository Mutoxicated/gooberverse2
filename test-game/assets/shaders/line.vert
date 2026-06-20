#version 460 core
layout (location = 0) in vec3 vert_pos;
layout (location = 1) in vec4 vert_color;
layout (location = 3) in vec3 vert_bary;

out vec3 fragPos; // in world coordinate space
out vec4 color;
out vec3 barycentric;

uniform mat4 proj;
uniform mat4 view;
uniform mat4 model;
uniform float scale;

void main() {
    color = vert_color;
    barycentric = vert_bary;
    vec4 pos = vec4(vert_pos*scale+(normalize(vert_pos)*0.001), 1.0);
    fragPos = (model * pos).xyz;
    gl_Position = proj * view * model * pos;
}
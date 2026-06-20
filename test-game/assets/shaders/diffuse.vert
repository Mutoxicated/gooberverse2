#version 460 core
layout (location = 0) in vec3 vert_pos;
layout (location = 1) in vec4 vert_color;
layout (location = 2) in vec3 normal;

out vec4 color;
out vec3 fragNormal;
out vec3 fragPos;

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;
uniform float scale;

void main()
{
    vec4 pos = vec4(vert_pos*scale, 1.0);

    mat3 model_rot = mat3(
        vec3(model[0][0], model[0][1], model[0][2]),
        vec3(model[1][0], model[1][1], model[1][2]),
        vec3(model[2][0], model[2][1], model[2][2])
    );

    fragNormal = model_rot * normal;
    fragPos = vec3(model * pos);

    color = vert_color;
    gl_Position = proj * view * model * pos;
}
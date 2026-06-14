#version 460 core
in vec4 color;
in vec3 fragNormal;
in vec3 fragPos;

out vec4 fragColor;

uniform vec3 lightPos;
uniform float ambientStrength;

void main()
{
    vec3 difference = lightPos-fragPos;
    //float inverse_dist = 1/sqrt(difference.x*difference.x+difference.y*difference.y+difference.z*difference.z);
    vec3 lightDirection = normalize(difference);
    float diff = dot(fragNormal, lightDirection)*0.15/* *inverse_dist */;
    float a = ambientStrength+diff;
    fragColor = vec4(vec3(1.0)*a, 1.0);
}
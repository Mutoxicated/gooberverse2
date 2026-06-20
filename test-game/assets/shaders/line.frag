#version 460 core

in vec4 color;
in vec3 barycentric;
in vec3 fragPos;

out vec4 fragColor;

uniform vec3 camPos;

void main() {
    vec3 diff = abs(camPos-fragPos);
    float dist = (diff.x+diff.y+diff.z)*0.333;
    float t = clamp((dist-5.0)/(1.0-5.0), 0.1, 1.0);
    vec3 unitWidth = fwidth(barycentric);
    // Alias the line a bit.
    vec3 aliased = smoothstep(vec3(0.0, 0.0, 0.0), unitWidth * 2.0 * t, barycentric);
    // Use the coordinate closest to the edge.
    float alpha = 1.0 - min(aliased.x, min(aliased.y, aliased.z));
    if (alpha == 0)
        discard;
    fragColor = vec4(color.rgb, alpha);
}
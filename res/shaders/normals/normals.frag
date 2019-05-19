#version 150

in vec3 v_normal;
out vec4 color;

void main() {
    vec3 normal_color = vec3(v_normal.x + 1, v_normal.y + 1, v_normal.z + 1) / 2.0;
    color = vec4(normal_color * normal_color, 1.0);
}

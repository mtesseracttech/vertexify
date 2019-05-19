#version 150
in vec4 vertex_color;
in vec3 vertex_normal;

out vec4 color;
void main()
{
    //vec3 normal_color = vec3(vertex_normal.x, vertex_normal.y, vertex_normal.z);
    //color = vec4(normal_color, 1.0);
    color = vertex_color;
}
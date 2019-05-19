#version 150
in vec3 position;
in vec3 normal;

uniform mat4 view;
uniform mat4 model;
uniform mat4 perspective;

out Vertex
{
    vec4 normal;
    vec4 color;
} vertex;

out vec3 vertex_normal;

void main()
{
    gl_Position = vec4(position, 1.0);
    vertex.normal = vec4(normal, 1.0);
    vertex_normal = normal;
    vertex.color =  vec4(1.0, 1.0, 0.0, 1.0);
}
#version 330 core

in vec3 position;
in vec2 tex_coords;

out vec2 TexCoords;

uniform mat4 vp;
uniform mat4 model;

void main() {
    gl_Position = vp * model * vec4(position, 1.0);

    TexCoords = tex_coords;
}

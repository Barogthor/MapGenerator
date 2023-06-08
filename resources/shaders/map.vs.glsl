#version 330 core

in vec3 position;
in vec3 color;

out vec3 oColor;

uniform mat4 vp;
uniform mat4 model;

void main() {
    gl_Position = vp * model * vec4(position, 1.0);

    oColor = color;
}

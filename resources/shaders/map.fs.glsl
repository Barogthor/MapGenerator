#version 330 core

out vec4 FragColor;

in vec3 oColor;

uniform vec3 uColor;
uniform sampler2D tex;

void main()
{
    FragColor = vec4(oColor, 1.0);
}
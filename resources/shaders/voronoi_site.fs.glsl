#version 330 core

in vec3 uColor;
out vec4 FragColor;

in vec2 TexCoords;


void main()
{
    FragColor = vec4(uColor, 1.0);
}
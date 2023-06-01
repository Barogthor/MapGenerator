#version 330 core

out vec4 FragColor;

in vec2 TexCoords;

uniform vec3 uColor;
uniform sampler2D tex;

void main()
{
    FragColor = texture(tex, TexCoords);
    //    FragColor = vec4(uColor, 1.0);
}
#version 450

layout(set = 1, binding = 0) uniform ModelMatrix {
    mat4 input_matrix;
};

layout(location = 0) in vec3 input_position;
layout(location = 1) in vec2 input_uv;
layout(location = 2) in vec3 input_normal;

layout(location = 0) out vec3 output_position;
layout(location = 1) out vec2 output_uv;
layout(location = 2) out vec3 output_normal;
layout(location = 3) out mat4 output_matrix;

void main() {
    output_position = input_position;
    output_uv = input_uv;
    output_normal = input_normal;
    output_matrix = input_matrix;
    gl_Position = vec4(input_uv, 0.0, 1.0);
}


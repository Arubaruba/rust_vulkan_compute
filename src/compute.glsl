#version 450

#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout(set = 0, binding = 0) buffer Data {
    int asdf;
    int arr[2];
} buffers;

void main()
{	
    buffers.arr[0] = buffers.arr[1];
}
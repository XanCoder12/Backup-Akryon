; Nyxara userland: print a greeting via write(2), then exit(2).

[BITS 32]

SYS_EXIT equ 1
SYS_WRITE equ 4

section .rodata
hello_msg db "Hello from Nyxara userland!", 10
hello_len equ $ - hello_msg

section .text
global _start
_start:
    mov eax, SYS_WRITE
    mov ebx, 1
    mov ecx, hello_msg
    mov edx, hello_len
    int 0x80

    mov eax, SYS_EXIT
    xor ebx, ebx
    int 0x80

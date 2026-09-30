; Nyxara userland: fork(2) + waitpid(2) demo.
; The child prints its pid and exits with code 42; the parent waits,
; prints the exit code, and exits cleanly.

[BITS 32]

SYS_EXIT equ 1
SYS_FORK equ 2
SYS_WRITE equ 4
SYS_WAITPID equ 7
SYS_GETPID equ 20

section .data
child_pid dd 0
status dd 0

section .rodata
fork_msg db "forktest: parent forked, child pid "
fork_len equ $ - fork_msg
child_msg db "forktest: child running, my pid is "
child_len equ $ - child_msg
exit_msg db "forktest: parent: child exited with code "
exit_len equ $ - exit_msg
newline db 10

section .text
global _start
_start:
    mov eax, SYS_FORK
    int 0x80
    test eax, eax
    jz .child

    mov [child_pid], eax
    mov eax, SYS_WRITE
    mov ebx, 1
    mov ecx, fork_msg
    mov edx, fork_len
    int 0x80
    mov ebx, [child_pid]
    call print_dec
    call print_nl

    mov eax, SYS_WAITPID
    mov ebx, [child_pid]
    mov ecx, status
    xor edx, edx
    int 0x80
    mov eax, SYS_WRITE
    mov ebx, 1
    mov ecx, exit_msg
    mov edx, exit_len
    int 0x80
    mov ebx, [status]
    call print_dec
    call print_nl

    mov eax, SYS_EXIT
    xor ebx, ebx
    int 0x80

.child:
    mov eax, SYS_GETPID
    int 0x80
    mov [child_pid], eax
    mov eax, SYS_WRITE
    mov ebx, 1
    mov ecx, child_msg
    mov edx, child_len
    int 0x80
    mov ebx, [child_pid]
    call print_dec
    call print_nl

    mov eax, SYS_EXIT
    mov ebx, 42
    int 0x80

print_dec:
    sub esp, 16
    lea edi, [esp + 15]
    mov eax, ebx
    mov esi, 10
.digit:
    xor edx, edx
    div esi
    add dl, '0'
    mov [edi], dl
    dec edi
    test eax, eax
    jnz .digit
    inc edi
    mov ecx, edi
    mov edx, esp
    add edx, 16
    sub edx, ecx
    mov eax, SYS_WRITE
    mov ebx, 1
    int 0x80
    add esp, 16
    ret

print_nl:
    mov eax, SYS_WRITE
    mov ebx, 1
    mov ecx, newline
    mov edx, 1
    int 0x80
    ret

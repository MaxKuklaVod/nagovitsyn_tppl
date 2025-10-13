BITS 64

section .data
    file_path   db "data.txt", 0
    ARRAY_LEN   equ 7

    output_msg  db "Arithmetic mean of the difference from file: "
    msg_len     equ $ - output_msg

section .bss
    array_x     resd ARRAY_LEN
    array_y     resd ARRAY_LEN
    result      resd 1
    file_buffer resb 1024

section .text
    global _start

_start:
    mov rax, 2
    mov rdi, file_path
    xor rsi, rsi
    xor rdx, rdx
    syscall
    mov r12, rax

    mov rax, 0
    mov rdi, r12
    mov rsi, file_buffer
    mov rdx, 1024
    syscall

    mov rax, 3
    mov rdi, r12
    syscall

    mov rsi, file_buffer
    mov rdi, array_x
    call process_buffer_to_array
    mov rdi, array_y
    call process_buffer_to_array

    mov r11, ARRAY_LEN
    mov r9, array_x
    mov r10, array_y
    xor r8, r8

sum_differences:
    mov eax, [r9]
    sub eax, [r10]
    add r8d, eax
    add r9, 4
    add r10, 4
    dec r11
    jnz sum_differences

    mov eax, r8d
    cdq
    mov ecx, ARRAY_LEN
    idiv ecx
    mov [result], eax

    mov rax, 1
    mov rdi, 1
    mov rsi, output_msg
    mov rdx, msg_len
    syscall

    mov eax, [result]
    call integer_to_string_and_print

    mov rax, 60
    xor rdi, rdi
    syscall

process_buffer_to_array:
    push rdi
    mov ecx, ARRAY_LEN
.main_parse_loop:
    call parse_single_integer
    mov [rdi], eax
    add rdi, 4
    loop .main_parse_loop
    pop rdi
    ret

parse_single_integer:
    xor eax, eax
.find_digit:
    cmp byte [rsi], ' '
    je .next_char_in_buffer
    cmp byte [rsi], 10
    je .next_char_in_buffer
    cmp byte [rsi], 13
    je .next_char_in_buffer
    jmp .parse_loop_start
.next_char_in_buffer:
    inc rsi
    jmp .find_digit
.parse_loop_start:
    movzx edx, byte [rsi]
    cmp edx, '0'
    jl .finished_parsing
    cmp edx, '9'
    jg .finished_parsing
    sub edx, '0'
    imul eax, 10
    add eax, edx
    inc rsi
    jmp .parse_loop_start
.finished_parsing:
    ret

integer_to_string_and_print:
    mov rdi, file_buffer + 1023
    mov byte [rdi], 10
    dec rdi
    mov r13, 10
    mov r14, 1
    test eax, eax
    jns .conversion_loop
    neg eax
    mov r14, -1
.conversion_loop:
    cmp eax, 0
    je .handle_zero_case
.convert_digit:
    xor edx, edx
    div r13
    add dl, '0'
    mov [rdi], dl
    dec rdi
    test eax, eax
    jnz .convert_digit
    jmp .check_for_sign
.handle_zero_case:
    mov byte[rdi], '0'
    dec rdi
.check_for_sign:
    cmp r14, 0
    jg .print_output
    mov byte [rdi], '-'
    dec rdi
.print_output:
    inc rdi
    mov rsi, rdi
    mov rdx, file_buffer + 1024
    sub rdx, rsi
    mov rax, 1
    mov rdi, 1
    syscall
    ret
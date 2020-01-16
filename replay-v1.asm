rav1e::ec::WriterBase<rav1e::ec::WriterRecorder>::replay (src/ec.rs:429):
 push    rbp
 mov     rbp, rsp
 push    r15
 push    r14
 push    r13
 push    r12
 push    rbx
 sub     rsp, 24
 mov     qword, ptr, [rbp, -, 56], rsi
 mov     r15, rdi
 lea     rax, [rdi, +, 16]
 mov     qword, ptr, [rbp, -, 48], rax
 mov     r13, qword, ptr, [rdi, +, 16]
 test    r13, r13
 je      LBB251_5
 mov     r14, qword, ptr, [rdx, +, 24]
 dec     r13
 mov     r12d, 4
 xor     ebx, ebx
LBB251_2:
 mov     rax, qword, ptr, [r15]
 movzx   ecx, word, ptr, [rax, +, r12]
 movzx   edx, word, ptr, [rax, +, r12, -, 2]
 movzx   esi, word, ptr, [rax, +, r12, -, 4]
 mov     rdi, qword, ptr, [rbp, -, 56]
 call    r14
 cmp     r13, rbx
 je      LBB251_5
 inc     rbx
 mov     rax, qword, ptr, [rbp, -, 48]
 mov     rdx, qword, ptr, [rax]
 add     r12, 6
 cmp     rdx, rbx
 ja      LBB251_2
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.63]
 mov     rsi, rbx
 call    core::panicking::panic_bounds_check
LBB251_5:
 mov     dword, ptr, [r15, +, 36], -557056
 mov     rax, qword, ptr, [rbp, -, 48]
 mov     qword, ptr, [rax, +, 8], 0
 mov     qword, ptr, [rax], 0
 add     rsp, 24
 pop     rbx
 pop     r12
 pop     r13
 pop     r14
 pop     r15
 pop     rbp
 ret

rav1e::ec::WriterBase<rav1e::ec::WriterRecorder>::replay (src/ec.rs:429):
 push    rbp
 mov     rbp, rsp
 push    r15
 push    r14
 push    r13
 push    r12
 push    rbx
 push    rax
 mov     r14, rdi
 mov     rax, qword, ptr, [rdi, +, 16]
 lea     rcx, [rdi, +, 16]
 mov     qword, ptr, [rbp, -, 48], rcx
 test    rax, rax
 je      LBB251_3
 mov     r12, rsi
 mov     rbx, qword, ptr, [r14]
 lea     rax, [rax, +, 2*rax]
 lea     r13, [rbx, +, 2*rax]
 mov     r15, qword, ptr, [rdx, +, 24]
LBB251_2:
 movzx   ecx, word, ptr, [rbx, +, 4]
 movzx   edx, word, ptr, [rbx, +, 2]
 movzx   esi, word, ptr, [rbx]
 mov     rdi, r12
 call    r15
 add     rbx, 6
 cmp     rbx, r13
 jne     LBB251_2
LBB251_3:
 mov     dword, ptr, [r14, +, 36], -557056
 mov     rax, qword, ptr, [rbp, -, 48]
 mov     qword, ptr, [rax, +, 8], 0
 mov     qword, ptr, [rax], 0
 add     rsp, 8
 pop     rbx
 pop     r12
 pop     r13
 pop     r14
 pop     r15
 pop     rbp
 ret

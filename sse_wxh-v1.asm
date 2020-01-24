file /rustc/73528e339aae0f17a15ffa49a8ac608f50c6cf14/src/libcore/num/mod.rs does not exist!
file /rustc/73528e339aae0f17a15ffa49a8ac608f50c6cf14/src/libcore/iter/range.rs does not exist!
file /rustc/73528e339aae0f17a15ffa49a8ac608f50c6cf14/src/libcore/slice/mod.rs does not exist!
file /rustc/73528e339aae0f17a15ffa49a8ac608f50c6cf14/src/libcore/iter/traits/iterator.rs does not exist!
file /rustc/73528e339aae0f17a15ffa49a8ac608f50c6cf14/src/libcore/ops/arith.rs does not exist!
file /rustc/73528e339aae0f17a15ffa49a8ac608f50c6cf14/src/libcore/cmp.rs does not exist!
file /rustc/73528e339aae0f17a15ffa49a8ac608f50c6cf14/src/libcore/iter/adapters/zip.rs does not exist!
file <::std::macros::panic macros> does not exist!
rav1e::rdo::sse_wxh (src/rdo.rs:236):
 push    rbp
 push    r15
 push    r14
 push    r13
 push    r12
 push    rbx
 sub     rsp, 168
 mov     qword, ptr, [rsp], rdx
 test    dl, 3
 jne     .LBB417_6
 mov     r12, rcx
 test    r12b, 3
 jne     .LBB417_8
 mov     r14, rsi
 mov     r13, rdi
 mov     rax, qword, ptr, [rsp]
 cmp     rax, 8
 mov     ebx, 8
 cmovb   rbx, rax
 mov     ebp, 8
 cmp     r12, 8
 cmovb   rbp, r12
 mov     rdi, rbx
 mov     rsi, rbp
 call    rav1e::partition::BlockSize::from_width_and_height
 mov     rax, qword, ptr, [r13, +, 8]
 mov     cl, byte, ptr, [rax, +, 32]
 mov     qword, ptr, [rsp, +, 160], rax
 mov     al, byte, ptr, [rax, +, 40]
 shr     rbx, cl
 mov     ecx, eax
 shr     rbp, cl
 test    rbp, rbp
 je      .LBB417_30
 mov     rax, r12
 xor     edx, edx
 div     rbp
 mov     qword, ptr, [rsp, +, 104], rax
 cmp     rbp, r12
 jbe     .LBB417_9
 xor     eax, eax
 jmp     .LBB417_5
.LBB417_9:
 test    rbx, rbx
 je      .LBB417_32
 mov     rax, qword, ptr, [r13, +, 40]
 mov     qword, ptr, [rsp, +, 8], rax
 mov     rcx, qword, ptr, [r13]
 mov     rax, qword, ptr, [r13, +, 32]
 mov     qword, ptr, [rsp, +, 24], rax
 mov     rax, qword, ptr, [r14, +, 40]
 mov     qword, ptr, [rsp, +, 152], rax
 mov     rax, qword, ptr, [r14, +, 32]
 mov     qword, ptr, [rsp, +, 48], rax
 mov     rax, qword, ptr, [r14]
 mov     qword, ptr, [rsp, +, 88], rax
 mov     rax, qword, ptr, [r14, +, 8]
 mov     qword, ptr, [rsp, +, 144], rax
 mov     qword, ptr, [rsp, +, 96], rcx
 neg     rcx
 mov     qword, ptr, [rsp, +, 80], rcx
 mov     rax, qword, ptr, [rsp]
 xor     edx, edx
 div     rbx
 mov     qword, ptr, [rsp, +, 120], rax
 xor     eax, eax
 mov     qword, ptr, [rsp, +, 40], rax
 xor     eax, eax
 xor     ecx, ecx
 mov     qword, ptr, [rsp, +, 32], rcx
 xor     ecx, ecx
 cmp     rbx, qword, ptr, [rsp]
 mov     qword, ptr, [rsp, +, 112], rcx
 jbe     .LBB417_13
.LBB417_11:
 mov     rcx, qword, ptr, [rsp, +, 112]
 add     rcx, 1
 sub     qword, ptr, [rsp, +, 32], rbp
 add     qword, ptr, [rsp, +, 40], rbp
 cmp     rcx, qword, ptr, [rsp, +, 104]
 jae     .LBB417_5
 cmp     rbx, qword, ptr, [rsp]
 mov     qword, ptr, [rsp, +, 112], rcx
 ja      .LBB417_11
.LBB417_13:
 imul    rcx, rbp
 mov     qword, ptr, [rsp, +, 16], rcx
 mov     rcx, qword, ptr, [rsp, +, 88]
 mov     qword, ptr, [rsp, +, 72], rcx
 mov     rcx, qword, ptr, [rsp, +, 96]
 mov     qword, ptr, [rsp, +, 64], rcx
 mov     rcx, qword, ptr, [rsp, +, 80]
 mov     qword, ptr, [rsp, +, 56], rcx
 xor     edi, edi
 jmp     .LBB417_14
.LBB417_29:
 movabs  rax, 4503599627370495
 and     rdi, rax
 mov     rax, qword, ptr, [rsp, +, 136]
 add     rax, rdi
 sub     qword, ptr, [rsp, +, 56], rbx
 add     qword, ptr, [rsp, +, 64], rbx
 add     qword, ptr, [rsp, +, 72], rbx
 mov     rdi, qword, ptr, [rsp, +, 128]
 cmp     rdi, qword, ptr, [rsp, +, 120]
 jae     .LBB417_11
.LBB417_14:
 lea     rcx, [rdi, +, 1]
 imul    rdi, rbx
 mov     rdx, rcx
 imul    rdx, rbx
 cmp     rdx, rdi
 jb      .LBB417_15
 mov     qword, ptr, [rsp, +, 128], rcx
 mov     qword, ptr, [rsp, +, 136], rax
 cmp     qword, ptr, [rsp, +, 24], rdx
 jb      .LBB417_21
 mov     r14, qword, ptr, [rsp, +, 40]
 mov     rax, qword, ptr, [rsp, +, 32]
 xor     edi, edi
 xor     r15d, r15d
 mov     rcx, qword, ptr, [rsp, +, 16]
 add     rcx, r15
 cmp     qword, ptr, [rsp, +, 8], rcx
 ja      .LBB417_23
 jmp     .LBB417_20
.LBB417_28:
 mov     ecx, ebp
 add     rdi, rcx
 add     rax, -1
 add     r14, 1
 mov     rbp, r11
 cmp     r15, r11
 je      .LBB417_29
 mov     rcx, qword, ptr, [rsp, +, 16]
 add     rcx, r15
 cmp     qword, ptr, [rsp, +, 8], rcx
 jbe     .LBB417_20
.LBB417_23:
 cmp     qword, ptr, [rsp, +, 152], rcx
 jbe     .LBB417_20
 cmp     qword, ptr, [rsp, +, 48], rdx
 jb      .LBB417_33
 mov     r11, rbp
 add     r15, 1
 mov     rcx, qword, ptr, [rsp, +, 160]
 mov     r10, qword, ptr, [rcx]
 mov     r12, r10
 imul    r12, rax
 add     r12, qword, ptr, [rsp, +, 56]
 imul    r10, r14
 add     r10, qword, ptr, [rsp, +, 64]
 mov     rcx, qword, ptr, [rsp, +, 144]
 mov     rcx, qword, ptr, [rcx]
 imul    rcx, r14
 add     rcx, qword, ptr, [rsp, +, 72]
 xor     r13d, r13d
 xor     ebp, ebp
.LBB417_26:
 cmp     r12, r13
 je      .LBB417_28
 lea     r8, [rcx, +, r13]
 movzx   r9d, byte, ptr, [r10, +, r13]
 add     r13, 1
 movzx   esi, byte, ptr, [r8]
 sub     r9d, esi
 imul    r9d, r9d
 add     ebp, r9d
 cmp     rbx, r13
 jne     .LBB417_26
 jmp     .LBB417_28
.LBB417_5:
 add     rsp, 168
 pop     rbx
 pop     r12
 pop     r13
 pop     r14
 pop     r15
 pop     rbp
 ret
.LBB417_33:
 mov     rdi, rdx
 mov     rsi, qword, ptr, [rsp, +, 48]
 call    core::slice::slice_index_len_fail
 ud2
.LBB417_15:
 mov     rax, qword, ptr, [rsp, +, 16]
 cmp     qword, ptr, [rsp, +, 8], rax
 jbe     .LBB417_20
 mov     rsi, rdx
 call    core::slice::slice_index_order_fail
 ud2
.LBB417_21:
 mov     rax, qword, ptr, [rsp, +, 16]
 cmp     qword, ptr, [rsp, +, 8], rax
 jbe     .LBB417_20
 mov     rdi, rdx
 mov     rsi, qword, ptr, [rsp, +, 24]
 call    core::slice::slice_index_len_fail
 ud2
.LBB417_20:
 lea     rdi, [rip, +, .L__unnamed_173]
 lea     rdx, [rip, +, .L__unnamed_55]
 mov     esi, 42
 call    std::panicking::begin_panic
 ud2
.LBB417_6:
 lea     rdi, [rip, +, .L__unnamed_403]
 lea     rdx, [rip, +, .L__unnamed_404]
 mov     esi, 40
 call    std::panicking::begin_panic
 ud2
.LBB417_8:
 lea     rdi, [rip, +, .L__unnamed_405]
 lea     rdx, [rip, +, .L__unnamed_406]
 mov     esi, 40
 call    std::panicking::begin_panic
 ud2
.LBB417_30:
 lea     rdi, [rip, +, str.1]
 lea     rdx, [rip, +, .L__unnamed_407]
 mov     esi, 25
 call    core::panicking::panic
 ud2
.LBB417_32:
 lea     rdi, [rip, +, str.1]
 lea     rdx, [rip, +, .L__unnamed_408]
 mov     esi, 25
 call    core::panicking::panic
 ud2

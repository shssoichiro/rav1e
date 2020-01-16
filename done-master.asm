 pub fn done(&mut self) -> Vec<u8> {
 push    rbp
 mov     rbp, rsp
 push    r15
 push    r14
 push    r13
 push    r12
 push    rbx
 sub     rsp, 56
 mov     r8, rsi
 mov     qword, ptr, [rbp, -, 96], rdi
 let mut c = self.cnt;
 movzx   ecx, word, ptr, [rsi, +, 38]
 s += c;
 lea     r14d, [rcx, +, 10]
 if s > 0 {
 test    r14w, r14w
 mov     qword, ptr, [rbp, -, 88], rsi
 if s > 0 {
 jle     LBB252_1
 mov     r9d, 16383
 let mut e = ((l + m) & !m) | (m + 1);
 add     r9d, dword, ptr, [r8, +, 24]
 let mut n = (1 << (c + 16)) - 1;
 add     cl, 16
 mov     edx, -1
 shl     edx, cl
 let mut e = ((l + m) & !m) | (m + 1);
 and     r9d, -32768
 or      r9d, 16384
 let mut n = (1 << (c + 16)) - 1;
 not     edx
     mov     r15, qword, ptr, [r8, +, 16]
     jmp     LBB252_3
LBB252_4:
     mov     rax, qword, ptr, [r8]
LBB252_12:
     mov     word, ptr, [rax, +, 2*r15], di
     mov     r15, qword, ptr, [r8, +, 16]
     inc     r15
     mov     qword, ptr, [r8, +, 16], r15
 e &= n;
 and     r9d, edx
 s -= 8;
 add     r14d, -8
 n >>= 8;
 shr     edx, 8
 if s <= 0 {
 test    r14w, r14w
 if s <= 0 {
 jle     LBB252_13
LBB252_3:
 self.s.precarry.push((e >> (c + 16)) as u16);
 lea     ecx, [r14, +, 6]
 mov     edi, r9d
 shr     edi, cl
     cmp     r15, qword, ptr, [r8, +, 8]
     jne     LBB252_4
     mov     rbx, r15
     inc     rbx
     je      LBB252_26
     lea     rsi, [r15, +, r15]
     cmp     rsi, rbx
     cmova   rbx, rsi
     xor     r12d, r12d
     mov     rax, rbx
     add     rax, rbx
     setae   al
     mov     r13, rbx
     add     r13, rbx
     jb      LBB252_26
     mov     dword, ptr, [rbp, -, 44], edi
     mov     dword, ptr, [rbp, -, 48], edx
     mov     dword, ptr, [rbp, -, 52], r9d
     mov     r12b, al
     add     r12, r12
     test    r15, r15
     je      LBB252_8
     mov     rdi, qword, ptr, [r8]
     mov     edx, 2
     mov     rcx, r13
     call    ___rust_realloc
     jmp     LBB252_10
LBB252_8:
     mov     rdi, r13
     mov     rsi, r12
     call    ___rust_alloc
LBB252_10:
     test    rax, rax
     mov     r9d, dword, ptr, [rbp, -, 52]
     mov     edx, dword, ptr, [rbp, -, 48]
     mov     edi, dword, ptr, [rbp, -, 44]
     je      LBB252_25
     mov     r8, qword, ptr, [rbp, -, 88]
     mov     qword, ptr, [r8], rax
     mov     qword, ptr, [r8, +, 8], rbx
     mov     r15, qword, ptr, [r8, +, 16]
     jmp     LBB252_12
LBB252_1:
 let mut offs = self.s.precarry.len();
 mov     r15, qword, ptr, [r8, +, 16]
LBB252_13:
     test    r15, r15
     je      LBB252_14
     mov     esi, 1
     mov     rdi, r15
     call    ___rust_alloc_zeroed
     test    rax, rax
     je      LBB252_27
     mov     rcx, rax
     mov     qword, ptr, [rbp, -, 80], rax
     mov     qword, ptr, [rbp, -, 72], r15
     mov     qword, ptr, [rbp, -, 64], r15
 while offs > 0 {
 lea     rsi, [r15, -, 1]
 xor     edi, edi
 mov     rbx, qword, ptr, [rbp, -, 88]
LBB252_17:
     mov     rdx, qword, ptr, [rbx, +, 16]
     cmp     rdx, rsi
     jbe     LBB252_18
     cmp     r15, rsi
     jbe     LBB252_21
 mov     rax, qword, ptr, [rbx]
 add     di, word, ptr, [rax, +, 2*rsi]
 out[offs] = c as u8;
 movzx   edi, di
 mov     byte, ptr, [rcx, +, rsi], dil
 c >>= 8;
 shr     edi, 8
 while offs > 0 {
 dec     rsi
 cmp     rsi, -1
 while offs > 0 {
 jne     LBB252_17
 jmp     LBB252_23
LBB252_14:
     mov     qword, ptr, [rbp, -, 80], 1
     mov     qword, ptr, [rbp, -, 72], r15
     mov     qword, ptr, [rbp, -, 64], r15
LBB252_23:
 out
 mov     rcx, qword, ptr, [rbp, -, 64]
 mov     rax, qword, ptr, [rbp, -, 96]
 mov     qword, ptr, [rax, +, 16], rcx
 mov     rdx, qword, ptr, [rbp, -, 80]
 mov     rcx, qword, ptr, [rbp, -, 72]
 mov     qword, ptr, [rax, +, 8], rcx
 mov     qword, ptr, [rax], rdx
 }
 add     rsp, 56
 pop     rbx
 pop     r12
 pop     r13
 pop     r14
 pop     r15
 pop     rbp
 ret
LBB252_26:
     call    alloc::raw_vec::capacity_overflow
LBB252_18:
     lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.63]
     call    core::panicking::panic_bounds_check
     jmp     LBB252_19
LBB252_21:
     lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.64]
     mov     rdx, r15
     call    core::panicking::panic_bounds_check
LBB252_19:
 ud2
LBB252_25:
     mov     rdi, r13
     mov     rsi, r12
     call    alloc::alloc::handle_alloc_error
LBB252_27:
     mov     esi, 1
     mov     rdi, r15
     call    alloc::alloc::handle_alloc_error
LBB252_24:
     mov     rbx, rax
     lea     rdi, [rbp, -, 80]
 }
 call    core::ptr::real_drop_in_place
 mov     rdi, rbx
 call    __Unwind_Resume
 ud2

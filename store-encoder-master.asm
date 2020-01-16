 fn store(&mut self, fl: u16, fh: u16, nms: u16) {
 push    rbp
 push    r15
 push    r14
 push    r13
 push    r12
 push    rbx
 sub     rsp, 56
 mov     rbp, rdi
 let (l, r) = self.lr_compute(fl, fh, nms);
 movzx   r12d, word, ptr, [rdi, +, 36]
 mov     eax, r12d
 shr     eax, 8
 if fl < 32768 {
 test    si, si
 if fl < 32768 {
 js      .LBB1456_3
 u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 movzx   esi, si
 shr     esi, 6
 u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 imul    esi, eax
 u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 shr     esi
 + EC_MIN_PROB * nms as u32;
 movzx   edi, cx
 u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 lea     esi, [rsi, +, 4*rdi]
 v = (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 movzx   edx, dx
 shr     edx, 6
 v = (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 imul    edx, eax
 v = (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 shr     edx
 (r - u, (u - v) as u16)
 sub     r12d, esi
 lea     eax, [rdx, +, 4*rcx]
 (r - u, (u - v) as u16)
 sub     esi, eax
 add     esi, 4
 mov     r13d, r12d
 mov     r12d, esi
 let mut low = l + self.s.low;
 add     r13d, dword, ptr, [rbp, +, 24]
 let mut c = self.cnt;
 movzx   esi, word, ptr, [rbp, +, 38]
     test    r12w, r12w
     je      .LBB1456_4
.LBB1456_2:
     bsr     ax, r12w
     xor     eax, 15
     jmp     .LBB1456_5
.LBB1456_3:
 r -= (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 movzx   edx, dx
 shr     edx, 6
 r -= (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 imul    eax, edx
 r -= (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 shr     eax
 + EC_MIN_PROB * (nms - 1) as u32;
 lea     eax, [rax, +, 4*rcx]
 (0, r as u16)
 sub     r12d, eax
 add     r12d, 4
 xor     r13d, r13d
 let mut low = l + self.s.low;
 add     r13d, dword, ptr, [rbp, +, 24]
 let mut c = self.cnt;
 movzx   esi, word, ptr, [rbp, +, 38]
     test    r12w, r12w
     jne     .LBB1456_2
.LBB1456_4:
     mov     ax, 16
.LBB1456_5:
     mov     ecx, 16
     size_of::<Self>() * 8 - self.leading_zeros() as usize (src/util/math.rs:56)
     sub     ecx, eax
     movzx   eax, cx
     mov     r14d, 16
 let d = 16 - r.ilog();
 sub     r14, rax
 if s >= 0 {
 mov     eax, esi
 add     ax, r14w
 if s >= 0 {
 js      .LBB1456_40
 c += 16;
 lea     edx, [rsi, +, 16]
 let mut m = (1 << c) - 1;
 mov     ecx, edx
 and     ecx, 31
 mov     edi, -1
 shl     edi, cl
 not     edi
 if s >= 8 {
 cmp     ax, 7
 if s >= 8 {
 jle     .LBB1456_13
 self.s.precarry.push((low >> c) as u16);
 mov     edx, r13d
 shr     edx, cl
     mov     rax, qword, ptr, [rbp, +, 16]
     cmp     rax, qword, ptr, [rbp, +, 8]
     jne     .LBB1456_20
     mov     dword, ptr, [rsp, +, 16], edx
     mov     dword, ptr, [rsp, +, 20], edi
     mov     qword, ptr, [rsp, +, 48], rsi
     mov     r15, rax
     inc     r15
     je      .LBB1456_41
     lea     rcx, [rax, +, rax]
     cmp     rcx, r15
     cmova   r15, rcx
     xor     edx, edx
     mov     rcx, r15
     add     rcx, r15
     setae   cl
     mov     rbx, r15
     add     rbx, r15
     jb      .LBB1456_41
     mov     dl, cl
     add     rdx, rdx
     test    rax, rax
     mov     qword, ptr, [rsp, +, 40], rdx
     mov     qword, ptr, [rsp, +, 32], rbx
     je      .LBB1456_27
     mov     rdi, qword, ptr, [rbp]
     test    rbx, rbx
     je      .LBB1456_32
     mov     rsi, rbx
     call    qword, ptr, [rip, +, realloc@GOTPCREL]
     mov     rbx, rax
     test    rbx, rbx
     jne     .LBB1456_36
     jmp     .LBB1456_42
.LBB1456_13:
     mov     rax, qword, ptr, [rbp, +, 16]
 mov     esi, edx
 self.s.precarry.push((low >> c) as u16);
 mov     edx, r13d
 shr     edx, cl
     cmp     rax, qword, ptr, [rbp, +, 8]
     jne     .LBB1456_38
.LBB1456_15:
     mov     dword, ptr, [rsp, +, 16], edx
     mov     dword, ptr, [rsp, +, 20], edi
     mov     qword, ptr, [rsp, +, 48], rsi
     mov     r15, rax
     inc     r15
     je      .LBB1456_41
     lea     rcx, [rax, +, rax]
     cmp     rcx, r15
     cmova   r15, rcx
     xor     edx, edx
     mov     rcx, r15
     add     rcx, r15
     setae   cl
     mov     rbx, r15
     add     rbx, r15
     jb      .LBB1456_41
     mov     dl, cl
     add     rdx, rdx
     test    rax, rax
     mov     qword, ptr, [rsp, +, 40], rdx
     mov     qword, ptr, [rsp, +, 32], rbx
     je      .LBB1456_21
     mov     rdi, qword, ptr, [rbp]
     test    rbx, rbx
     je      .LBB1456_24
     mov     rsi, rbx
     call    qword, ptr, [rip, +, realloc@GOTPCREL]
     mov     rbx, rax
     test    rbx, rbx
     jne     .LBB1456_31
     jmp     .LBB1456_42
.LBB1456_20:
     mov     rbx, qword, ptr, [rbp]
     jmp     .LBB1456_37
.LBB1456_21:
     cmp     rdx, rbx
     jbe     .LBB1456_30
     mov     qword, ptr, [rsp, +, 8], 0
     lea     rdi, [rsp, +, 8]
     mov     esi, 8
     mov     rdx, rbx
     call    qword, ptr, [rip, +, posix_memalign@GOTPCREL]
     test    eax, eax
     jne     .LBB1456_42
     mov     rbx, qword, ptr, [rsp, +, 8]
     test    rbx, rbx
     jne     .LBB1456_31
     jmp     .LBB1456_42
.LBB1456_24:
     mov     qword, ptr, [rsp, +, 24], rdi
     mov     qword, ptr, [rsp, +, 8], 0
     lea     rdi, [rsp, +, 8]
     mov     esi, 8
     xor     edx, edx
     call    qword, ptr, [rip, +, posix_memalign@GOTPCREL]
     test    eax, eax
     jne     .LBB1456_42
     mov     rbx, qword, ptr, [rsp, +, 8]
     test    rbx, rbx
     je      .LBB1456_42
     mov     rdi, qword, ptr, [rsp, +, 24]
     call    qword, ptr, [rip, +, free@GOTPCREL]
     jmp     .LBB1456_31
.LBB1456_27:
     cmp     rdx, rbx
     jbe     .LBB1456_35
     mov     qword, ptr, [rsp, +, 8], 0
     lea     rdi, [rsp, +, 8]
     mov     esi, 8
     mov     rdx, rbx
     call    qword, ptr, [rip, +, posix_memalign@GOTPCREL]
     test    eax, eax
     jne     .LBB1456_42
     mov     rbx, qword, ptr, [rsp, +, 8]
     test    rbx, rbx
     jne     .LBB1456_36
     jmp     .LBB1456_42
.LBB1456_30:
     mov     rdi, rbx
     call    qword, ptr, [rip, +, malloc@GOTPCREL]
     mov     rbx, rax
     test    rbx, rbx
     je      .LBB1456_42
.LBB1456_31:
     mov     qword, ptr, [rbp], rbx
     mov     qword, ptr, [rbp, +, 8], r15
     mov     rax, qword, ptr, [rbp, +, 16]
     mov     rsi, qword, ptr, [rsp, +, 48]
     mov     edi, dword, ptr, [rsp, +, 20]
     mov     edx, dword, ptr, [rsp, +, 16]
     jmp     .LBB1456_39
.LBB1456_32:
     mov     qword, ptr, [rsp, +, 24], rdi
     mov     qword, ptr, [rsp, +, 8], 0
     lea     rdi, [rsp, +, 8]
     mov     esi, 8
     xor     edx, edx
     call    qword, ptr, [rip, +, posix_memalign@GOTPCREL]
     test    eax, eax
     jne     .LBB1456_42
     mov     rbx, qword, ptr, [rsp, +, 8]
     test    rbx, rbx
     je      .LBB1456_42
     mov     rdi, qword, ptr, [rsp, +, 24]
     call    qword, ptr, [rip, +, free@GOTPCREL]
     jmp     .LBB1456_36
.LBB1456_35:
     mov     rdi, rbx
     call    qword, ptr, [rip, +, malloc@GOTPCREL]
     mov     rbx, rax
     test    rbx, rbx
     je      .LBB1456_42
.LBB1456_36:
     mov     qword, ptr, [rbp], rbx
     mov     qword, ptr, [rbp, +, 8], r15
     mov     rax, qword, ptr, [rbp, +, 16]
     mov     rsi, qword, ptr, [rsp, +, 48]
     mov     edi, dword, ptr, [rsp, +, 20]
     mov     edx, dword, ptr, [rsp, +, 16]
.LBB1456_37:
     mov     word, ptr, [rbx, +, 2*rax], dx
     mov     rax, qword, ptr, [rbp, +, 16]
     add     rax, 1
     mov     qword, ptr, [rbp, +, 16], rax
 low &= m;
 and     r13d, edi
 c -= 8;
 add     esi, 8
 m >>= 8;
 shr     edi, 8
 self.s.precarry.push((low >> c) as u16);
 mov     ecx, esi
 and     ecx, 31
 mov     edx, r13d
 shr     edx, cl
     cmp     rax, qword, ptr, [rbp, +, 8]
     je      .LBB1456_15
.LBB1456_38:
     mov     rbx, qword, ptr, [rbp]
.LBB1456_39:
     mov     word, ptr, [rbx, +, 2*rax], dx
     add     qword, ptr, [rbp, +, 16], 1
 s = c + (d as i16) - 24;
 lea     eax, [r14, +, rsi]
 add     eax, -24
 low &= m;
 and     r13d, edi
.LBB1456_40:
 self.s.low = low << d;
 mov     ecx, r14d
 shl     r13d, cl
 self.rng = r << d;
 and     r14b, 15
 mov     ecx, r14d
 shl     r12d, cl
 self.s.low = low << d;
 mov     dword, ptr, [rbp, +, 24], r13d
 self.rng = r << d;
 mov     word, ptr, [rbp, +, 36], r12w
 self.cnt = s;
 mov     word, ptr, [rbp, +, 38], ax
 }
 add     rsp, 56
 pop     rbx
 pop     r12
 pop     r13
 pop     r14
 pop     r15
 pop     rbp
 ret
.LBB1456_41:
 call    alloc::raw_vec::capacity_overflow
 ud2
.LBB1456_42:
 mov     rdi, qword, ptr, [rsp, +, 32]
 mov     rsi, qword, ptr, [rsp, +, 40]
 call    alloc::alloc::handle_alloc_error
 ud2

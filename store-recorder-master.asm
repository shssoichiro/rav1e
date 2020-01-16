 fn store(&mut self, fl: u16, fh: u16, nms: u16) {
 push    rbp
 push    r15
 push    r14
 push    r13
 push    r12
 push    rbx
 sub     rsp, 40
 mov     r8d, ecx
 mov     rbx, rdi
 let (_l, r) = self.lr_compute(fl, fh, nms);
 movzx   eax, word, ptr, [rdi, +, 36]
 mov     ecx, eax
 shr     ecx, 8
 self.s.storage.push((fl, fh, nms));
 movzx   r13d, dx
 u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 movzx   r14d, si
 if fl < 32768 {
 test    si, si
 if fl < 32768 {
 js      .LBB1455_1
 u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 mov     eax, r14d
 shr     eax, 6
 u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 imul    eax, ecx
 u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 shr     eax
 v = (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 mov     edx, r13d
 shr     edx, 6
 v = (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 imul    edx, ecx
 v = (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 shr     edx
 u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 neg     edx
 (r - u, (u - v) as u16)
 lea     eax, [rdx, +, rax]
 add     eax, -65532
     test    ax, ax
     je      .LBB1455_4
.LBB1455_5:
     bsr     cx, ax
     xor     ecx, 15
 let mut c = self.cnt;
 movzx   edx, word, ptr, [rbx, +, 38]
 if s >= 0 {
 mov     esi, edx
 add     si, cx
 if s >= 0 {
 js      .LBB1455_11
.LBB1455_7:
 if s >= 8 {
 cmp     si, 7
 if s >= 8 {
 jle     .LBB1455_8
 self.s.bytes += 1;
 mov     rsi, qword, ptr, [rbx, +, 24]
 add     rsi, 1
 mov     qword, ptr, [rbx, +, 24], rsi
 c -= 8;
 add     edx, 8
 jmp     .LBB1455_10
.LBB1455_1:
 r -= (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 mov     edx, r13d
 shr     edx, 6
 r -= (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 imul    ecx, edx
 r -= (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 shr     ecx
 + EC_MIN_PROB * (nms - 1) as u32;
 lea     ecx, [rcx, +, 4*r8]
 (0, r as u16)
 sub     eax, ecx
 add     eax, 4
     test    ax, ax
     jne     .LBB1455_5
.LBB1455_4:
     mov     cx, 16
 let mut c = self.cnt;
 movzx   edx, word, ptr, [rbx, +, 38]
 if s >= 0 {
 mov     esi, edx
 add     si, cx
 if s >= 0 {
 jns     .LBB1455_7
 jmp     .LBB1455_11
.LBB1455_8:
 c += 16;
 add     edx, 16
 self.s.bytes += 1;
 mov     rsi, qword, ptr, [rbx, +, 24]
.LBB1455_10:
 add     rsi, 1
 mov     qword, ptr, [rbx, +, 24], rsi
 s = c + (d as i16) - 24;
 lea     esi, [rcx, +, rdx]
 add     esi, -24
.LBB1455_11:
 self.rng = r << d;
 and     cl, 15
 shl     eax, cl
 mov     word, ptr, [rbx, +, 36], ax
 self.cnt = s;
 mov     word, ptr, [rbx, +, 38], si
     mov     rcx, qword, ptr, [rbx, +, 16]
     cmp     rcx, qword, ptr, [rbx, +, 8]
     jne     .LBB1455_12
     mov     qword, ptr, [rsp, +, 32], r8
     mov     r15, rcx
     inc     r15
     je      .LBB1455_30
     lea     rax, [rcx, +, rcx]
     cmp     rax, r15
     cmova   r15, rax
     mov     edx, 6
     xor     r12d, r12d
     mov     rax, r15
     mul     rdx
     setno   dl
     jo      .LBB1455_30
     mov     r12b, dl
     add     r12, r12
     test    rcx, rcx
     mov     qword, ptr, [rsp, +, 24], rax
     je      .LBB1455_16
     mov     rdi, qword, ptr, [rbx]
     test    rax, rax
     je      .LBB1455_21
     mov     rsi, rax
     call    qword, ptr, [rip, +, realloc@GOTPCREL]
     mov     rbp, rax
     test    rbp, rbp
     jne     .LBB1455_27
     jmp     .LBB1455_29
.LBB1455_12:
     mov     rbp, qword, ptr, [rbx]
     jmp     .LBB1455_28
.LBB1455_16:
     cmp     r12, rax
     jbe     .LBB1455_19
     mov     qword, ptr, [rsp, +, 8], 0
     lea     rdi, [rsp, +, 8]
     mov     esi, 8
     mov     rdx, rax
     call    qword, ptr, [rip, +, posix_memalign@GOTPCREL]
     test    eax, eax
     jne     .LBB1455_29
     mov     rbp, qword, ptr, [rsp, +, 8]
     test    rbp, rbp
     jne     .LBB1455_27
     jmp     .LBB1455_29
.LBB1455_21:
     mov     qword, ptr, [rsp, +, 16], rdi
     mov     qword, ptr, [rsp, +, 8], 0
     lea     rdi, [rsp, +, 8]
     mov     esi, 8
     xor     edx, edx
     call    qword, ptr, [rip, +, posix_memalign@GOTPCREL]
     test    eax, eax
     jne     .LBB1455_29
     mov     rbp, qword, ptr, [rsp, +, 8]
     test    rbp, rbp
     je      .LBB1455_29
     mov     rdi, qword, ptr, [rsp, +, 16]
     call    qword, ptr, [rip, +, free@GOTPCREL]
     jmp     .LBB1455_27
.LBB1455_19:
     mov     rdi, rax
     call    qword, ptr, [rip, +, malloc@GOTPCREL]
     mov     rbp, rax
     test    rbp, rbp
     je      .LBB1455_29
.LBB1455_27:
     mov     qword, ptr, [rbx], rbp
     mov     qword, ptr, [rbx, +, 8], r15
     mov     rcx, qword, ptr, [rbx, +, 16]
     mov     r8, qword, ptr, [rsp, +, 32]
.LBB1455_28:
 movzx   eax, r8w
 shl     rax, 32
 shl     r13, 16
 or      r14, r13
 or      r14, rax
     lea     rax, [rcx, +, 2*rcx]
     mov     dword, ptr, [rbp, +, 2*rax], r14d
     shr     r14, 32
     mov     word, ptr, [rbp, +, 2*rax, +, 4], r14w
     add     qword, ptr, [rbx, +, 16], 1
 }
 add     rsp, 40
 pop     rbx
 pop     r12
 pop     r13
 pop     r14
 pop     r15
 pop     rbp
 ret
.LBB1455_30:
     call    alloc::raw_vec::capacity_overflow
     ud2
.LBB1455_29:
     mov     rdi, qword, ptr, [rsp, +, 24]
     mov     rsi, r12
     call    alloc::alloc::handle_alloc_error
     ud2

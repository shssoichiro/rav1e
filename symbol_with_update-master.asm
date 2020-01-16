 fn symbol_with_update(&mut self, s: u32, cdf: &mut [u16]) {
 push    rbx
 let nsymbs = cdf.len() - 1;
 mov     rbx, rcx
 sub     rbx, 1
     jb      .LBB565_20
     mov     r8, rcx
     mov     r9, rdx
     mov     edx, esi
 let nms = cdf.len() - s as usize;
 mov     esi, esi
 let fl = if s > 0 { cdf[s as usize - 1] } else { 32768 };
 test    edx, edx
 let fl = if s > 0 { cdf[s as usize - 1] } else { 32768 };
 je      .LBB565_2
 let fl = if s > 0 { cdf[s as usize - 1] } else { 32768 };
 lea     rax, [rsi, -, 1]
 let fl = if s > 0 { cdf[s as usize - 1] } else { 32768 };
 cmp     rax, rbx
 jae     .LBB565_21
 movzx   r11d, word, ptr, [r9, +, 2*rsi, -, 2]
 let fh = cdf[s as usize];
 cmp     rbx, rsi
 jbe     .LBB565_6
.LBB565_8:
 sub     rbx, rsi
 movzx   esi, word, ptr, [r9, +, 2*rsi]
     let (_l, r) = self.lr_compute(fl, fh, nms); (src/ec.rs:189)
     movzx   r10d, word, ptr, [rdi, +, 12]
     mov     eax, r10d
     shr     eax, 8
     if fl < 32768 { (src/ec.rs:343)
     test    r11w, r11w
     if fl < 32768 { (src/ec.rs:343)
     js      .LBB565_9
     u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT)) (src/ec.rs:344)
     movzx   ecx, r11w
     shr     ecx, 6
     u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT)) (src/ec.rs:344)
     imul    ecx, eax
     u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT)) (src/ec.rs:344)
     shr     ecx
     v = (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT)) (src/ec.rs:346)
     shr     esi, 6
     v = (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT)) (src/ec.rs:346)
     imul    esi, eax
     v = (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT)) (src/ec.rs:346)
     shr     esi
     sub     ebx, ebx
     shl     ebx, 2
     u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT)) (src/ec.rs:344)
     sub     ebx, esi
     (r - u, (u - v) as u16) (src/ec.rs:348)
     lea     r10d, [rbx, +, rcx]
     add     r10d, -65532
     test    r10w, r10w
     je      .LBB565_12
.LBB565_13:
     bsr     cx, r10w
     xor     ecx, 15
     let mut c = self.cnt; (src/ec.rs:191)
     movzx   esi, word, ptr, [rdi, +, 14]
     if s >= 0 { (src/ec.rs:194)
     mov     eax, esi
     add     ax, cx
     if s >= 0 { (src/ec.rs:194)
     js      .LBB565_19
.LBB565_15:
     if s >= 8 { (src/ec.rs:196)
     cmp     ax, 7
     if s >= 8 { (src/ec.rs:196)
     jle     .LBB565_16
     self.s.bytes += 1; (src/ec.rs:197)
     mov     rax, qword, ptr, [rdi]
     add     rax, 1
     mov     qword, ptr, [rdi], rax
     c -= 8; (src/ec.rs:198)
     add     esi, 8
     jmp     .LBB565_18
.LBB565_2:
     mov     r11w, -32768
 let fh = cdf[s as usize];
 cmp     rbx, rsi
 ja      .LBB565_8
.LBB565_6:
 lea     rdi, [rip, +, .L__unnamed_458]
 mov     rdx, rbx
 call    core::panicking::panic_bounds_check
 ud2
.LBB565_9:
     r -= (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT)) (src/ec.rs:350)
     shr     esi, 6
     r -= (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT)) (src/ec.rs:350)
     imul    eax, esi
     r -= (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT)) (src/ec.rs:350)
     shr     eax
     + EC_MIN_PROB * (nms - 1) as u32; (src/ec.rs:351)
     lea     eax, [rax, +, 4*rbx]
     (0, r as u16) (src/ec.rs:352)
     sub     r10d, eax
     add     r10d, 4
     test    r10w, r10w
     jne     .LBB565_13
.LBB565_12:
     mov     cx, 16
     let mut c = self.cnt; (src/ec.rs:191)
     movzx   esi, word, ptr, [rdi, +, 14]
     if s >= 0 { (src/ec.rs:194)
     mov     eax, esi
     add     ax, cx
     if s >= 0 { (src/ec.rs:194)
     jns     .LBB565_15
     jmp     .LBB565_19
.LBB565_16:
     c += 16; (src/ec.rs:195)
     add     esi, 16
     self.s.bytes += 1; (src/ec.rs:200)
     mov     rax, qword, ptr, [rdi]
.LBB565_18:
     add     rax, 1
     mov     qword, ptr, [rdi], rax
     s = c + (d as i16) - 24; (src/ec.rs:201)
     lea     eax, [rcx, +, rsi]
     add     eax, -24
.LBB565_19:
     self.rng = r << d; (src/ec.rs:203)
     and     cl, 15
     shl     r10d, cl
     mov     word, ptr, [rdi, +, 12], r10w
     self.cnt = s; (src/ec.rs:204)
     mov     word, ptr, [rdi, +, 14], ax
 update_cdf(cdf, s);
 mov     rdi, r9
 mov     rsi, r8
 pop     rbx
 jmp     _ZN5rav1e3asm3x862ec10update_cdf17h416e1c7249953495E
.LBB565_20:
     mov     rdi, rbx
     xor     esi, esi
     call    core::slice::slice_index_len_fail
     ud2
.LBB565_21:
 let fl = if s > 0 { cdf[s as usize - 1] } else { 32768 };
 lea     rdi, [rip, +, .L__unnamed_463]
 mov     rsi, rax
 mov     rdx, rbx
 call    core::panicking::panic_bounds_check
 ud2

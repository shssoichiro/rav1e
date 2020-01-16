 fn store(&mut self, fl: u16, fh: u16, nms: u16) {
 let (_l, r) = self.lr_compute(fl, fh, nms);
 movzx   r8d, word, ptr, [rdi, +, 12]
 mov     eax, r8d
 shr     eax, 8
 if fl < 32768 {
 test    si, si
 if fl < 32768 {
 js      .LBB1454_1
 u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 movzx   ecx, si
 shr     ecx, 6
 u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 imul    ecx, eax
 u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 shr     ecx
 v = (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 movzx   edx, dx
 shr     edx, 6
 v = (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 imul    edx, eax
 v = (((r >> 8) * (fh as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 shr     edx
 u = (((r >> 8) * (fl as u32 >> EC_PROB_SHIFT)) >> (7 - EC_PROB_SHIFT))
 neg     edx
 (r - u, (u - v) as u16)
 lea     r8d, [rdx, +, rcx]
 add     r8d, -65532
     test    r8w, r8w
     je      .LBB1454_4
.LBB1454_5:
     bsr     cx, r8w
     xor     ecx, 15
 let mut c = self.cnt;
 movzx   edx, word, ptr, [rdi, +, 14]
 if s >= 0 {
 mov     eax, edx
 add     ax, cx
 if s >= 0 {
 js      .LBB1454_11
.LBB1454_7:
 if s >= 8 {
 cmp     ax, 7
 if s >= 8 {
 jle     .LBB1454_8
 self.s.bytes += 1;
 mov     rax, qword, ptr, [rdi]
 add     rax, 1
 mov     qword, ptr, [rdi], rax
 c -= 8;
 add     edx, 8
 jmp     .LBB1454_10
.LBB1454_1:
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
 sub     r8d, eax
 add     r8d, 4
     test    r8w, r8w
     jne     .LBB1454_5
.LBB1454_4:
     mov     cx, 16
 let mut c = self.cnt;
 movzx   edx, word, ptr, [rdi, +, 14]
 if s >= 0 {
 mov     eax, edx
 add     ax, cx
 if s >= 0 {
 jns     .LBB1454_7
 jmp     .LBB1454_11
.LBB1454_8:
 c += 16;
 add     edx, 16
 self.s.bytes += 1;
 mov     rax, qword, ptr, [rdi]
.LBB1454_10:
 add     rax, 1
 mov     qword, ptr, [rdi], rax
 s = c + (d as i16) - 24;
 lea     eax, [rcx, +, rdx]
 add     eax, -24
.LBB1454_11:
 self.rng = r << d;
 and     cl, 15
 shl     r8d, cl
 mov     word, ptr, [rdi, +, 12], r8w
 self.cnt = s;
 mov     word, ptr, [rdi, +, 14], ax
 }
 ret

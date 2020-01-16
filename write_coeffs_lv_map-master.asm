 pub fn write_coeffs_lv_map<T: Coefficient>(
 push    rbp
 mov     rbp, rsp
 push    r15
 push    r14
 push    r13
 push    r12
 push    rbx
 and     rsp, -32
 mov     eax, 11136
 call    __rust_probestack
 sub     rsp, rax
 mov     r15, r9
 mov     r12, r8
 mov     qword, ptr, [rsp, +, 72], rcx
 mov     qword, ptr, [rsp, +, 24], rdx
 mov     qword, ptr, [rsp, +, 8], rsi
 mov     r13d, dword, ptr, [rbp, +, 48]
     TX_64X64 | TX_64X32 | TX_32X64 => TX_32X32, (src/context.rs:1955)
     lea     eax, [r13, -, 4]
     mov     ebx, r13d
     cmp     eax, 14
     ja      .LBB447_3
     mov     ecx, 32
     mov     edx, 16
     lea     rbx, [rip, +, .LJTI447_0]
     movsxd  rsi, dword, ptr, [rbx, +, 4*rax]
     add     rsi, rbx
     mov     ebx, r13d
     jmp     rsi
.LBB447_2:
     mov     ebx, 3
.LBB447_3:
     TX_4X4 | TX_4X8 | TX_4X16 => 2, (src/transform/mod.rs:116)
     mov     ecx, ebx
     lea     rdx, [rip, +, .Lswitch.table._ZN5rav1e11recon_intra13has_top_right17h21096a7ec98f3035E]
     mov     cl, byte, ptr, [rdx, +, 8*rcx]
     mov     edx, 1
     1 << self.width_log2() (src/transform/mod.rs:111)
     shl     rdx, cl
     TX_64X64 | TX_64X32 | TX_32X64 => TX_32X32, (src/context.rs:1955)
     cmp     eax, 14
     ja      .LBB447_103
     lea     rcx, [rip, +, .LJTI447_1]
     movsxd  rsi, dword, ptr, [rcx, +, 4*rax]
     add     rsi, rcx
     mov     rcx, rdx
     mov     eax, r13d
     jmp     rsi
.LBB447_5:
     mov     eax, 3
     jmp     .LBB447_8
.LBB447_6:
     mov     eax, 9
     jmp     .LBB447_8
.LBB447_7:
     mov     eax, 10
     mov     rdx, rcx
.LBB447_8:
     mov     qword, ptr, [rsp, +, 16], rdi
     TX_4X4 | TX_8X4 | TX_16X4 => 2, (src/transform/mod.rs:134)
     mov     eax, eax
     lea     rcx, [rip, +, .Lswitch.table._ZN5rav1e11recon_intra15has_bottom_left17ha8c8ba4e0e3f0f8eE.398]
     mov     rcx, qword, ptr, [rcx, +, 8*rax]
     mov     esi, 1
     1 << self.height_log2() (src/transform/mod.rs:129)
     shl     rsi, cl
     mov     qword, ptr, [rsp, +, 80], rcx
 &mut coeffs_storage.array[..width * height],
 shl     rdx, cl
     cmp     rdx, 1025
     jae     .LBB447_128
     mov     ebx, dword, ptr, [rbp, +, 56]
     mov     rax, r13
     shl     rax, 8
     lea     r14, [rip, +, _ZN5rav1e10scan_order15av1_scan_orders17hba7ad037b1fa1f23E]
     add     r14, rax
     shl     rbx, 4
     mov     rax, qword, ptr, [rbp, +, 32]
     test    rdx, rdx
     mov     qword, ptr, [rsp, +, 32], rdx
     mov     qword, ptr, [rsp], rsi
     je      .LBB447_11
     add     rdx, rdx
     lea     rdi, [rsp, +, 160]
     *a = MaybeUninit::new(value); (src/util/uninit.rs:17)
     xor     esi, esi
     call    qword, ptr, [rip, +, memset@GOTPCREL]
     mov     rax, qword, ptr, [rbp, +, 32]
     mov     rdx, qword, ptr, [rsp, +, 32]
.LBB447_11:
     mov     r9, qword, ptr, [rbp, +, 24]
     mov     rcx, qword, ptr, [rbx, +, r14]
     mov     qword, ptr, [rsp, +, 104], rcx
     mov     rcx, qword, ptr, [rbx, +, r14, +, 8]
     TX_4X4 => TX_4X4, (src/transform/mod.rs:198)
     mov     r8d, r13d
     test    rax, rax
     mov     qword, ptr, [rsp, +, 128], rcx
     je      .LBB447_21
     add     rcx, rcx
     xor     edi, edi
.LBB447_13:
     test    rcx, rcx
     je      .LBB447_17
     mov     rsi, qword, ptr, [rsp, +, 104]
 for (i, &scan_idx) in scan.iter().take(eob).enumerate() {
 movzx   esi, word, ptr, [rsi, +, 2*rdi]
 coeffs[i] = coeffs_in[scan_idx as usize];
 cmp     rsi, r9
 jae     .LBB447_126
 coeffs[i] = coeffs_in[scan_idx as usize];
 cmp     rdx, rdi
 je      .LBB447_127
 coeffs[i] = coeffs_in[scan_idx as usize];
 mov     rbx, qword, ptr, [rbp, +, 16]
 movzx   esi, word, ptr, [rbx, +, 2*rsi]
 coeffs[i] = coeffs_in[scan_idx as usize];
 mov     word, ptr, [rsp, +, 2*rdi, +, 160], si
 add     rdi, 1
     add     rcx, -2
     cmp     rax, rdi
     jne     .LBB447_13
.LBB447_17:
     lea     rcx, [rdx, +, rdx]
     xor     edx, edx
     mov     rsi, rax
     xor     eax, eax
.LBB447_18:
     cmp     rcx, rdx
     je      .LBB447_20
     movzx   ebx, word, ptr, [rsp, +, rdx, +, 160]
     if self.is_negative() { -*self } else { *self } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/sign.rs:49)
     mov     edi, ebx
     neg     di
     cmovl   di, bx
     #[inline] fn as_(self) -> $U { self as $U } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/cast.rs:746)
     movsx   edi, di
     add     eax, edi
     add     rdx, 2
     add     rsi, -1
     jne     .LBB447_18
.LBB447_20:
     mov     dword, ptr, [rsp, +, 68], eax
 coeffs.iter().take(eob).map(|c| u32::cast_from(c.abs())).sum();
 mov     dword, ptr, [rsp, +, 44], eax
 mov     esi, 4
 xor     ecx, ecx
 lea     rdx, [rip, +, .LJTI447_2]
 movsxd  rdi, dword, ptr, [rdx, +, 4*r8]
 add     rdi, rdx
 jmp     rdi
.LBB447_21:
 xor     eax, eax
 mov     dword, ptr, [rsp, +, 68], 0
 mov     dword, ptr, [rsp, +, 44], eax
 mov     esi, 4
 lea     rcx, [rip, +, .LJTI447_2]
 movsxd  rdx, dword, ptr, [rcx, +, 4*r8]
 add     rdx, rcx
 xor     ecx, ecx
 jmp     rdx
.LBB447_22:
 mov     ecx, 1
 xor     edx, edx
 lea     rsi, [rip, +, .LJTI447_3]
 movsxd  rdi, dword, ptr, [rsi, +, 4*r8]
 add     rdi, rsi
 mov     esi, 1
 mov     eax, 1
 jmp     rdi
.LBB447_23:
 mov     ecx, 2
 xor     edx, edx
 lea     rsi, [rip, +, .LJTI447_3]
 movsxd  rdi, dword, ptr, [rsi, +, 4*r8]
 add     rdi, rsi
 mov     esi, 2
 mov     eax, 2
 jmp     rdi
.LBB447_24:
 mov     ecx, 3
.LBB447_25:
 xor     edx, edx
 lea     rsi, [rip, +, .LJTI447_3]
 movsxd  rdi, dword, ptr, [rsi, +, 4*r8]
 add     rdi, rsi
 mov     esi, ecx
 mov     eax, ecx
 jmp     rdi
.LBB447_26:
 mov     edx, 2
 jmp     .LBB447_30
.LBB447_27:
 mov     edx, 4
 mov     eax, esi
 jmp     .LBB447_31
.LBB447_28:
 mov     edx, 3
 jmp     .LBB447_30
.LBB447_29:
 mov     edx, 1
.LBB447_30:
 mov     eax, ecx
.LBB447_31:
 mov     cl, byte, ptr, [rbp, +, 64]
     (tx_size.sqr() as usize + tx_size.sqr_up() as usize + 1) >> 1 (src/context.rs:3842)
     lea     ebx, [rax, +, rdx]
     add     ebx, 1
     mov     eax, ebx
     shr     eax
     mov     qword, ptr, [rsp, +, 48], rax
 self.bc.get_txb_ctx(plane_bsize, tx_size, plane, bo, xdec, ydec);
 movzx   esi, cl
 mov     rdi, qword, ptr, [rsp, +, 16]
 mov     edx, r13d
 mov     rcx, qword, ptr, [rsp, +, 72]
 mov     r8, r12
 mov     r9, r15
 push    qword, ptr, [rbp, +, 80]
 push    qword, ptr, [rbp, +, 72]
 call    rav1e::context::BlockContext::get_txb_ctx
 add     rsp, 16
 mov     qword, ptr, [rsp, +, 112], rdx
 let cdf = &mut self.fc.txb_skip_cdf[txs_ctx][txb_ctx.txb_skip_ctx];
 cmp     ebx, 10
 jae     .LBB447_129
 cmp     rax, 12
 mov     rcx, qword, ptr, [rbp, +, 32]
 ja      .LBB447_130
     xor     esi, esi
     test    rcx, rcx
     sete    sil
 let cdf = &mut self.fc.txb_skip_cdf[txs_ctx][txb_ctx.txb_skip_ctx];
 imul    rcx, qword, ptr, [rsp, +, 48], 78
 mov     rbx, qword, ptr, [rsp, +, 16]
 add     rcx, qword, ptr, [rbx, +, 4696]
 lea     rax, [rax, +, 2*rax]
 lea     rdx, [rcx, +, 2*rax]
 add     rdx, 9546
 mov     rax, qword, ptr, [rsp, +, 24]
 $w.symbol_with_update($s, $cdf);
 mov     rax, qword, ptr, [rax, +, 40]
 mov     ecx, 3
 mov     rdi, qword, ptr, [rsp, +, 8]
 mov     qword, ptr, [rsp, +, 56], rax
 call    rax
     cmp     qword, ptr, [rbp, +, 32], 0
 if eob == 0 {
 je      .LBB447_37
 lea     rdi, [rsp, +, 2232]
 mov     r14d, 4776
 let mut levels_buf = [0u8; TX_PAD_2D];
 mov     edx, 4776
 xor     esi, esi
 call    qword, ptr, [rip, +, memset@GOTPCREL]
 mov     r8, qword, ptr, [rsp]
 &mut levels_buf[TX_PAD_TOP * (height + TX_PAD_HOR)..];
 lea     rdi, [r8, +, r8]
 add     rdi, 8
     cmp     rdi, 4777
     jae     .LBB447_131
     sub     r14, rdi
     mov     r10, qword, ptr, [rbp, +, 24]
     test    r10, r10
     mov     rbx, qword, ptr, [rsp, +, 80]
     je      .LBB447_38
     mov     rdx, r10
     mov     ecx, ebx
     shr     rdx, cl
     lea     rax, [r8, -, 1]
     and     rax, r10
     cmp     rax, 1
     sbb     rdx, -1
     mov     rsi, rdx
     jmp     .LBB447_39
.LBB447_37:
 self.bc.set_coeff_context(plane, bo, tx_size, xdec, ydec, 0);
 mov     rdi, rbx
 mov     rsi, qword, ptr, [rsp, +, 72]
 mov     rdx, r12
 mov     rcx, r15
 mov     r8d, r13d
 mov     r9, qword, ptr, [rbp, +, 72]
 push    0
 push    qword, ptr, [rbp, +, 80]
 call    rav1e::context::BlockContext::set_coeff_context
 add     rsp, 16
 xor     eax, eax
 jmp     .LBB447_118
.LBB447_38:
 xor     esi, esi
.LBB447_39:
 lea     r11, [r8, +, 4]
     test    r14, r14
     mov     qword, ptr, [rsp, +, 144], r15
     mov     qword, ptr, [rsp, +, 136], r12
     je      .LBB447_41
     mov     rax, r14
     xor     edx, edx
     div     r11
     cmp     rdx, 1
     sbb     rax, -1
     jmp     .LBB447_42
.LBB447_41:
     xor     eax, eax
.LBB447_42:
     lea     rcx, [rsp, +, rdi]
     add     rcx, 2232
     mov     qword, ptr, [rsp, +, 88], rcx
     cmp     rsi, rax
     cmova   rsi, rax
     mov     qword, ptr, [rsp, +, 96], rsi
     test    rsi, rsi
     je      .LBB447_60
     lea     rax, [rsp, +, rdi]
     add     rax, 2240
     mov     rcx, qword, ptr, [rbp, +, 16]
     add     rcx, 16
     mov     qword, ptr, [rsp, +, 152], rcx
     xor     r15d, r15d
     pxor    xmm0, xmm0
     movdqa  xmm1, xmmword, ptr, [rip, +, .LCPI447_0]
     mov     rdx, qword, ptr, [rsp, +, 88]
     jmp     .LBB447_45
.LBB447_44:
     add     rax, r11
     add     rdx, r11
     cmp     r15, qword, ptr, [rsp, +, 96]
     mov     rbx, r9
     jae     .LBB447_60
.LBB447_45:
     mov     r13, r15
     imul    r13, r11
     lea     r12, [r11, +, r13]
     cmp     r12, r14
     cmova   r12, r14
     mov     rcx, r13
     add     rcx, r11
     cmovb   r12, r14
     mov     rdi, r15
     mov     r9, rbx
     mov     ecx, ebx
     shl     rdi, cl
     mov     rbx, qword, ptr, [rsp]
     lea     rsi, [rdi, +, rbx]
     cmp     rsi, r10
     cmova   rsi, r10
     mov     rcx, rdi
     add     rcx, rbx
     cmovb   rsi, r10
     add     r15, 1
     mov     r8, qword, ptr, [rbp, +, 16]
     lea     rcx, [r8, +, 2*rdi]
     mov     rbx, qword, ptr, [rsp, +, 88]
     add     r13, rbx
     lea     rsi, [r8, +, 2*rsi]
     add     r12, rbx
     sub     rsi, rcx
     shr     rsi
     sub     r12, r13
     cmp     rsi, r12
     cmovbe  r12, rsi
     test    r12, r12
     je      .LBB447_44
     cmp     r12, 8
     jae     .LBB447_48
     xor     ebx, ebx
     jmp     .LBB447_55
.LBB447_48:
     mov     r10, r14
     mov     rbx, r12
     movabs  rsi, 9223372036854775800
     and     rbx, rsi
     lea     rsi, [rbx, -, 8]
     mov     r8, rsi
     shr     r8, 3
     add     r8, 1
     mov     r9d, r8d
     and     r9d, 1
     test    rsi, rsi
     je      .LBB447_59
     sub     r8, r9
     mov     rsi, qword, ptr, [rsp, +, 152]
     lea     r14, [rsi, +, 2*rdi]
     xor     edi, edi
.LBB447_50:
 *level = clamp(coeff.abs(), T::cast_from(0), T::cast_from(127)).as_();
 movdqu  xmm2, xmmword, ptr, [r14, +, 2*rdi, -, 16]
     if self.is_negative() { -*self } else { *self } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/sign.rs:49)
     movdqa  xmm3, xmm2
     psraw   xmm3, 15
     paddw   xmm2, xmm3
     pxor    xmm2, xmm3
     if input < min { (src/util/math.rs:40)
     pminsw  xmm2, xmm1
     pmaxsw  xmm2, xmm0
 *level = clamp(coeff.abs(), T::cast_from(0), T::cast_from(127)).as_();
 packuswb xmm2, xmm0
 movq    qword, ptr, [rax, +, rdi, -, 8], xmm2
 *level = clamp(coeff.abs(), T::cast_from(0), T::cast_from(127)).as_();
 movdqu  xmm2, xmmword, ptr, [r14, +, 2*rdi]
     if self.is_negative() { -*self } else { *self } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/sign.rs:49)
     movdqa  xmm3, xmm2
     psraw   xmm3, 15
     paddw   xmm2, xmm3
     pxor    xmm2, xmm3
     if input < min { (src/util/math.rs:40)
     pminsw  xmm2, xmm1
     pmaxsw  xmm2, xmm0
 *level = clamp(coeff.abs(), T::cast_from(0), T::cast_from(127)).as_();
 packuswb xmm2, xmm0
 movq    qword, ptr, [rax, +, rdi], xmm2
     add     rdi, 16
     add     r8, -2
     jne     .LBB447_50
     test    r9, r9
     je      .LBB447_53
.LBB447_52:
 *level = clamp(coeff.abs(), T::cast_from(0), T::cast_from(127)).as_();
 movdqu  xmm2, xmmword, ptr, [rcx, +, 2*rdi]
     if self.is_negative() { -*self } else { *self } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/sign.rs:49)
     movdqa  xmm3, xmm2
     psraw   xmm3, 15
     paddw   xmm2, xmm3
     pxor    xmm2, xmm3
     if input < min { (src/util/math.rs:40)
     pminsw  xmm2, xmm1
     pmaxsw  xmm2, xmm0
 *level = clamp(coeff.abs(), T::cast_from(0), T::cast_from(127)).as_();
 packuswb xmm2, xmm0
 movq    qword, ptr, [r13, +, rdi], xmm2
.LBB447_53:
     cmp     r12, rbx
     mov     r14, r10
     mov     r9, qword, ptr, [rsp, +, 80]
     mov     r10, qword, ptr, [rbp, +, 24]
     jne     .LBB447_55
     jmp     .LBB447_44
.LBB447_54:
 mov     byte, ptr, [rdx, +, rbx], dil
 mov     rbx, rsi
     cmp     rsi, r12
     jae     .LBB447_44
.LBB447_55:
 *level = clamp(coeff.abs(), T::cast_from(0), T::cast_from(127)).as_();
 movzx   esi, word, ptr, [rcx, +, 2*rbx]
     if self.is_negative() { -*self } else { *self } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/sign.rs:49)
     mov     edi, esi
     neg     di
     cmovl   di, si
     if input < min { (src/util/math.rs:40)
     movsx   esi, di
     cmp     esi, 128
     jl      .LBB447_57
     mov     edi, 127
.LBB447_57:
     lea     rsi, [rbx, +, 1]
     test    di, di
     #[inline] fn as_(self) -> $U { self as $U } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/cast.rs:746)
     jg      .LBB447_54
     xor     edi, edi
     jmp     .LBB447_54
.LBB447_59:
     xor     edi, edi
     test    r9, r9
     jne     .LBB447_52
     jmp     .LBB447_53
.LBB447_60:
 let plane_type = if plane == 0 { 0 } else { 1 } as usize;
 xor     eax, eax
 mov     qword, ptr, [rsp], rax
 cmp     qword, ptr, [rsp, +, 72], 0
 setne   bl
 mov     ecx, dword, ptr, [rbp, +, 48]
 mov     r12d, dword, ptr, [rbp, +, 56]
 if plane == 0 {
 je      .LBB447_63
 mov     rdx, qword, ptr, [rbp, +, 32]
     let t = if eob < 33 { (src/context.rs:3867)
     cmp     rdx, 32
     mov     qword, ptr, [rsp, +, 120], r14
     let t = if eob < 33 { (src/context.rs:3867)
     ja      .LBB447_64
.LBB447_62:
     eob_to_pos_small[eob] as u32 (src/context.rs:3868)
     lea     rax, [rip, +, _ZN5rav1e7context16eob_to_pos_small17h96a044e8ba93da08E]
     add     rax, rdx
     movzx   r14d, byte, ptr, [rax]
     cmp     r14, 11
     assert!(eob as i32 >= k_eob_group_start[t as usize] as i32); (src/context.rs:3873)
     ja      .LBB447_132
.LBB447_65:
     lea     rax, [rip, +, _ZN5rav1e7context17k_eob_group_start17h82d065e9f0c70a19E]
     movzx   eax, word, ptr, [rax, +, 2*r14]
     assert!(eob as i32 >= k_eob_group_start[t as usize] as i32); (src/context.rs:3873)
     mov     r15d, edx
     sub     r15d, eax
     assert!(eob as i32 >= k_eob_group_start[t as usize] as i32); (src/context.rs:3873)
     jl      .LBB447_133
 lea     rax, [rip, +, _ZN5rav1e7context16tx_type_to_class17h5e3c0d86560233a5E]
 mov     rcx, qword, ptr, [rsp]
 mov     cl, bl
 mov     qword, ptr, [rsp], rcx
 mov     r13b, byte, ptr, [r12, +, rax]
     TX_4X4 | TX_4X8 | TX_4X16 => 2, (src/transform/mod.rs:116)
     lea     rax, [rip, +, .Lswitch.table._ZN5rav1e11recon_intra13has_top_right17h21096a7ec98f3035E]
     mov     ecx, dword, ptr, [rbp, +, 48]
     mov     rdx, rcx
     mov     rcx, qword, ptr, [rax, +, 8*rcx]
     TX_4X4 | TX_8X4 | TX_16X4 => 2, (src/transform/mod.rs:134)
     lea     rax, [rip, +, .Lswitch.table._ZN5rav1e11recon_intra15has_bottom_left17ha8c8ba4e0e3f0f8eE.398]
     mov     rdx, qword, ptr, [rax, +, 8*rdx]
 #[derive(Copy, Clone, PartialEq)]
 xor     eax, eax
 test    r13b, r13b
 setne   al
 eob_pt - 1,
 lea     esi, [r14, -, 1]
 0 => &mut self.fc.eob_flag_cdf16[plane_type][eob_multi_ctx],
 add     rcx, rdx
 add     rcx, -4
 cmp     rcx, 5
 ja      .LBB447_74
 lea     rdx, [rip, +, .LJTI447_4]
 movsxd  rcx, dword, ptr, [rdx, +, 4*rcx]
 add     rcx, rdx
 mov     rdx, qword, ptr, [rsp, +, 16]
 mov     rdi, qword, ptr, [rsp, +, 8]
 jmp     rcx
.LBB447_68:
 mov     rcx, qword, ptr, [rsp]
 0 => &mut self.fc.eob_flag_cdf16[plane_type][eob_multi_ctx],
 lea     rcx, [rcx, +, 2*rcx]
 shl     rcx, 3
 add     rcx, qword, ptr, [rdx, +, 4696]
 lea     rax, [rax, +, 2*rax]
 lea     rdx, [rcx, +, 4*rax]
 add     rdx, 10512
 mov     ecx, 6
 jmp     .LBB447_75
.LBB447_63:
 mov     dl, byte, ptr, [rbp, +, 40]
     #[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)] (src/predict.rs:64)
     xor     eax, eax
     cmp     dl, 13
     seta    al
     mov     rsi, qword, ptr, [rsp, +, 16]
 self.write_tx_type(
 mov     rdi, qword, ptr, [rsi, +, 4696]
 movzx   r10d, byte, ptr, [rbp, +, 88]
 movzx   r9d, dl
 mov     rsi, qword, ptr, [rsp, +, 8]
 mov     rdx, qword, ptr, [rsp, +, 24]
 mov     r8d, r12d
 push    r10
 push    rax
 call    rav1e::context::ContextWriter::write_tx_type
 add     rsp, 16
 mov     rdx, qword, ptr, [rbp, +, 32]
     let t = if eob < 33 { (src/context.rs:3867)
     cmp     rdx, 32
     mov     qword, ptr, [rsp, +, 120], r14
     let t = if eob < 33 { (src/context.rs:3867)
     jbe     .LBB447_62
.LBB447_64:
     let e = cmp::min((eob - 1) >> 5, 16); (src/context.rs:3870)
     lea     rax, [rdx, -, 1]
     shr     rax, 5
     cmp     rax, 16
     mov     ecx, 16
     cmovb   rcx, rax
     eob_to_pos_large[e as usize] as u32 (src/context.rs:3871)
     lea     rax, [rip, +, _ZN5rav1e7context16eob_to_pos_large17h4148ccf24f4baf5bE]
     add     rax, rcx
     movzx   r14d, byte, ptr, [rax]
     cmp     r14, 11
     assert!(eob as i32 >= k_eob_group_start[t as usize] as i32); (src/context.rs:3873)
     jbe     .LBB447_65
.LBB447_132:
     lea     rdi, [rip, +, .Lanon.ae0d0e2763daf29edd045d9223ca4411.313]
     mov     edx, 12
 mov     rsi, r14
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_69:
 mov     rcx, qword, ptr, [rsp]
 4 => &mut self.fc.eob_flag_cdf256[plane_type][eob_multi_ctx],
 lea     rcx, [rcx, +, 4*rcx]
 shl     rcx, 3
 add     rcx, qword, ptr, [rdx, +, 4696]
 lea     rax, [rax, +, 4*rax]
 lea     rdx, [rcx, +, 4*rax]
 add     rdx, 10752
 mov     ecx, 10
 jmp     .LBB447_75
.LBB447_70:
 mov     rcx, qword, ptr, [rsp]
 2 => &mut self.fc.eob_flag_cdf64[plane_type][eob_multi_ctx],
 shl     rcx, 5
 add     rcx, qword, ptr, [rdx, +, 4696]
 shl     rax, 4
 lea     rdx, [rax, +, rcx]
 add     rdx, 10616
 mov     ecx, 8
 jmp     .LBB447_75
.LBB447_71:
 mov     rcx, qword, ptr, [rsp]
 3 => &mut self.fc.eob_flag_cdf128[plane_type][eob_multi_ctx],
 lea     rcx, [rcx, +, 8*rcx]
 shl     rcx, 2
 add     rcx, qword, ptr, [rdx, +, 4696]
 lea     rax, [rax, +, 8*rax]
 lea     rdx, [rcx, +, 2*rax]
 add     rdx, 10680
 mov     ecx, 9
 jmp     .LBB447_75
.LBB447_72:
 1 => &mut self.fc.eob_flag_cdf32[plane_type][eob_multi_ctx],
 mov     rcx, rax
 shl     rcx, 4
 sub     rcx, rax
 sub     rcx, rax
 mov     rbx, qword, ptr, [rsp]
 lea     rax, [rbx, +, 8*rbx]
 lea     rax, [rax, +, 2*rax]
 add     rax, rbx
 add     rax, qword, ptr, [rdx, +, 4696]
 lea     rdx, [rcx, +, rax]
 add     rdx, 10560
 mov     ecx, 7
 jmp     .LBB447_75
.LBB447_73:
 5 => &mut self.fc.eob_flag_cdf512[plane_type][eob_multi_ctx],
 imul    rcx, qword, ptr, [rsp], 44
 add     rcx, qword, ptr, [rdx, +, 4696]
 lea     rdx, [rax, +, 4*rax]
 lea     rdx, [rax, +, 4*rdx]
 add     rdx, rax
 add     rdx, rcx
 add     rdx, 10832
 mov     ecx, 11
 jmp     .LBB447_75
.LBB447_74:
 mov     rcx, qword, ptr, [rsp]
 _ => &mut self.fc.eob_flag_cdf1024[plane_type][eob_multi_ctx],
 lea     rcx, [rcx, +, 2*rcx]
 shl     rcx, 4
 mov     rdx, qword, ptr, [rsp, +, 16]
 add     rcx, qword, ptr, [rdx, +, 4696]
 lea     rax, [rax, +, 2*rax]
 lea     rdx, [rcx, +, 8*rax]
 add     rdx, 10920
 mov     ecx, 12
 mov     rdi, qword, ptr, [rsp, +, 8]
.LBB447_75:
 $w.symbol_with_update($s, $cdf);
 call    qword, ptr, [rsp, +, 56]
 let eob_offset_bits = k_eob_offset_bits[eob_pt as usize];
 lea     rax, [rip, +, _ZN5rav1e7context17k_eob_offset_bits17h4ed36c7d9fbabeb5E]
 movzx   ebx, word, ptr, [rax, +, 2*r14]
 if eob_offset_bits > 0 {
 test    bx, bx
 if eob_offset_bits > 0 {
 je      .LBB447_80
 &mut self.fc.eob_extra_cdf[txs_ctx][plane_type][(eob_pt - 3) as usize]
 lea     esi, [r14, -, 3]
 &mut self.fc.eob_extra_cdf[txs_ctx][plane_type][(eob_pt - 3) as usize]
 cmp     esi, 8
 ja      .LBB447_135
 lea     ecx, [rbx, -, 1]
 if (eob_extra & (1 << eob_shift)) != 0 { 1 } else { 0 };
 xor     eax, eax
 bt      r15d, ecx
 setb    al
 &mut self.fc.eob_extra_cdf[txs_ctx][plane_type][(eob_pt - 3) as usize]
 imul    rcx, qword, ptr, [rsp, +, 48], 108
 mov     rdx, qword, ptr, [rsp, +, 16]
 add     rcx, qword, ptr, [rdx, +, 4696]
 imul    rdx, qword, ptr, [rsp], 54
 add     rdx, rcx
 lea     rcx, [rsi, +, 2*rsi]
 lea     rdx, [rdx, +, 2*rcx]
 add     rdx, 9972
 $w.symbol_with_update($s, $cdf);
 mov     ecx, 3
 mov     r12, qword, ptr, [rsp, +, 8]
 mov     rdi, r12
 mov     esi, eax
 call    qword, ptr, [rsp, +, 56]
     cmp     bx, 2
     jb      .LBB447_80
     mov     rax, qword, ptr, [rsp, +, 24]
     mov     r14, qword, ptr, [rax, +, 56]
     add     ebx, -2
.LBB447_79:
 bit = if (eob_extra & (1 << eob_shift)) != 0 { 1 } else { 0 };
 xor     esi, esi
 bt      r15d, ebx
 setb    sil
 w.bit(bit as u16);
 mov     rdi, r12
 call    r14
     add     ebx, -1
     cmp     bx, -1
     jne     .LBB447_79
.LBB447_80:
     lea     r10, [rsp, +, 7008]
 self.get_nz_map_contexts(
 movzx   eax, r13b
 mov     rdi, qword, ptr, [rsp, +, 88]
 mov     rsi, qword, ptr, [rsp, +, 120]
 mov     rdx, qword, ptr, [rsp, +, 104]
 mov     rcx, qword, ptr, [rsp, +, 128]
 mov     r14, qword, ptr, [rbp, +, 32]
 mov     r8d, r14d
 mov     ebx, dword, ptr, [rbp, +, 48]
 mov     r9d, ebx
 push    r10
 mov     qword, ptr, [rsp, +, 104], rax
 push    rax
 call    rav1e::context::ContextWriter::get_nz_map_contexts
 mov     r8, r14
 add     rsp, 16
     TX_64X64 | TX_64X32 | TX_32X64 => TX_32X32, (src/context.rs:1955)
     lea     rax, [rip, +, .Lswitch.table._ZN5rav1e7context13ContextWriter19get_nz_map_contexts17h8a6a1758ea33f613E]
     mov     rax, qword, ptr, [rax, +, 8*rbx]
     mov     qword, ptr, [rsp, +, 80], rax
     mov     rcx, qword, ptr, [rsp, +, 48]
     cmp     rcx, 3
     mov     eax, 3
     cmovb   rax, rcx
     imul    r15, rax, 420
     jmp     .LBB447_82
.LBB447_81:
     test    r14, r14
     mov     r8, qword, ptr, [rbp, +, 32]
     je      .LBB447_94
.LBB447_82:
     mov     rcx, r14
     add     r14, -1
     mov     rax, qword, ptr, [rsp, +, 128]
 let pos = scan[c] as usize;
 cmp     r14, rax
 mov     rbx, qword, ptr, [rsp, +, 16]
 mov     rdx, qword, ptr, [rsp, +, 32]
 mov     rdi, qword, ptr, [rsp, +, 8]
 jae     .LBB447_119
 mov     rax, qword, ptr, [rsp, +, 104]
 movzx   r13d, word, ptr, [rax, +, 2*rcx, -, 2]
 cmp     r13, 4095
 let coeff_ctx = coeff_contexts.array[pos];
 ja      .LBB447_120
 let v = coeffs[c];
 cmp     r14, rdx
 jae     .LBB447_121
 movsx   rsi, byte, ptr, [rsp, +, r13, +, 7008]
 movzx   eax, word, ptr, [rsp, +, 2*rcx, +, 158]
     if self.is_negative() { -*self } else { *self } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/sign.rs:49)
     mov     r12d, eax
     neg     r12w
     cmovl   r12w, ax
     #[inline] fn as_(self) -> $U { self as $U } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/cast.rs:746)
     movsx   eax, r12w
     cmp     eax, 3
     mov     edx, 3
     cmovae  eax, edx
 if c == eob - 1 {
 cmp     rcx, r8
 if c == eob - 1 {
 jne     .LBB447_88
 &mut self.fc.coeff_base_eob_cdf[txs_ctx][plane_type]
 cmp     sil, 3
 ja      .LBB447_123
 mov     rcx, qword, ptr, [rsp, +, 48]
 &mut self.fc.coeff_base_eob_cdf[txs_ctx][plane_type]
 shl     rcx, 6
 add     rcx, qword, ptr, [rbx, +, 4696]
 (cmp::min(u32::cast_from(level), 3) - 1) as u32,
 add     eax, -1
 mov     rdx, qword, ptr, [rsp]
 &mut self.fc.coeff_base_eob_cdf[txs_ctx][plane_type]
 shl     rdx, 5
 add     rdx, rcx
 lea     rdx, [rdx, +, 8*rsi]
 add     rdx, 11016
 $w.symbol_with_update($s, $cdf);
 mov     ecx, 4
 mov     esi, eax
 call    qword, ptr, [rsp, +, 56]
     cmp     r12w, 3
 if level > T::cast_from(NUM_BASE_LEVELS) {
 jl      .LBB447_81
 jmp     .LBB447_90
.LBB447_88:
 &mut self.fc.coeff_base_cdf[txs_ctx][plane_type][coeff_ctx as usize]
 cmp     sil, 41
 ja      .LBB447_124
 &mut self.fc.coeff_base_cdf[txs_ctx][plane_type][coeff_ctx as usize]
 imul    rcx, qword, ptr, [rsp, +, 48], 840
 add     rcx, qword, ptr, [rbx, +, 4696]
 imul    rdx, qword, ptr, [rsp], 420
 add     rdx, rcx
 lea     rcx, [rsi, +, 4*rsi]
 lea     rdx, [rdx, +, 2*rcx]
 add     rdx, 11336
 $w.symbol_with_update($s, $cdf);
 mov     ecx, 5
 mov     esi, eax
 call    qword, ptr, [rsp, +, 56]
     cmp     r12w, 3
 if level > T::cast_from(NUM_BASE_LEVELS) {
 jl      .LBB447_81
.LBB447_90:
 mov     rdi, qword, ptr, [rsp, +, 88]
 mov     rsi, qword, ptr, [rsp, +, 120]
 let br_ctx = Self::get_br_ctx(levels, pos, bhl, tx_class);
 mov     rdx, r13
 mov     rcx, qword, ptr, [rsp, +, 80]
 mov     r8, qword, ptr, [rsp, +, 96]
 call    rav1e::context::ContextWriter::get_br_ctx
 mov     r13, rax
 cmp     rax, 20
 if idx >= T::cast_from(COEFF_BASE_RANGE) {
 ja      .LBB447_125
 if k < T::cast_from(BR_CDF_SIZE - 1) {
 add     r12d, -3
 mov     bx, 3
.LBB447_92:
 cmp     r12w, 4
 mov     eax, 3
 cmovl   eax, r12d
     #[inline] fn as_(self) -> $U { self as $U } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/cast.rs:746)
     movsx   esi, ax
     mov     rax, qword, ptr, [rsp, +, 16]
     mov     rax, qword, ptr, [rax, +, 4696]
 &mut self.fc.coeff_br_cdf
 add     rax, r15
 imul    rcx, qword, ptr, [rsp], 210
 add     rcx, rax
 lea     rax, [4*r13]
 add     rax, r13
 lea     rdx, [rcx, +, 2*rax]
 add     rdx, 15536
 $w.symbol_with_update($s, $cdf);
 mov     ecx, 5
 mov     rdi, qword, ptr, [rsp, +, 8]
 call    qword, ptr, [rsp, +, 56]
     cmp     r12w, 3
 if k < T::cast_from(BR_CDF_SIZE - 1) {
 jl      .LBB447_81
 lea     eax, [rbx, +, 3]
 add     r12d, -3
 cmp     bx, 11
 mov     ebx, eax
 jbe     .LBB447_92
 jmp     .LBB447_81
.LBB447_94:
 mov     rax, qword, ptr, [rsp, +, 112]
 cmp     rax, 3
 mov     rdx, qword, ptr, [rsp, +, 32]
     jae     .LBB447_104
     xor     r12d, r12d
     mov     rcx, qword, ptr, [rsp]
     lea     r15, [rcx, +, 8*rcx]
     add     r15, r15
     lea     r14, [rax, +, 2*rax]
     mov     r13d, dword, ptr, [rbp, +, 48]
 let v = coeffs[c];
 cmp     rdx, r12
 jne     .LBB447_98
 jmp     .LBB447_122
.LBB447_102:
     add     ebx, -15
     #[inline] fn as_(self) -> $U { self as $U } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/cast.rs:746)
     movsx   esi, bx
     mov     rdi, qword, ptr, [rsp, +, 8]
     mov     rax, qword, ptr, [rsp, +, 24]
 w.write_golomb(u32::cast_from(
 call    qword, ptr, [rax, +, 72]
 mov     r8, qword, ptr, [rbp, +, 32]
 mov     rdx, qword, ptr, [rsp, +, 32]
.LBB447_96:
     cmp     r8, r12
     je      .LBB447_111
 let v = coeffs[c];
 cmp     rdx, r12
 je      .LBB447_122
.LBB447_98:
 mov     rax, r12
 add     r12, 1
 movzx   esi, word, ptr, [rsp, +, 2*rax, +, 160]
 test    esi, esi
 if v == T::cast_from(0) {
 je      .LBB447_96
     if self.is_negative() { -*self } else { *self } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/sign.rs:49)
     mov     ebx, esi
     neg     ebx
     fn is_negative(&self) -> bool { *self < 0 } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/sign.rs:70)
     test    si, si
     if self.is_negative() { -*self } else { *self } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/sign.rs:49)
     cmovns  ebx, esi
 let sign = if v < T::cast_from(0) { 1 } else { 0 };
 shr     esi, 15
 if c == 0 {
 test    rax, rax
 if c == 0 {
 je      .LBB447_101
 mov     rdi, qword, ptr, [rsp, +, 8]
 mov     rax, qword, ptr, [rsp, +, 24]
 w.bit(sign as u16);
 call    qword, ptr, [rax, +, 56]
     cmp     bx, 15
     mov     rdx, qword, ptr, [rsp, +, 32]
     mov     r8, qword, ptr, [rbp, +, 32]
 if level > T::cast_from(COEFF_BASE_RANGE + NUM_BASE_LEVELS) {
 jl      .LBB447_96
 jmp     .LBB447_102
.LBB447_101:
 mov     rax, qword, ptr, [rsp, +, 16]
 mov     rax, qword, ptr, [rax, +, 4696]
 &mut self.fc.dc_sign_cdf[plane_type][txb_ctx.dc_sign_ctx]
 add     rax, r15
 lea     rdx, [rax, +, 2*r14]
 add     rdx, 9936
 $w.symbol_with_update($s, $cdf);
 mov     ecx, 3
 mov     rdi, qword, ptr, [rsp, +, 8]
 call    qword, ptr, [rsp, +, 56]
     cmp     bx, 15
     mov     rdx, qword, ptr, [rsp, +, 32]
     mov     r8, qword, ptr, [rbp, +, 32]
 if level > T::cast_from(COEFF_BASE_RANGE + NUM_BASE_LEVELS) {
 jl      .LBB447_96
 jmp     .LBB447_102
.LBB447_103:
     mov     eax, r13d
     jmp     .LBB447_8
.LBB447_104:
     xor     r14d, r14d
     mov     r13d, dword, ptr, [rbp, +, 48]
 let v = coeffs[c];
 cmp     rdx, r14
 jne     .LBB447_107
 jmp     .LBB447_122
.LBB447_105:
 add     r14, 1
     cmp     r8, r14
     je      .LBB447_111
 cmp     rdx, r14
 je      .LBB447_122
.LBB447_107:
 movzx   eax, word, ptr, [rsp, +, 2*r14, +, 160]
     test    ax, ax
 if v == T::cast_from(0) {
 je      .LBB447_105
     if self.is_negative() { -*self } else { *self } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/sign.rs:49)
     mov     ebx, eax
     neg     bx
     cmovl   bx, ax
 if c == 0 {
 test    r14, r14
 if c == 0 {
 je      .LBB447_136
 let sign = if v < T::cast_from(0) { 1 } else { 0 };
 movzx   esi, ax
 shr     esi, 15
 mov     rdi, qword, ptr, [rsp, +, 8]
 mov     rax, qword, ptr, [rsp, +, 24]
 w.bit(sign as u16);
 call    qword, ptr, [rax, +, 56]
     cmp     bx, 15
     mov     rdx, qword, ptr, [rsp, +, 32]
     mov     r8, qword, ptr, [rbp, +, 32]
 if level > T::cast_from(COEFF_BASE_RANGE + NUM_BASE_LEVELS) {
 jl      .LBB447_105
     add     ebx, -15
     #[inline] fn as_(self) -> $U { self as $U } (/home/soichiro/.cargo/registry/src/github.com-1ecc6299db9ec823/num-traits-0.2.11/src/cast.rs:746)
     movsx   esi, bx
     mov     rdi, qword, ptr, [rsp, +, 8]
     mov     rax, qword, ptr, [rsp, +, 24]
 w.write_golomb(u32::cast_from(
 call    qword, ptr, [rax, +, 72]
 mov     r8, qword, ptr, [rbp, +, 32]
 mov     rdx, qword, ptr, [rsp, +, 32]
 jmp     .LBB447_105
.LBB447_111:
 mov     ecx, dword, ptr, [rsp, +, 68]
     cmp     ecx, 63
     mov     eax, 63
     cmovb   eax, ecx
 cul_level = cmp::min(COEFF_CONTEXT_MASK as u32, cul_level);
 mov     dword, ptr, [rsp, +, 44], eax
     test    rdx, rdx
 BlockContext::set_dc_sign(&mut cul_level, i32::cast_from(coeffs[0]));
 je      .LBB447_134
     if dc_val < 0 { (src/context.rs:1582)
     cmp     word, ptr, [rsp, +, 160], 0
     mov     rcx, qword, ptr, [rsp, +, 144]
     mov     rdx, qword, ptr, [rsp, +, 136]
     mov     rdi, qword, ptr, [rsp, +, 16]
     if dc_val < 0 { (src/context.rs:1582)
     js      .LBB447_115
     } else if dc_val > 0 { (src/context.rs:1584)
     je      .LBB447_117
     *cul_level += 2 << COEFF_CONTEXT_BITS; (src/context.rs:1585)
     or      eax, 128
     jmp     .LBB447_116
.LBB447_115:
     *cul_level |= 1 << COEFF_CONTEXT_BITS; (src/context.rs:1583)
     or      eax, 64
.LBB447_116:
     mov     dword, ptr, [rsp, +, 44], eax
.LBB447_117:
     mov     rsi, qword, ptr, [rsp, +, 72]
 self.bc.set_coeff_context(plane, bo, tx_size, xdec, ydec, cul_level as u8);
 mov     r8d, r13d
 mov     r9, qword, ptr, [rbp, +, 72]
 push    rax
 push    qword, ptr, [rbp, +, 80]
 call    rav1e::context::BlockContext::set_coeff_context
 add     rsp, 16
 mov     al, 1
.LBB447_118:
 }
 lea     rsp, [rbp, -, 40]
 pop     rbx
 pop     r12
 pop     r13
 pop     r14
 pop     r15
 pop     rbp
 ret
.LBB447_119:
 let pos = scan[c] as usize;
 lea     rdi, [rip, +, .L__unnamed_471]
 mov     rsi, r14
 mov     rdx, rax
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_120:
 let coeff_ctx = coeff_contexts.array[pos];
 lea     rdi, [rip, +, .L__unnamed_472]
 mov     edx, 4096
 mov     rsi, r13
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_121:
 let v = coeffs[c];
 lea     rdi, [rip, +, .L__unnamed_473]
 mov     rsi, r14
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_122:
 let v = coeffs[c];
 lea     rdi, [rip, +, .L__unnamed_474]
 mov     rsi, rdx
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_123:
 &mut self.fc.coeff_base_eob_cdf[txs_ctx][plane_type]
 lea     rdi, [rip, +, .L__unnamed_475]
 mov     edx, 4
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_124:
 &mut self.fc.coeff_base_cdf[txs_ctx][plane_type][coeff_ctx as usize]
 lea     rdi, [rip, +, .L__unnamed_476]
 mov     edx, 42
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_125:
 &mut self.fc.coeff_br_cdf
 lea     rdi, [rip, +, .L__unnamed_477]
 mov     edx, 21
 mov     rsi, r13
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_126:
 coeffs[i] = coeffs_in[scan_idx as usize];
 lea     rdi, [rip, +, .L__unnamed_478]
 mov     rdx, r9
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_127:
 coeffs[i] = coeffs_in[scan_idx as usize];
 lea     rdi, [rip, +, .L__unnamed_479]
 mov     rsi, rdx
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_128:
     mov     esi, 1024
     mov     rdi, rdx
     call    core::slice::slice_index_len_fail
     ud2
.LBB447_129:
 let cdf = &mut self.fc.txb_skip_cdf[txs_ctx][txb_ctx.txb_skip_ctx];
 lea     rdi, [rip, +, .L__unnamed_480]
 mov     edx, 5
 mov     rsi, qword, ptr, [rsp, +, 48]
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_130:
 lea     rdi, [rip, +, .L__unnamed_480]
 mov     edx, 13
 mov     rsi, rax
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_131:
     mov     esi, 4776
     call    core::slice::slice_index_order_fail
     ud2
.LBB447_133:
     assert!(eob as i32 >= k_eob_group_start[t as usize] as i32); (src/context.rs:3873)
     lea     rdi, [rip, +, .Lanon.ae0d0e2763daf29edd045d9223ca4411.315]
     lea     rdx, [rip, +, .Lanon.ae0d0e2763daf29edd045d9223ca4411.314]
     mov     esi, 68
     call    std::panicking::begin_panic
     ud2
.LBB447_134:
 BlockContext::set_dc_sign(&mut cul_level, i32::cast_from(coeffs[0]));
 lea     rdi, [rip, +, .L__unnamed_481]
 xor     esi, esi
 xor     edx, edx
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_135:
 &mut self.fc.eob_extra_cdf[txs_ctx][plane_type][(eob_pt - 3) as usize]
 lea     rdi, [rip, +, .L__unnamed_482]
 mov     edx, 9
 call    core::panicking::panic_bounds_check
 ud2
.LBB447_136:
 &mut self.fc.dc_sign_cdf[plane_type][txb_ctx.dc_sign_ctx]
 lea     rdi, [rip, +, .L__unnamed_483]
 mov     edx, 3
 mov     rsi, qword, ptr, [rsp, +, 112]
 call    core::panicking::panic_bounds_check
 ud2

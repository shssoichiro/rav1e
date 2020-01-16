 fn get_nz_map_contexts(
 push    rbp
 mov     rbp, rsp
 push    r15
 push    r14
 push    r13
 push    r12
 push    rbx
 sub     rsp, 104
 mov     r11, rcx
 mov     qword, ptr, [rbp, -, 144], rsi
 movsxd  rbx, dword, ptr, [rbp, +, 16]
 mov     ecx, 2
 mov     r10d, 9
 mov     edi, 3
 TX_64X64 | TX_64X32 | TX_32X64 => TX_32X32,
 mov     eax, ebx
 lea     rsi, [rip, +, LJTI379_0]
 mov     qword, ptr, [rbp, -, 128], rax
 movsxd  rax, dword, ptr, [rsi, +, 4*rax]
 add     rax, rsi
 mov     esi, ebx
 jmp     rax
LBB379_2:
 mov     ecx, 3
 mov     esi, ebx
 jmp     LBB379_7
LBB379_3:
 mov     r10d, ebx
LBB379_4:
 mov     ecx, 4
 mov     esi, r10d
 jmp     LBB379_7
LBB379_5:
 mov     edi, ebx
 jmp     LBB379_6
LBB379_1:
 mov     edi, 10
LBB379_6:
 mov     ecx, 5
 mov     esi, edi
LBB379_7:
     TX_4X4 | TX_8X4 | TX_16X4 => 2, (src/transform/mod.rs:134)
     movsxd  rax, esi
     lea     rsi, [rip, +, l_switch.table._ZN5rav1e11recon_intra15has_bottom_left17hca714326086cb015E.398]
     self.width_log2() + self.height_log2() (src/transform/mod.rs:155)
     add     ecx, dword, ptr, [rsi, +, 8*rax]
     mov     r10d, 1
     1 << self.area_log2() (src/transform/mod.rs:151)
     mov     edi, 1
     shl     rdi, cl
     test    r9w, r9w
     je      LBB379_76
     lea     rax, [rip, +, l_switch.table._ZN5rav1e7context13ContextWriter19get_nz_map_contexts17ha95c07bd33304f58E]
     mov     rcx, qword, ptr, [rax, +, 8*rbx]
     mov     eax, 1
     shl     rax, cl
     lea     r13d, [r9, -, 1]
     lea     r12, [rax, +, 4]
     add     rax, 5
     mov     qword, ptr, [rbp, -, 64], rax
     mov     ebx, 2
     shl     rbx, cl
     mov     eax, 3
     shl     rax, cl
     mov     esi, 4
     mov     qword, ptr, [rbp, -, 120], rcx
     shl     rsi, cl
     mov     rcx, r11
     add     rbx, 8
     mov     qword, ptr, [rbp, -, 112], rbx
     add     rax, 12
     mov     qword, ptr, [rbp, -, 56], rax
     add     rsi, 16
     mov     qword, ptr, [rbp, -, 136], rsi
     mov     rax, rdi
     shr     rax, 3
     shr     rdi, 2
     movzx   esi, r13w
 cmp     rdi, rsi
 mov     r11d, 0
 mov     r14, qword, ptr, [rbp, +, 40]
 setb    r11b
 or      r11, 2
 cmp     rax, rsi
 mov     rdi, qword, ptr, [rbp, +, 32]
 cmovae  r11, r10
 xor     r15d, r15d
 test    r13w, r13w
 cmove   r11, r15
     movzx   ebx, r9w
 movzx   eax, byte, ptr, [rbp, +, 24]
 mov     qword, ptr, [rbp, -, 104], rax
 mov     qword, ptr, [rbp, -, 96], r8
 mov     qword, ptr, [rbp, -, 88], rdx
 mov     qword, ptr, [rbp, -, 48], rcx
 mov     qword, ptr, [rbp, -, 80], r12
 mov     qword, ptr, [rbp, -, 72], rbx
LBB379_9:
 let pos = scan[i as usize];
 cmp     r8, r15
 je      LBB379_82
 movzx   r9d, word, ptr, [rcx, +, 2*r15]
 mov     rax, r11
 i == eob - 1,
 cmp     rsi, r15
 if is_eob {
 je      LBB379_74
 let padded_idx = coeff_idx + ((coeff_idx >> bhl) << TX_PAD_HOR_LOG2);
 mov     r13, r9
 mov     rcx, qword, ptr, [rbp, -, 120]
 shr     r13, cl
 let padded_idx = coeff_idx + ((coeff_idx >> bhl) << TX_PAD_HOR_LOG2);
 lea     rdi, [r9, +, 4*r13]
     cmp     rdi, rdx
     ja      LBB379_84
     mov     rax, rdx
     sub     rax, rdi
 let mut mag = cmp::min(3, levels[1]); // { 1, 0 }
 cmp     rax, 2
 jb      LBB379_44
 mag += cmp::min(3, levels[(1 << bhl) + TX_PAD_HOR]); // { 0, 1 }
 cmp     r12, rax
 jae     LBB379_45
 mov     r10, rsi
 add     rdi, qword, ptr, [rbp, -, 144]
 movzx   edx, byte, ptr, [rdi, +, 1]
     cmp     dl, 3
     jb      LBB379_15
     mov     ebx, 3
     movzx   r8d, byte, ptr, [rdi, +, r12]
     cmp     r8b, 3
     jae     LBB379_18
     jmp     LBB379_19
LBB379_15:
     movzx   ebx, dl
     movzx   r8d, byte, ptr, [rdi, +, r12]
     cmp     r8b, 3
     jb      LBB379_19
LBB379_18:
     mov     r8d, 3
LBB379_19:
     movzx   edx, byte, ptr, [rbp, +, 24]
 if tx_class == TX_CLASS_2D {
 cmp     dl, 2
 je      LBB379_30
 test    dl, dl
 jne     LBB379_37
 mag += cmp::min(3, levels[(1 << bhl) + TX_PAD_HOR + 1]); // { 1, 1 }
 cmp     qword, ptr, [rbp, -, 64], rax
 jae     LBB379_46
 mag += cmp::min(3, levels[2]); // { 2, 0 }
 cmp     rax, 3
 jb      LBB379_47
 mov     rdx, qword, ptr, [rbp, -, 64]
 movzx   edx, byte, ptr, [rdi, +, rdx]
     cmp     dl, 3
     jb      LBB379_24
     mov     r12d, 3
     mov     rdx, qword, ptr, [rbp, -, 112]
 mag += cmp::min(3, levels[(2 << bhl) + (2 << TX_PAD_HOR_LOG2)]); // { 0, 2 }
 cmp     rdx, rax
 jb      LBB379_27
 jmp     LBB379_49
LBB379_30:
 mag += cmp::min(3, levels[2]); // { 2, 0 }
 cmp     rax, 3
 jb      LBB379_51
 mag += cmp::min(3, levels[3]); // { 3, 0 }
 je      LBB379_52
 movzx   edx, byte, ptr, [rdi, +, 2]
     cmp     dl, 3
     jb      LBB379_33
     mov     r12d, 3
 mag += cmp::min(3, levels[4]); // { 4, 0 }
 cmp     rax, 4
 ja      LBB379_36
 jmp     LBB379_53
LBB379_37:
 mov     rsi, qword, ptr, [rbp, -, 112]
 mag += cmp::min(3, levels[(2 << bhl) + (2 << TX_PAD_HOR_LOG2)]); // { 0, 2 }
 cmp     rsi, rax
 jae     LBB379_54
 mag += cmp::min(3, levels[(3 << bhl) + (3 << TX_PAD_HOR_LOG2)]); // { 0, 3 }
 cmp     qword, ptr, [rbp, -, 56], rax
 jae     LBB379_55
 movzx   edx, byte, ptr, [rdi, +, rsi]
     cmp     dl, 3
     jb      LBB379_40
     mov     r12d, 3
     mov     rdx, qword, ptr, [rbp, -, 136]
 mag += cmp::min(3, levels[(4 << bhl) + (4 << TX_PAD_HOR_LOG2)]); // { 0, 4 }
 cmp     rdx, rax
 jb      LBB379_43
 jmp     LBB379_56
LBB379_24:
 movzx   r12d, dl
 mov     rdx, qword, ptr, [rbp, -, 112]
 mag += cmp::min(3, levels[(2 << bhl) + (2 << TX_PAD_HOR_LOG2)]); // { 0, 2 }
 cmp     rdx, rax
 jae     LBB379_49
LBB379_27:
 lea     rax, [rdi, +, 2]
 if (tx_class as u32 | coeff_idx as u32) == 0 {
 mov     esi, r9d
 or      esi, dword, ptr, [rbp, -, 104]
 if (tx_class as u32 | coeff_idx as u32) == 0 {
 je      LBB379_29
LBB379_57:
 add     r8b, bl
 add     r12b, r8b
 movzx   eax, byte, ptr, [rax]
 movzx   edx, byte, ptr, [rdi, +, rdx]
 mov     esi, 3
 cmp     al, 3
 jb      LBB379_58
 mov     edi, 3
 add     r12b, dil
 cmp     dl, 3
 jb      LBB379_61
 jmp     LBB379_62
LBB379_58:
 movzx   edi, al
 add     r12b, dil
 cmp     dl, 3
 jae     LBB379_62
LBB379_61:
 movzx   esi, dl
LBB379_62:
 let row: usize = coeff_idx - (col << bhl);
 mov     rbx, r13
 mov     rcx, qword, ptr, [rbp, -, 120]
 shl     rbx, cl
 add     r12b, sil
 let ctx = ((stats + 1) >> 1).min(4);
 inc     r12b
 let ctx = ((stats + 1) >> 1).min(4);
 shr     r12b
     cmp     r12b, 4
     jb      LBB379_63
     mov     edi, 4
     jmp     LBB379_65
LBB379_63:
     movzx   edi, r12b
LBB379_65:
     mov     r8, qword, ptr, [rbp, -, 96]
     mov     rdx, qword, ptr, [rbp, -, 88]
     mov     rcx, qword, ptr, [rbp, -, 48]
     mov     r12, qword, ptr, [rbp, -, 80]
     mov     rsi, qword, ptr, [rbp, -, 104]
     mov     rax, r9
     sub     rax, rbx
 TX_CLASS_2D => {
 test    rsi, rsi
 je      LBB379_69
 cmp     rsi, 1
 mov     rbx, qword, ptr, [rbp, -, 72]
 je      LBB379_77
 TX_CLASS_VERT => nz_map_ctx_offset_1d[row],
 cmp     rax, 32
 jae     LBB379_80
 ctx
 lea     rsi, [rip, +, __ZN5rav1e7context20nz_map_ctx_offset_1d17h4cdde8b7a2f5d5fdE]
 add     rdi, qword, ptr, [rsi, +, 8*rax]
 jmp     LBB379_79
LBB379_77:
 TX_CLASS_HORIZ => nz_map_ctx_offset_1d[col],
 cmp     r13, 31
 ja      LBB379_81
 ctx
 lea     rax, [rip, +, __ZN5rav1e7context20nz_map_ctx_offset_1d17h4cdde8b7a2f5d5fdE]
 add     rdi, qword, ptr, [rax, +, 8*r13]
LBB379_79:
 mov     rax, rdi
 mov     rsi, r10
 mov     rdi, qword, ptr, [rbp, +, 32]
LBB379_74:
 coeff_contexts[pos as usize] = Self::get_nz_map_ctx(
 cmp     r9, r14
 jae     LBB379_83
LBB379_75:
 inc     r15
 mov     byte, ptr, [rdi, +, r9], al
     cmp     rbx, r15
     jne     LBB379_9
     jmp     LBB379_76
LBB379_69:
     cmp     rax, 4
     mov     rbx, qword, ptr, [rbp, -, 72]
     jb      LBB379_71
     mov     eax, 4
LBB379_71:
     mov     rcx, r8
     cmp     r13, 4
     jb      LBB379_73
     mov     r13d, 4
LBB379_73:
 av1_nz_map_ctx_offset[tx_size as usize][cmp::min(row, 4)]
 lea     rax, [rax, +, 4*rax]
 mov     rsi, qword, ptr, [rbp, -, 128]
 lea     rsi, [rsi, +, 4*rsi]
 lea     rsi, [rsi, +, 4*rsi]
 lea     r8, [rip, +, __ZN5rav1e7context21av1_nz_map_ctx_offset17h187907214aca2eedE]
 add     rsi, r8
 add     rsi, rax
 movsx   rax, byte, ptr, [r13, +, rsi]
 ctx
 add     rax, rdi
 mov     rsi, r10
 mov     rdi, qword, ptr, [rbp, +, 32]
 mov     r8, rcx
 mov     rcx, qword, ptr, [rbp, -, 48]
 coeff_contexts[pos as usize] = Self::get_nz_map_ctx(
 cmp     r9, r14
 jb      LBB379_75
 jmp     LBB379_83
LBB379_33:
 movzx   r12d, dl
 mag += cmp::min(3, levels[4]); // { 4, 0 }
 cmp     rax, 4
 jbe     LBB379_53
LBB379_36:
 lea     rax, [rdi, +, 3]
 mov     edx, 4
 if (tx_class as u32 | coeff_idx as u32) == 0 {
 mov     esi, r9d
 or      esi, dword, ptr, [rbp, -, 104]
 if (tx_class as u32 | coeff_idx as u32) == 0 {
 jne     LBB379_57
LBB379_29:
 xor     eax, eax
 mov     r8, qword, ptr, [rbp, -, 96]
 mov     rdx, qword, ptr, [rbp, -, 88]
 mov     rcx, qword, ptr, [rbp, -, 48]
 mov     r12, qword, ptr, [rbp, -, 80]
 mov     rsi, r10
 mov     rdi, qword, ptr, [rbp, +, 32]
 mov     rbx, qword, ptr, [rbp, -, 72]
 coeff_contexts[pos as usize] = Self::get_nz_map_ctx(
 cmp     r9, r14
 jb      LBB379_75
 jmp     LBB379_83
LBB379_40:
 movzx   r12d, dl
 mov     rdx, qword, ptr, [rbp, -, 136]
 mag += cmp::min(3, levels[(4 << bhl) + (4 << TX_PAD_HOR_LOG2)]); // { 0, 4 }
 cmp     rdx, rax
 jae     LBB379_56
LBB379_43:
 mov     rax, qword, ptr, [rbp, -, 56]
 add     rax, rdi
 if (tx_class as u32 | coeff_idx as u32) == 0 {
 mov     esi, r9d
 or      esi, dword, ptr, [rbp, -, 104]
 if (tx_class as u32 | coeff_idx as u32) == 0 {
 jne     LBB379_57
 jmp     LBB379_29
LBB379_76:
 }
 add     rsp, 104
 pop     rbx
 pop     r12
 pop     r13
 pop     r14
 pop     r15
 pop     rbp
 ret
LBB379_82:
 let pos = scan[i as usize];
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.329]
 mov     rsi, r8
 mov     rdx, r8
 call    core::panicking::panic_bounds_check
LBB379_83:
 coeff_contexts[pos as usize] = Self::get_nz_map_ctx(
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.330]
 mov     rsi, r9
 mov     rdx, r14
 call    core::panicking::panic_bounds_check
LBB379_44:
 let mut mag = cmp::min(3, levels[1]); // { 1, 0 }
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.316]
 mov     esi, 1
 mov     rdx, rax
 call    core::panicking::panic_bounds_check
LBB379_45:
 mag += cmp::min(3, levels[(1 << bhl) + TX_PAD_HOR]); // { 0, 1 }
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.317]
 mov     rsi, r12
 mov     rdx, rax
 call    core::panicking::panic_bounds_check
LBB379_84:
     mov     rsi, rdx
     call    core::slice::slice_index_order_fail
LBB379_46:
 mag += cmp::min(3, levels[(1 << bhl) + TX_PAD_HOR + 1]); // { 1, 1 }
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.318]
 mov     rsi, qword, ptr, [rbp, -, 64]
 mov     rdx, rax
 call    core::panicking::panic_bounds_check
LBB379_47:
 mag += cmp::min(3, levels[2]); // { 2, 0 }
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.319]
 mov     esi, 2
 mov     rdx, rax
 call    core::panicking::panic_bounds_check
LBB379_51:
 mag += cmp::min(3, levels[2]); // { 2, 0 }
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.321]
 mov     esi, 2
 mov     rdx, rax
 call    core::panicking::panic_bounds_check
LBB379_54:
 mag += cmp::min(3, levels[(2 << bhl) + (2 << TX_PAD_HOR_LOG2)]); // { 0, 2 }
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.324]
 mov     rdx, rax
 call    core::panicking::panic_bounds_check
LBB379_55:
 mag += cmp::min(3, levels[(3 << bhl) + (3 << TX_PAD_HOR_LOG2)]); // { 0, 3 }
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.325]
 mov     rsi, qword, ptr, [rbp, -, 56]
 mov     rdx, rax
 call    core::panicking::panic_bounds_check
LBB379_49:
 mag += cmp::min(3, levels[(2 << bhl) + (2 << TX_PAD_HOR_LOG2)]); // { 0, 2 }
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.320]
 mov     rsi, rdx
 mov     rdx, rax
 call    core::panicking::panic_bounds_check
LBB379_53:
 mag += cmp::min(3, levels[4]); // { 4, 0 }
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.323]
 mov     esi, 4
 mov     rdx, rax
 call    core::panicking::panic_bounds_check
LBB379_56:
 mag += cmp::min(3, levels[(4 << bhl) + (4 << TX_PAD_HOR_LOG2)]); // { 0, 4 }
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.326]
 mov     rsi, rdx
 mov     rdx, rax
 call    core::panicking::panic_bounds_check
LBB379_52:
 mag += cmp::min(3, levels[3]); // { 3, 0 }
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.322]
 mov     esi, 3
 mov     edx, 3
 call    core::panicking::panic_bounds_check
LBB379_80:
 TX_CLASS_VERT => nz_map_ctx_offset_1d[row],
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.327]
 mov     edx, 32
 mov     rsi, rax
 call    core::panicking::panic_bounds_check
LBB379_81:
 TX_CLASS_HORIZ => nz_map_ctx_offset_1d[col],
 lea     rdi, [rip, +, l_anon.f655925466ffa8fa1a9836a0ae3197d6.328]
 mov     edx, 32
 mov     rsi, r13
 call    core::panicking::panic_bounds_check

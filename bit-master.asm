 fn bit(&mut self, bit: u16) {
 sub     rsp, 4
 self.bool(bit == 1, 16384);
 xor     eax, eax
 cmp     si, 1
 sete    al
 mov     ecx, 16384
 mov     esi, -32768
 self.store(fl, fh, nms as u16);
 cmove   esi, ecx
 self.symbol(if val { 1 } else { 0 }, &[f, 0]);
 mov     dword, ptr, [rsp], 16384
 mov     ecx, 2
 self.store(fl, fh, nms as u16);
 sub     ecx, eax
 movzx   edx, word, ptr, [rsp, +, 2*rax]
 add     rsp, 4
 jmp     _ZN100_$LT$rav1e..ec..WriterBase$LT$rav1e..ec..WriterRecorder$GT$$u20$as$u20$rav1e..ec..StorageBackend$GT$5store17haaea1683eb4e284aE

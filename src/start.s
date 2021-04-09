/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
 .globl _start

.section ".init"

.global t210_reset
.global _start
.global exception_handle
.global test_print_core
.global drop_to_el1
_start:
start:
    b cold_start
    b warm_start

cold_start:
    msr daifset, #0xf

    mrs x0, MPIDR_EL1
    and x1, x0, #0xFF

    cmp x1, #0x0
    bne warm_start

    // Interrupts go to EL2
    mov x0, #0x10
    msr hcr_el2, x0

    // Set up MMU
    adr x0, translation_table
    
    ldr x2, =(0x4FF) // id0 cached, id1 Device-nGnRE
    msr mair_el2, x2
    
    ldr x2, =(1 << 20 | 1 << 13 | 1 << 12 | (1) << 16 | 1 << 10 | 1 << 8 | 0x1F) // TCR: SH0 inner sharable, ORGN0 "Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable.", IRGN0 "Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable."
    msr ttbr0_el2, X0
    msr tcr_el2, X2
    isb
    
    // Set up exception vectors
    adr x0, vector_table_el2
    msr vbar_el2, x0
    isb

    mrs X0, sctlr_el2
    // Read System Control Register 
    // configuration data  
    orr X0, X0, #1 // mmu en
    bic x0, x0, #(1<<1) // aligned access
    orr x0, x0, #(1<<2) // data cache
    orr x0, x0, #(1<<12) // instruction cache
    bic x0, x0, #(1<<19) // no wxn
    msr sctlr_el2, X0
    isb
    
    // FPEN
    mov	x0, #0x33ff
	msr	cptr_el2, x0

    mrs x0, MPIDR_EL1
    and x1, x0, #0xFF

    ldr x0, =__stack_end
    lsl x2, x1, #17
    sub x0, x0, x2
    mov sp, x0
    
    ldr x0, =__bss_start
	ldr x1, =__bss_end
	mov x2, #0x0
	mov x3, #0x0
_bss_clear_loop:
    stp x2, x3, [x0, #0x0]
    add x0, x0, #0x10
    cmp x0, x1
    ble _bss_clear_loop

    msr daifclr, #0xf
    isb

    sub sp, sp, #0x10
    str lr, [sp]

    bl main_cold
    
    ldr lr, [sp]
    add sp, sp, #0x10

    b t210_reset

warm_start:
    msr daifset, #0xf
    mov x8, x0

    // Interrupts go to EL2
    mov x0, #0x10
    msr hcr_el2, x0

    // Set up MMU
    adr x0, translation_table

    ldr x2, =(0x4FF) // id0 cached, id1 Device-nGnRE
    msr mair_el2, x2

    ldr x2, =(1 << 20 | 1 << 13 | 1 << 12 | (1) << 16 | 1 << 10 | 1 << 8 | 0x1F) // TCR: SH0 inner sharable, ORGN0 "Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable.", IRGN0 "Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable."
    msr ttbr0_el2, X0
    msr tcr_el2, X2
    isb

    // Set up exception vectors
    adr x0, vector_table_el2
    msr vbar_el2, x0
    isb

    mrs X0, sctlr_el2
    // Read System Control Register
    // configuration data
    orr X0, X0, #1 // mmu en
    bic x0, x0, #(1<<1) // aligned access
    orr x0, x0, #(1<<2) // data cache
    orr x0, x0, #(1<<12) // instruction cache
    bic x0, x0, #(1<<19) // no wxn
    msr sctlr_el2, X0
    isb

    // FPEN
    mov	x0, #0x33ff
	msr	cptr_el2, x0



    mrs x0, MPIDR_EL1
    and x1, x0, #0xFF

    ldr x0, =__stack_end
    lsl x2, x1, #17
    sub x0, x0, x2
    mov sp, x0

    msr daifclr, #0xf
    isb

    sub sp, sp, #0x10
    str lr, [sp]

    mov x0, x8
    bl main_warm

    ldr lr, [sp]
    add sp, sp, #0x10

    b t210_reset

t210_reset:
    mov x1, #0xf0f
    ldr x0, =0xC3000006
    smc #1

.pool

exception_print:
    // Store context
    sub sp, sp, #0x120

    stp x0, x1, [sp, #0x0]
    stp x2, x3, [sp, #0x10]
    stp x4, x5, [sp, #0x20]
    stp x6, x7, [sp, #0x30]
    stp x8, x9, [sp, #0x40]
    stp x10, x11, [sp, #0x50]
    stp x12, x13, [sp, #0x60]
    stp x14, x15, [sp, #0x70]
    stp x16, x17, [sp, #0x80]
    stp x18, x19, [sp, #0x90]
    stp x20, x21, [sp, #0xA0]
    stp x22, x23, [sp, #0xB0]
    stp x24, x25, [sp, #0xC0]
    stp x26, x27, [sp, #0xD0]
    str x28, [sp, #0xE0]
    str x30, [sp, #0xF0] // 30
    
    add x21, sp, #0x120
    str x21, [sp, #0xE8] // 29
    
    mrs	x21, elr_el2 // pc, 31
    str x21, [sp, #0xF8] // pc, 31
    
    mrs	x21, spsr_el2
    mrs	x22, elr_el2
    stp	x21, x22, [sp, #0x100] // 32,33
    mrs	x22, esr_el2
    str x22, [sp, #0x110] // 34

    sub sp, sp, #0x10
    str lr, [sp]
    
    add x1, sp, #0x10
    mov x0, #0x4
    bl exception_handle
    
    ldr lr, [sp]
    add sp, sp, #0x10
    
    cmp x0, #0x0
    beq t210_reset
    
    //msr daifclr, #0b0010
    //isb
    
    msr elr_el2, x0
    
    ldp	x21, x22, [sp, #0x100] // 32,33
    msr	spsr_el2, x21
    //msr	elr_el2, x22
    
    ldp x0, x1, [sp, #0x0]
    ldp x2, x3, [sp, #0x10]
    ldp x4, x5, [sp, #0x20]
    ldp x6, x7, [sp, #0x30]
    ldp x8, x9, [sp, #0x40]
    ldp x10, x11, [sp, #0x50]
    ldp x12, x13, [sp, #0x60]
    ldp x14, x15, [sp, #0x70]
    ldp x16, x17, [sp, #0x80]
    ldp x18, x19, [sp, #0x90]
    ldp x20, x21, [sp, #0xA0]
    ldp x22, x23, [sp, #0xB0]
    ldp x24, x25, [sp, #0xC0]
    ldp x26, x27, [sp, #0xD0]
    ldr x28, [sp, #0xE0]
    ldr x30, [sp, #0xF0]
    add sp, sp, #0x120
    eret

.pool

irq_print:
    // Store context
    sub sp, sp, #0x120

    stp x0, x1, [sp, #0x0]
    stp x2, x3, [sp, #0x10]
    stp x4, x5, [sp, #0x20]
    stp x6, x7, [sp, #0x30]
    stp x8, x9, [sp, #0x40]
    stp x10, x11, [sp, #0x50]
    stp x12, x13, [sp, #0x60]
    stp x14, x15, [sp, #0x70]
    stp x16, x17, [sp, #0x80]
    stp x18, x19, [sp, #0x90]
    stp x20, x21, [sp, #0xA0]
    stp x22, x23, [sp, #0xB0]
    stp x24, x25, [sp, #0xC0]
    stp x26, x27, [sp, #0xD0]
    str x28, [sp, #0xE0]
    str x30, [sp, #0xF0] // 30
    
    add x21, sp, #0x120
    str x21, [sp, #0xE8] // 29
    
    mrs	x21, elr_el2 // pc, 31
    str x21, [sp, #0xF8] // pc, 31
    
    mrs	x21, spsr_el2
    mrs	x22, elr_el2
    stp	x21, x22, [sp, #0x100] // 32,33
    mrs	x22, esr_el2
    str x22, [sp, #0x110] // 34

    sub sp, sp, #0x10
    str lr, [sp]
    
    add x1, sp, #0x10
    mov x0, #0x4
    bl irq_handle
    
    ldr lr, [sp]
    add sp, sp, #0x10
    
    cmp x0, #0x0
    beq t210_reset
    
    //msr daifclr, #0b0010
    //isb

    msr elr_el2, x0
    
    ldp	x21, x22, [sp, #0x100] // 32,33
    msr	spsr_el2, x21
    //msr	elr_el2, x22

    ldp x0, x1, [sp, #0x0]
    ldp x2, x3, [sp, #0x10]
    ldp x4, x5, [sp, #0x20]
    ldp x6, x7, [sp, #0x30]
    ldp x8, x9, [sp, #0x40]
    ldp x10, x11, [sp, #0x50]
    ldp x12, x13, [sp, #0x60]
    ldp x14, x15, [sp, #0x70]
    ldp x16, x17, [sp, #0x80]
    ldp x18, x19, [sp, #0x90]
    ldp x20, x21, [sp, #0xA0]
    ldp x22, x23, [sp, #0xB0]
    ldp x24, x25, [sp, #0xC0]
    ldp x26, x27, [sp, #0xD0]
    ldr x28, [sp, #0xE0]
    ldr x30, [sp, #0xF0]
    add sp, sp, #0x120
    eret

.pool

.balign 0x1000
vector_table_el2:
curr_el_sp0_sync:        // The exception handler for a synchronous 
                         // exception from the current EL using SP0.
clrex
nop
b exception_print

.balign 0x80
curr_el_sp0_irq:         // The exception handler for an IRQ exception
                         // from the current EL using SP0.
clrex
nop
b irq_print

.balign 0x80
curr_el_sp0_fiq:         // The exception handler for an FIQ exception
                         // from the current EL using SP0.
clrex
nop
b irq_print

.balign 0x80
curr_el_sp0_serror:      // The exception handler for a System Error 
                         // exception from the current EL using SP0.
clrex
nop
b exception_print

.balign 0x80
curr_el_spx_sync:        // The exception handler for a synchrous 
                         // exception from the current EL using the
                         // current SP.
clrex
nop
b exception_print


.balign 0x80
curr_el_spx_irq:         // The exception handler for an IRQ exception from 
                         // the current EL using the current SP.
clrex
nop
b irq_print

.balign 0x80
curr_el_spx_fiq:         // The exception handler for an FIQ from 
                         // the current EL using the current SP.
clrex
nop
b irq_print

.balign 0x80
curr_el_spx_serror:      // The exception handler for a System Error 
                         // exception from the current EL using the
                         // current SP.
clrex
nop
b exception_print

 .balign 0x80
lower_el_aarch64_sync:   // The exception handler for a synchronous 
                         // exception from a lower EL (AArch64).
clrex
nop
b exception_print

.balign 0x80
lower_el_aarch64_irq:    // The exception handler for an IRQ from a lower EL
                         // (AArch64).
clrex
nop
b irq_print

.balign 0x80
lower_el_aarch64_fiq:    // The exception handler for an FIQ from a lower EL
                         // (AArch64).
clrex
nop
b irq_print

.balign 0x80
lower_el_aarch64_serror: // The exception handler for a System Error 
                         // exception from a lower EL(AArch64).
clrex
nop
b exception_print

.balign 0x80
lower_el_aarch32_sync:   // The exception handler for a synchronous 
                         // exception from a lower EL(AArch32).
clrex
nop
b exception_print


.balign 0x80
lower_el_aarch32_irq:    // The exception handler for an IRQ exception 
                         // from a lower EL (AArch32).
clrex
nop
b irq_print


.balign 0x80
lower_el_aarch32_fiq:    // The exception handler for an FIQ exception from 
                         // a lower EL (AArch32).
clrex
nop
b irq_print
                         
.balign 0x80
lower_el_aarch32_serror: // The exception handler for a System Error
                         // exception from a lower EL(AArch32).
clrex
nop
b exception_print

.pool

.balign 0x1000
translation_table:
.dword (0x00000000000725 | 0x000000000)
.dword (0x00000000000725 | 0x040000000)
.dword (0x00000000000721 | 0x080000000)
.dword (0x00000000000721 | 0x0C0000000)
.dword (0x00000000000721 | 0x100000000)
.dword (0x00000000000721 | 0x140000000)
.dword (0x00000000000721 | 0x180000000)
.dword (0x00000000000721 | 0x1C0000000)

.dword (0x00000000000721 | 0x200000000)
.dword (0x00000000000721 | 0x240000000)
.dword (0x00000000000721 | 0x280000000)
.dword (0x00000000000721 | 0x2C0000000)
.dword (0x00000000000721 | 0x300000000)
.dword (0x00000000000721 | 0x340000000)
.dword (0x00000000000721 | 0x380000000)
.dword (0x00000000000721 | 0x3C0000000)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)
.dword (0x0)

.balign 0x1000


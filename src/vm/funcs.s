/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of therainsdowninafrica and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

.section ".text"

.global smc1
smc1:
    smc #1
    ret
    
.global smc1_shim
smc1_shim:
    sub sp, sp, #0x50
    stp x0, x1, [sp, #0x0]
    stp x2, x3, [sp, #0x10]
    stp x4, x5, [sp, #0x20]
    stp x6, x7, [sp, #0x30]
    stp x8, x9, [sp, #0x40]
    
    mov x8, x0
    ldp x0, x1, [x8, #0x0]
    ldp x2, x3, [x8, #0x10]
    ldp x4, x5, [x8, #0x20]
    ldp x6, x7, [x8, #0x30]
    
    smc #1
    nop
    
    stp x0, x1, [x8, #0x0]
    stp x2, x3, [x8, #0x10]
    stp x4, x5, [x8, #0x20]
    stp x6, x7, [x8, #0x30]
    
    ldp x8, x9, [sp, #0x40]
    ldp x6, x7, [sp, #0x30]
    ldp x4, x5, [sp, #0x20]
    ldp x2, x3, [sp, #0x10]
    ldp x0, x1, [sp, #0x0]
    add sp, sp,  #0x50

    ret
    
.global smc0_shim
smc0_shim:
    sub sp, sp, #0x50
    stp x0, x1, [sp, #0x0]
    stp x2, x3, [sp, #0x10]
    stp x4, x5, [sp, #0x20]
    stp x6, x7, [sp, #0x30]
    stp x8, x9, [sp, #0x40]
    
    mov x8, x0
    ldp x0, x1, [x8, #0x0]
    ldp x2, x3, [x8, #0x10]
    ldp x4, x5, [x8, #0x20]
    ldp x6, x7, [x8, #0x30]
    
    smc #0
    nop
    
    stp x0, x1, [x8, #0x0]
    stp x2, x3, [x8, #0x10]
    stp x4, x5, [x8, #0x20]
    stp x6, x7, [x8, #0x30]
    
    ldp x8, x9, [sp, #0x40]
    ldp x6, x7, [sp, #0x30]
    ldp x4, x5, [sp, #0x20]
    ldp x2, x3, [sp, #0x10]
    ldp x0, x1, [sp, #0x0]
    add sp, sp,  #0x50

    ret
    
.global drop_to_el1
drop_to_el1:
    msr     elr_el2, x0
    //msr     sp_el1, x1
    
    //msr daifset, #8 // don't debug current EL
    
    ldr     x2, =(0x3c5)     // EL1_SP1 | D | A | I | F | SS =  | (1<<21)
    msr     spsr_el2, x2
    
    mov x0, x1

    eret

.global _enable_single_step
_enable_single_step:
    ldr x3, =(0)
    msr OSLAR_EL1, x3 // unlock
    
    ldr x3, =(1 << 8) // route debug exceptions to EL2
    msr MDCR_EL2, x3
    
    mrs x2, spsr_el2
    ldr     x3, =(1<<21)     // SS
    orr x2, x2, x3
    msr     spsr_el2, x2

    msr daifset, #8 // don't debug current EL
    ldr x3, =(1<<13 | 1) // single-step, kernel debug en
    msr MDSCR_EL1, x3
    isb
    ret

.global _disable_single_step
_disable_single_step:
    ldr x3, =(0)
    msr OSLAR_EL1, x3 // unlock
    
    ldr x3, =(1 << 8) // route debug exceptions to EL2
    msr MDCR_EL2, x3
    
    mrs x2, spsr_el2
    ldr     x3, =(1<<21)     // SS
    orr x2, x2, x3
    msr     spsr_el2, x2

    msr daifset, #8 // don't debug current EL
    ldr x3, =(1<<13 | 0) // single-step, kernel debug en
    msr MDSCR_EL1, x3
    isb
    ret

.global vttbr_apply
vttbr_apply:
    ldr x2, =(0b001 << 16  | 0 << 14 | (1) << 6 | 0x1F) // TCR: 4GiB address size, 4KiB granule, SH0 inner sharable, ORGN0 "Normal memory, Outer Write-Back Read-Allocate Write-Allocate Cacheable.", IRGN0 "Normal memory, Inner Write-Back Read-Allocate Write-Allocate Cacheable."
    ldr x3, =(1 << 31 /*| 1 << 30 */| 1 << 26 /*| 1 << 25/* | 1 << 21*/ | 1 << 19 /*| 1 << 14 | 1 << 13*/ | 0 << 10 | 0 << 5 | 0 << 4 | 0 << 3 | 0<<1 | 1 << 0) // TRVM, TGE, TVM, TTLB, TPU, TPC, TSW, TACR, TIDCP,
    msr vttbr_el2, x0
    msr vtcr_el2, x2
    msr hcr_el2, x3

    isb
    ret

.global no_hyp_stuff
no_hyp_stuff:
    ldr x3, =(1 << 31 /*| 1 << 30 */| 0 << 26 /*| 1 << 25/* | 1 << 21*/ | 0 << 19 /*| 1 << 14 | 1 << 13*/ | 0 << 10 | 0 << 5 | 0 << 4 | 0 << 3 | 0<<1 | 0 << 0) // TRVM, TGE, TVM, TTLB, TPU, TPC, TSW, TACR, TIDCP,
    msr hcr_el2, x3

    isb
    ret

.global disable_smcstuff
disable_smcstuff:
    ldr x3, =(1 << 31 /*| 1 << 30 */| 1 << 26 /*| 1 << 25/* | 1 << 21 | 1 << 19 | 1 << 14 | 1 << 13*/ | 0 << 10 /*| 1 << 5 | 1 << 4 | 1 << 3*/ | 1 << 0) // TRVM, TGE, TVM, TTLB, TPU, TPC, TSW, TACR, TIDCP, 
    msr hcr_el2, x3

    isb
    ret
    
.pool

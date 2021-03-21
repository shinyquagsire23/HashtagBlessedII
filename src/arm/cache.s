/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

.section ".text"

.global _dcache_flush_invalidate
_dcache_flush_invalidate:
	add x1, x1, x0
	mrs x8, CTR_EL0
	lsr x8, x8, #16
	and x8, x8, #0xf
	mov x9, #4
	lsl x9, x9, x8
	sub x10, x9, #1
	bic x8, x0, x10
	mov x10, x1

dcache_flush_invalidate_L0:
	dc  civac, x8
	add x8, x8, x9
	cmp x8, x10
	bcc dcache_flush_L0

	dsb sy
	ret

.global _dcache_invalidate
_dcache_invalidate:
	add x1, x1, x0
	mrs x8, CTR_EL0
	lsr x8, x8, #16
	and x8, x8, #0xf
	mov x9, #4
	lsl x9, x9, x8
	sub x10, x9, #1
	bic x8, x0, x10
	mov x10, x1

dcache_invalidate_L0:
	dc  ivac, x8
	add x8, x8, x9
	cmp x8, x10
	bcc dcache_invalidate_L0

	dsb sy
	ret

.global _icache_invalidate
_icache_invalidate:
	add x1, x1, x0
	mrs x8, CTR_EL0
	lsr x8, x8, #16
	and x8, x8, #0xf
	mov x9, #4
	lsl x9, x9, x8
	sub x10, x9, #1
	bic x8, x0, x10
	mov x10, x1

icache_invalidate_L0:
	ic  ivau, x8
	add x8, x8, x9
	cmp x8, x10
	bcc icache_invalidate_L0

	dsb sy
	ret

.global _dcache_flush
_dcache_flush:
	add x1, x1, x0
	mrs x8, CTR_EL0
	lsr x8, x8, #16
	and x8, x8, #0xf
	mov x9, #4
	lsl x9, x9, x8
	sub x10, x9, #1
	bic x8, x0, x10
	mov x10, x1

dcache_flush_L0:
	dc  cvac, x8
	add x8, x8, x9
	cmp x8, x10
	bcc dcache_invalidate_L0

	dsb sy
	ret

.global _dcache_zero
_dcache_zero:
	add x1, x1, x0
	mrs x8, CTR_EL0
	lsr x8, x8, #16
	and x8, x8, #0xf
	mov x9, #4
	lsl x9, x9, x8
	sub x10, x9, #1
	bic x8, x0, x10
	mov x10, x1

dcache_zero_L0:
	dc  zva, x8
	add x8, x8, x9
	cmp x8, x10
	bcc _dcache_zero

	dsb sy
	ret

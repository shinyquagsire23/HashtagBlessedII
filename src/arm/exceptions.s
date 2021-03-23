/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

.section ".text"

.global _get_hpfar_el2
_get_hpfar_el2:
    mrs x0, hpfar_el2
    ret

.global _get_far_el2
_get_far_el2:
    mrs x0, far_el2
    ret

.global _get_elr_el2
_get_elr_el2:
    mrs x0, elr_el2
    ret

.global _get_esr_el2
_get_esr_el2:
	mrs x0, esr_el2
	ret

.global _get_afsr0_el2
_get_afsr0_el2:
	mrs x0, afsr0_el2
	ret

.global _get_afsr1_el2
_get_afsr1_el2:
	mrs x0, afsr1_el2
	ret

.global _get_sp_el2
_get_sp_el2:
	mrs x0, sp_el2
	ret

.global _get_elr_el1
_get_elr_el1:
    mrs x0, elr_el1
    ret

.global _get_esr_el1
_get_esr_el1:
	mrs x0, esr_el1
	ret

.global _get_afsr0_el1
_get_afsr0_el1:
	mrs x0, afsr0_el1
	ret

.global _get_afsr1_el1
_get_afsr1_el1:
	mrs x0, afsr1_el1
	ret

.global _get_sp_el1
_get_sp_el1:
	mrs x0, sp_el1
	ret

.global _get_sp_el0
_get_sp_el0:
	mrs x0, sp_el0
	ret

.global _get_spsr_el2
__get_spsr_el2:
	mrs x0, spsr_el2
	ret
	
.global _get_spsr_el1
_get_spsr_el1:
	mrs x0, spsr_el1
	ret

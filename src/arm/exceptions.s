/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

.section ".text"

.global get_hpfar_el2
get_hpfar_el2:
    mrs x0, hpfar_el2
    ret

.global get_far_el2
get_far_el2:
    mrs x0, far_el2
    ret

.global get_elr_el2
get_elr_el2:
    mrs x0, elr_el2
    ret

.global get_esr_el2
get_esr_el2:
	mrs x0, esr_el2
	ret

.global get_afsr0_el2
get_afsr0_el2:
	mrs x0, afsr0_el2
	ret

.global get_afsr1_el2
get_afsr1_el2:
	mrs x0, afsr1_el2
	ret

.global get_sp_el2
get_sp_el2:
	mrs x0, sp_el2
	ret

.global get_elr_el1
get_elr_el1:
    mrs x0, elr_el1
    ret

.global get_esr_el1
get_esr_el1:
	mrs x0, esr_el1
	ret

.global get_afsr0_el1
get_afsr0_el1:
	mrs x0, afsr0_el1
	ret

.global get_afsr1_el1
get_afsr1_el1:
	mrs x0, afsr1_el1
	ret

.global get_sp_el1
get_sp_el1:
	mrs x0, sp_el1
	ret

.global get_sp_el0
get_sp_el0:
	mrs x0, sp_el0
	ret


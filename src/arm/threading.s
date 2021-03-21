/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

.section ".text"

.global _get_core
_get_core:
    mrs x0, mpidr_el1
    and x0, x0, #0xff
    ret

.global get_core2
get_core2:
    mrs x0, vmpidr_el2
    and x0, x0, #0xff
    ret


.global get_mpidr
get_mpidr:
    mrs x0, mpidr_el1
    ret

.global get_vmpidr
get_vmpidr:
    mrs x0, vmpidr_el2
    ret

.global getSP_EL0
getSP_EL0:
    mrs x0, sp_el0
    ret

.pool

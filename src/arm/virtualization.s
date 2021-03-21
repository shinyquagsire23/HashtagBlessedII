/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

.section ".text"

.global _get_cnthctl_el2
_get_cnthctl_el2:
    mrs x0, cnthctl_el2
    ret

.global _set_cnthctl_el2
_set_cnthctl_el2:
    msr cnthctl_el2, x0
    ret


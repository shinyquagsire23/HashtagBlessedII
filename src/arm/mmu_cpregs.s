/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

.section ".text"

.global getTTB1
get_ttb1:
    mrs x0, ttbr1_el1
    ret


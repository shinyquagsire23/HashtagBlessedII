/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

.section ".text"

.global enable_fp
enable_fp:
	mrs x0, cpacr_el1
	orr x0, x0, #(3 << 20)
	msr cpacr_el1, x0
	ret

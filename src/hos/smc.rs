/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

// TODO enum?
pub const SMC_CPUSUSPEND:        u64 = 0x1C4000001;
pub const SMC_CPUOFF:            u64 = 0x184000002;
pub const SMC_CPUON:             u64 = 0x1C4000003;
pub const SMC_GETCONFIG:         u64 = 0x1C3000004;
pub const SMC_GENRANDOMBYTES:    u64 = 0x1C3000005;
pub const SMC_PANIC:             u64 = 0x1C3000006;
pub const SMC_CONFIGURECARVEOUT: u64 = 0x1C3000007;
pub const SMC_RWREGISTER:        u64 = 0x1C3000008;

// TODO enum?
pub const SMC0_SETCONFIG:             u32 = 0xC3000401;
pub const SMC0_GETCONFIG:             u32 = 0xC3000002;
pub const SMC0_GETRESULT:             u32 = 0xC3000003;
pub const SMC0_GETRESULTDATA:         u32 = 0xC3000404;
pub const SMC0_EXPMOD:                u32 = 0xC3000E05;
pub const SMC0_GENRANDOMBYTES:        u32 = 0xC3000006;
pub const SMC0_GENAESKEK:             u32 = 0xC3000007;
pub const SMC0_LOADAESKEY:            u32 = 0xC3000008;
pub const SMC0_COMPUTEAES:            u32 = 0xC3000009;
pub const SMC0_GENSPECIFICAESKEY:     u32 = 0xC300000A;
pub const SMC0_COMPUTECMAC:           u32 = 0xC300040B;
pub const SMC0_IMPORTESKEY:           u32 = 0xC300100C;
pub const SMC0_RECRYPTRSAPRIVKEY:     u32 = 0xC300D60C;
pub const SMC0_DECRYPTRSAPRIVKEY:     u32 = 0xC300100D;
pub const SMC0_IMPORTLOTUSKEY:        u32 = 0xC300100E;
pub const SMC0_STORAGEEXPMOD:         u32 = 0xC300060F;
pub const SMC0_UNWRAPTITLEKEY:        u32 = 0xC3000610;
pub const SMC0_LOADTITLEKEY:          u32 = 0xC3000011;
pub const SMC0_UNWRAPCOMMONTITLEKEYE: u32 = 0xC3000012;

// TODO enum
pub const CONFIGITEM_PROGRAMVERIFY:      u32 = (1);
pub const CONFIGITEM_DRAMID:             u32 = (2);
pub const CONFIGITEM_SEINT:              u32 = (3);
pub const CONFIGITEM_FUSEVER:            u32 = (4);
pub const CONFIGITEM_HWTYPE:             u32 = (5);
pub const CONFIGITEM_ISRETAIL:           u32 = (6);
pub const CONFIGITEM_ISRECOVERYBOOT:     u32 = (7);
pub const CONFIGITEM_DEVICEID:           u32 = (8);
pub const CONFIGITEM_BOOTREASON:         u32 = (9);
pub const CONFIGITEM_MEMORYMODE:         u32 = (10);
pub const CONFIGITEM_ISDEBUGMODE:        u32 = (11);
pub const CONFIGITEM_KERNELCONFIG:       u32 = (12);
pub const CONFIGITEM_CHARGERHIZ:         u32 = (13);
pub const CONFIGITEM_ISQUEST:            u32 = (14);
pub const CONFIGITEM_REGULATORTYPE:      u32 = (15);
pub const CONFIGITEM_DEVICEUNIQUEKEYGEN: u32 = (16);
pub const CONFIGITEM_PK2HASH:            u32 = (17);

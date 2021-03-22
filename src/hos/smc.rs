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
pub const SMC0_SETCONFIG:             u64 = 0xC3000401;
pub const SMC0_GETCONFIG:             u64 = 0xC3000002;
pub const SMC0_GETRESULT:             u64 = 0xC3000003;
pub const SMC0_GETRESULTDATA:         u64 = 0xC3000404;
pub const SMC0_EXPMOD:                u64 = 0xC3000E05;
pub const SMC0_GENRANDOMBYTES:        u64 = 0xC3000006;
pub const SMC0_GENAESKEK:             u64 = 0xC3000007;
pub const SMC0_LOADAESKEY:            u64 = 0xC3000008;
pub const SMC0_COMPUTEAES:            u64 = 0xC3000009;
pub const SMC0_GENSPECIFICAESKEY:     u64 = 0xC300000A;
pub const SMC0_COMPUTECMAC:           u64 = 0xC300040B;
pub const SMC0_IMPORTESKEY:           u64 = 0xC300100C;
pub const SMC0_RECRYPTRSAPRIVKEY:     u64 = 0xC300D60C;
pub const SMC0_DECRYPTRSAPRIVKEY:     u64 = 0xC300100D;
pub const SMC0_IMPORTLOTUSKEY:        u64 = 0xC300100E;
pub const SMC0_STORAGEEXPMOD:         u64 = 0xC300060F;
pub const SMC0_UNWRAPTITLEKEY:        u64 = 0xC3000610;
pub const SMC0_LOADTITLEKEY:          u64 = 0xC3000011;
pub const SMC0_UNWRAPCOMMONTITLEKEYE: u64 = 0xC3000012;

// TODO enum
pub const CONFIGITEM_PROGRAMVERIFY:      u64 = (1);
pub const CONFIGITEM_DRAMID:             u64 = (2);
pub const CONFIGITEM_SEINT:              u64 = (3);
pub const CONFIGITEM_FUSEVER:            u64 = (4);
pub const CONFIGITEM_HWTYPE:             u64 = (5);
pub const CONFIGITEM_ISRETAIL:           u64 = (6);
pub const CONFIGITEM_ISRECOVERYBOOT:     u64 = (7);
pub const CONFIGITEM_DEVICEID:           u64 = (8);
pub const CONFIGITEM_BOOTREASON:         u64 = (9);
pub const CONFIGITEM_MEMORYMODE:         u64 = (10);
pub const CONFIGITEM_ISDEBUGMODE:        u64 = (11);
pub const CONFIGITEM_KERNELCONFIG:       u64 = (12);
pub const CONFIGITEM_CHARGERHIZ:         u64 = (13);
pub const CONFIGITEM_ISQUEST:            u64 = (14);
pub const CONFIGITEM_REGULATORTYPE:      u64 = (15);
pub const CONFIGITEM_DEVICEUNIQUEKEYGEN: u64 = (16);
pub const CONFIGITEM_PK2HASH:            u64 = (17);

pub const fn get_smc_name(smc_cmd: u64) -> &'static str
{
    match (smc_cmd)
    {
        SMC_CPUSUSPEND => "CpuSuspend",
        SMC_CPUOFF => "CpuOff",
        SMC_CPUON => "CpuOn",
        SMC_GETCONFIG => "GetConfig",
        SMC_GENRANDOMBYTES => "GenerateRandomBytes",
        SMC_PANIC => "Panic",
        SMC_CONFIGURECARVEOUT => "ConfigureCarveout",
        SMC_RWREGISTER => "ReadWriteRegister",
        SMC0_SETCONFIG => "SetConfig",
        SMC0_GETCONFIG => "GetConfig",
        SMC0_GETRESULT => "GetResult",
        SMC0_GETRESULTDATA => "GetResultData",
        SMC0_EXPMOD => "ExpMod",
        SMC0_GENRANDOMBYTES => "GenerateRandomBytes",
        SMC0_GENAESKEK => "GenerateAesKek",
        SMC0_LOADAESKEY => "LoadAesKey",
        SMC0_COMPUTEAES => "ComputeAes",
        SMC0_GENSPECIFICAESKEY => "GenerateSpecificAesKey",
        SMC0_COMPUTECMAC => "ComputeCmac",
        SMC0_IMPORTESKEY => "ImportEsKey",
        SMC0_RECRYPTRSAPRIVKEY => "ReEncryptRsaPrivateKey",
        SMC0_DECRYPTRSAPRIVKEY => "DecryptOrImportRsaPrivateKey",
        SMC0_IMPORTLOTUSKEY => "ImportLotusKey",
        SMC0_STORAGEEXPMOD => "StorageExpMod",
        SMC0_UNWRAPTITLEKEY => "UnwrapTitleKey",
        SMC0_LOADTITLEKEY => "LoadTitleKey",
        SMC0_UNWRAPCOMMONTITLEKEYE => "UnwrapCommonTitleKey",
        _ => "Unknown",
    }
}

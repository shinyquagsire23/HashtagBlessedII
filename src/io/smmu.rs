/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::util::*;
use crate::arm::threading::*;
use crate::arm::cache::*;
use crate::vm::vmmu::*;
use crate::logger::*;
use alloc::collections::BTreeMap;
use crate::hos::smc::*;
use crate::util::*;

pub const AHB_BASE: u32 = 0x6000C000;

pub const AHB_ARBITRATION_DISABLE: u32 = (AHB_BASE + 0x004);

pub const MC_BASE: u64 = (0x70019000);
pub const MC_END: u64 = (0x7001A000);

pub const MC_ERR_STATUS: u64 = (MC_BASE + 0x8);
pub const MC_ERR_ADR: u64 = (MC_BASE + 0xC);
pub const MC_SMMU_CONFIG: u64 = (MC_BASE + 0x10);
pub const MC_SMMU_TLB_CONFIG: u64 = (MC_BASE + 0x14);
pub const MC_SMMU_PTC_CONFIG: u64 = (MC_BASE + 0x18);
pub const MC_SMMU_PTB_ASID: u64 = (MC_BASE + 0x1C);
pub const MC_SMMU_PTB_DATA: u64 = (MC_BASE + 0x20);
pub const MC_SMMU_TLB_FLUSH: u64 = (MC_BASE + 0x30);
pub const MC_SMMU_PTC_FLUSH: u64 = (MC_BASE + 0x34);
pub const MC_EMEM_CFG: u64 = (MC_BASE + 0x50);
pub const MC_EMEM_ADR_CFG: u64 = (MC_BASE + 0x54);
pub const MC_SMMU_PPCS1_ASID: u64 = (MC_BASE + 0x298);
pub const MC_SMMU_DC_ASID: u64 = (MC_BASE + 0x240); // Display0A/0B/0C
pub const MC_SMMU_DCB_ASID: u64 = (MC_BASE + 0x244);
pub const MC_SMMU_PTC_FLUSH_1: u64 = (MC_BASE + 0x9B8);
pub const MC_SMMU_SDMMC1A_ASID: u64 = (MC_BASE + 0xA94);
pub const MC_SMMU_SDMMC2A_ASID: u64 = (MC_BASE + 0xA98);
pub const MC_SMMU_SDMMC3A_ASID: u64 = (MC_BASE + 0xA9C);
pub const MC_SMMU_SDMMC4A_ASID: u64 = (MC_BASE + 0xAA0);

pub const SMMU_NUM_PAGES: usize = 0x400;

static mut LAST_MC_SMMU_PTC_FLUSH_HI: u32 = 0;

static mut SE_BUFFER: u64 = 0;
static mut SE_BUFFER_ADJ: u32 = 0;

static mut SDMMC_ASID: u32 = 6;
static mut SDMMC_BUFFER: u64 = 0;
static mut SDMMC_BUFFER_ADJ: u32 = 0;

static mut DC_ASID: u32 = 7;
static mut DC_BUFFER: u64 = 0;
static mut DC_BUFFER_ADJ: u32 = 0;

static mut SMMU_CURRENT_ASID: u8 = 0;
static mut SMMU_PAGE_MAPPINGS: BTreeMap<u64, u64> = BTreeMap::new();

static mut PTB_HOS_ASIDS: [u64; 0x80] = [0; 0x80];
static mut PTB_HTB_ASIDS: [u64; 0x80] = [0; 0x80];
static mut ASID_BUFFERS: [u64; 0x80] = [0; 0x80];
static mut ASID_BASES: [u32; 0x80] = [0; 0x80];
static mut SMMU_PAGE_ALLOCBITMAP: [u8; SMMU_NUM_PAGES/8] = [0; SMMU_NUM_PAGES/8];

#[repr(align(0x1000))]
struct SMMUPages([u32; 1024 * SMMU_NUM_PAGES]);

static mut SMMU_PAGES: SMMUPages = SMMUPages([0; 1024 * SMMU_NUM_PAGES]);

pub fn smmu_init()
{
    let ahb_arb_disable: MMIOReg = MMIOReg::new(AHB_ARBITRATION_DISABLE);
    
    /*unsafe
    {
    SMMU_PAGES = SMMUPages([0; 1024 * SMMU_NUM_PAGES]);
    SMMU_PAGE_ALLOCBITMAP = [0; SMMU_NUM_PAGES/8];
    SMMU_CURRENT_ASID = 0;
    PTB_HOS_ASIDS = [0; 0x80];
    PTB_HTB_ASIDS = [0; 0x80];
    ASID_BASES = [0; 0x80];
    ASID_BUFFERS = [0; 0x80];
    SMMU_PAGE_MAPPINGS = BTreeMap::new();
    }*/
    
    // TODO actual init
    
    // Allow usbd regs to be arbitrated
    // (SMMU will still be locked out but there's a workaround)
    ahb_arb_disable.w32(0);
}

pub fn smmu_print_err()
{
    let status = smmu_readreg(MC_ERR_STATUS);
    let addr = smmu_readreg(MC_ERR_ADR);
    smmu_writereg(MC_BASE, smmu_readreg(MC_BASE));
    smmu_writereg(MC_BASE, 0);
    smmu_writereg(MC_BASE+4, 0);

    let err_id = status & 0xFF;
    let err_adr1 = (status >> 12) & 7;
    let err_rw = (status & bit!(16)) != 0;
    let err_security = (status & bit!(17)) != 0;
    let err_swap = (status & bit!(18)) != 0;
    let err_adr_hi = (status >> 20) & 3;
    let err_invalid_smmu_page_nonsecure = (status & bit!(25)) != 0;
    let err_invalid_smmu_page_writable = (status & bit!(26)) != 0;
    let err_invalid_smmu_page_readable = (status & bit!(27)) != 0;
    let err_type = (status >> 28) & 7;
    
    if err_type == 7 {
        return;
    }
    
    println!("({:08x}, {:08x}) ERR_ID {:x} ERR_ADR1 {:x} ERR_RW {} ERR_SECURITY {} ERR_SWAP {}", status, addr, err_id, err_adr1, err_rw, err_security, err_swap);
    println!("ERR_ADR_HI {:x} INVALID_SEC {} INVALID_WRITE {} INVALID_READ {} ERR_TYPE {:x}", err_adr_hi, err_invalid_smmu_page_nonsecure, err_invalid_smmu_page_writable, err_invalid_smmu_page_readable, err_type);
}

pub fn smmu_test()
{
    /*let emem_cfg = smmu_readreg(MC_EMEM_CFG);
    
    let page_addr = 0xd0000000 as u64;
    let ptb_data_val = (((page_addr & 0x3FFFFFFFF)>>12) | bit!(29) | bit!(30) | bit!(31)) as u32;
    
    //smmu_writereg(MC_EMEM_CFG, emem_cfg & !bit!(31));
    smmu_writereg(MC_SMMU_PTB_ASID, 0x06);
    smmu_writereg(MC_SMMU_PTB_DATA, ptb_data_val);
    smmu_writereg(MC_SMMU_PTC_FLUSH, 0x00);
    smmu_readreg(MC_SMMU_TLB_CONFIG); // flush
    smmu_writereg(MC_SMMU_TLB_FLUSH, 0x00);
    smmu_readreg(MC_SMMU_TLB_CONFIG); // flush
    //smmu_writereg(MC_EMEM_CFG, emem_cfg);
    
    smmu_print_err();*/
}

pub fn smmu_get_se_buffer() -> u64
{
    unsafe { return SE_BUFFER; }
}

pub fn smmu_get_se_buffer_adj() -> u32
{
    unsafe { return SE_BUFFER_ADJ; }
}

pub fn smmu_get_sdmmc_buffer() -> u64
{
    unsafe { return SDMMC_BUFFER; }//ASID_BUFFERS[SDMMC_ASID];
}

pub fn smmu_get_sdmmc_buffer_adj() -> u32
{
    unsafe { return SDMMC_BUFFER_ADJ; }//ASID_BASES[SDMMC_ASID];
}

pub fn smmu_map_pages(hos: u64, hyp: u64)
{
    unsafe
    {
        SMMU_PAGE_MAPPINGS.insert(hos, hyp);
    }
}

pub fn smmu_unmap_page(hyp: u64)
{
    let mut hos: u64 = 0;
    unsafe
    {
        for (key, val) in &SMMU_PAGE_MAPPINGS
        {
            if *val == hyp
            {
                hos = *key;
                break;
            }
        }
        SMMU_PAGE_MAPPINGS.remove(&hos);
    }
}

pub fn smmu_find_hyp_mapping_from_hos(hos: u64) -> u64
{
    unsafe
    {
        if let Some(hyp) = SMMU_PAGE_MAPPINGS.get(&hos) {
            return *hyp;
        }
    }
    return 0;
}

pub fn smmu_find_hos_mapping_from_hyp(hyp: u64) -> u64
{
    let mut hos: u64 = 0;
    unsafe
    {
        for (key, val) in &SMMU_PAGE_MAPPINGS
        {
            if *val == hyp
            {
                hos = *key;
                break;
            }
        }
        return hos;
    }
    return 0;
}

pub fn smmu_freetable(smmu_tlb: u64, level: i32)
{
    for i in 0..(0x1000/4)
    {
        let curaddr = smmu_tlb + (i*4) as u64;
        let tblval = peek32(curaddr);
        if (tblval == 0) {
            continue;
        }

        //printf("freeing @ lv{} (asid {:02x}): {:016x}: {:08x}\n\r", level, SMMU_CURRENT_ASID, curaddr, tblval);
        
        let smmu_pa = ((tblval & 0x3fffff) as u64) << 12;

        if ((tblval & 0x10000000) != 0) // page table
        {
            //printf("freeing @ lv{} (asid {:02x}): lv{} page table {:016x}\n\r", level, SMMU_CURRENT_ASID, level+1, smmu_pa);
            smmu_freetable(smmu_pa, level + 1);
            smmu_freepage(smmu_pa);
        }
        poke32(curaddr, 0);
    }
}

pub fn smmu_translatetlb(smmu_tlb: u64, baseaddr: u32, level: i32)
{
    unsafe
    {
        for i in 0..(0x1000/4)
        {
            let curaddr = smmu_tlb + i*4;
            let deviceaddr = baseaddr + (i * (if level == 0 { 0x400000 } else { 0x1000 })) as u32;
            let tblval = peek32(curaddr);
            if (tblval == 0) {
                continue;
            }
            
            let smmu_ipa = ((tblval & 0x3fffff) as u64) << 12;
            let smmu_pa = ipaddr_to_paddr(smmu_ipa);
            
            if smmu_pa == 0 {
                println_core!("!! SMMU is mapping unavailable page {:x} !!", smmu_ipa);
            }
            
            //if (level == 0)
            //    printf("translating @ lv{} (asid {:02x}): {:016x}: {:08x} (ipa {:08x} pa {:08x} va {:08x})\n\r", level, SMMU_CURRENT_ASID, curaddr, tblval, smmu_ipa, smmu_pa, deviceaddr);
            
            if ((tblval & 0x10000000) != 0) // page table
            {
                let newpage = smmu_allocpage();
                if (newpage == 0)
                {
                    panic!("COULDN'T ALLOC SMMU PAGE!");
                }
                
                smmu_map_pages(smmu_pa, newpage);
                memcpy32(newpage, smmu_pa, 0x1000);
                //printf("translating @ lv{} (asid {:02x}): lv{} page table translated: {:016x} -> {:016x}, supplanting {:016x}\n\r", level, SMMU_CURRENT_ASID, level+1, smmu_ipa, smmu_pa, newpage);
                
                poke32(curaddr, (tblval & !0x3fffff) | (newpage >> 12) as u32);
                smmu_translatetlb(newpage, deviceaddr, level + 1);
            }
            else
            {
                //if ((!peek32(curaddr+4) || (i && !peek32(curaddr-4))) /*&& SMMU_CURRENT_ASID < 6*/)
                //    printf("translating @ lv{} (asid {:02x}): lv{} block translated: {:016x} -> {:016x}, device addr {:08x}\n\r", level, SMMU_CURRENT_ASID, level, smmu_ipa, smmu_pa, deviceaddr);

                if (SMMU_CURRENT_ASID == 5)
                {
                    if (SE_BUFFER == 0)
                    {
                        println!("(core {}) SE buffer: IPADDR {:016x} -> PADDR {:016x}, SMMU addr {:08x}", get_core(), smmu_ipa, smmu_pa, deviceaddr);
                        SE_BUFFER = smmu_pa;
                        SE_BUFFER_ADJ = deviceaddr;
                    }
                }
                else if (SMMU_CURRENT_ASID == 6)
                {
                    if (SDMMC_BUFFER == 0)
                    {
                        println!("(core {}) SDMMC buffer: IPADDR {:016x} -> PADDR {:016x}, SMMU addr {:08x}", get_core(), smmu_ipa, smmu_pa, deviceaddr);
                        SDMMC_BUFFER = smmu_pa;
                        SDMMC_BUFFER_ADJ = deviceaddr;
                    }
                }
                else if (SMMU_CURRENT_ASID == 7)
                {
                    if (DC_BUFFER == 0)
                    {
                        println!("(core {}) DC buffer: IPADDR {:016x} -> PADDR {:016x}, SMMU addr {:08x}", get_core(), smmu_ipa, smmu_pa, deviceaddr);
                        DC_BUFFER = smmu_pa;
                        DC_BUFFER_ADJ = deviceaddr;
                    }
                }
                else
                {
                    if (ASID_BUFFERS[SMMU_CURRENT_ASID as usize] == 0)
                    {
                        println!("(core {}) ASID {} buffer: IPADDR {:016x} -> PADDR {:016x}, SMMU addr {:08x}", get_core(), SMMU_CURRENT_ASID, smmu_ipa, smmu_pa, deviceaddr);
                        ASID_BUFFERS[SMMU_CURRENT_ASID as usize] = 1;//ASID_BUFFERS;
                        ASID_BASES[SMMU_CURRENT_ASID as usize] = deviceaddr;
                    }
                }
                
                poke32(curaddr, (tblval & !0x3fffff) | (smmu_pa >> 12) as u32);
                //if (SMMU_CURRENT_ASID == 5)
                //    poke32(curaddr, (tblval & !0x3fffff) | (smmu_ipa >> 12));
            }
        }
        dcache_flush(smmu_tlb,0x1000);
    }
}

pub fn smmu_flush_tlb(smmu_tlb: u64, baseaddr: u32, level: i32, asid: u8)
{
    dcache_flush(smmu_tlb,0x1000);
    
    for i in 0..(0x1000/4)
    {
        let addr_to_flush = smmu_tlb + (i*4) as u64;
        let curaddr = addr_to_flush;
        let tblval = peek32(curaddr);
        if (tblval == 0) {
            continue;
        }
        
        
        let ret1 = smmu_writereg(MC_SMMU_PTC_FLUSH_1, (addr_to_flush >> 32) as u32);
        let ret2 = smmu_writereg(MC_SMMU_PTC_FLUSH, ((addr_to_flush & 0xFFFFFFF0) | 1) as u32);
        if (ret1 != 0 || ret2 != 0)
        {
            println!("failed to write reg, {:08x} {:08x}", ret1, ret2);
        }

        let deviceaddr: u32 = (baseaddr + (i * (if level == 0 { 0x400000 } else { 0x1000 })) as u32) as u32;
        let smmu_pa = ((tblval & 0x3fffff) << 12) as u64;
        if ((tblval & 0x10000000) != 0) // page table
        {
            smmu_flush_tlb(smmu_pa, deviceaddr, level + 1, asid);
        }
        
        let ret3 = smmu_writereg(MC_SMMU_TLB_FLUSH, bit!(31) | (asid << 24) as u32 | ((deviceaddr >> 14) << 2) as u32 | 2);
        if (ret3 != 0)
        {
            println!("failed to write reg {:08x}", ret3);
        }
    }
    
    smmu_readreg(0x70019010);
}

pub fn smmu_match_asid(addr: u64) -> i32
{
    unsafe
    {
        for i in 0..0x80
        {
            if (PTB_HOS_ASIDS[i] == addr) {
                return i as i32;
            }
        }
        
        return -1;
    }
}

pub fn smmu_handle_rwreg(ctx: &mut [u64]) -> bool
{
unsafe{
    let reg = (ctx[1] & 0xFFFFFFFF) as u64;
    let is_write = (ctx[2] == 0xFFFFFFFF);
    let mut val = (ctx[3] & 0xFFFFFFFF) as u32;

    if reg != 0x70019054 && reg != 0x700199b8 && reg != 0x70019034 {
        //println_core!("smmu: rwreg {:08x} {} {:08x}", reg, if (is_write) { "<-" } else { "->" }, val);
    }
    
    if (!is_write) {
        if (reg == MC_SMMU_PTB_DATA)
        {
            val = smmu_readreg(reg);
            let smmu_hyp = ((val & 0x3fffff) << 12) as u64;
            let smmu_pa = smmu_find_hos_mapping_from_hyp(smmu_hyp);
            let smmu_ipa = paddr_to_ipaddr(smmu_pa);
            
            val = (val & !0x3fffff) | (smmu_ipa >> 12) as u32;
            
            ctx[0] = 0;
            ctx[1] = val as u64;
            return true;
        }
        return false;
    }
    
    //return;
    
    if (reg == MC_SMMU_PTB_DATA)
    {
        let smmu_ipa = ((val & 0x3fffff) << 12) as u64;
        let smmu_pa = ipaddr_to_paddr(smmu_ipa);

        PTB_HOS_ASIDS[SMMU_CURRENT_ASID as usize] = smmu_pa;
        let mut matched_page = smmu_find_hyp_mapping_from_hos(smmu_pa);
        if (matched_page == 0)
        {
            PTB_HTB_ASIDS[SMMU_CURRENT_ASID as usize] = smmu_allocpage();
            smmu_map_pages(smmu_pa, PTB_HTB_ASIDS[SMMU_CURRENT_ASID as usize]);
            matched_page = PTB_HTB_ASIDS[SMMU_CURRENT_ASID as usize];
        }


        //val = (val & !0x3fffff) | (PTB_HTB_ASIDS[SMMU_CURRENT_ASID] >> 12);
        val = (val & !0x3fffff) | (matched_page >> 12) as u32;
        //printf("core {}: translating IPA {:016x} -> PA {:016x}\n\r", get_core(), smmu_ipa, smmu_pa);
    }
    else if (reg == MC_SMMU_PTC_FLUSH && (val|LAST_MC_SMMU_PTC_FLUSH_HI) != 0) // PTC_FLUSH
    {
        let mut flushing_addr = ((LAST_MC_SMMU_PTC_FLUSH_HI as u64) << 32) | (val & 0xFFFFF000) as u64;
        flushing_addr = ipaddr_to_paddr(flushing_addr);

        let matched_page = smmu_find_hyp_mapping_from_hos(flushing_addr);
        if (matched_page == 0)
        {
            println!("(core {}) FAILED TO MATCH SMMU PAGE! FORCING FLUSH ALL...", get_core());
            val &= !1;
            val &= 0xFFFFFFF0;
            ctx[3] = (val & 0xFFFFFFFF) as u64;
            return false;
        }

        //let mut flushing_asid = smmu_match_asid(flushing_addr);
        let mut level = 0;
        //if (flushing_asid == -1)
        {
            //println!("(core {}) FAILED TO IDENTIFY SMMU ASID! FALLBACK... {:x}", get_core(), flushing_addr);
            
            //flushing_asid = SMMU_CURRENT_ASID as i32;
            //level = 1;
        }
        
        let smmu_hos = flushing_addr;
        let smmu_hyp = matched_page; // TODO?
        
        if (smmu_hyp == 0)
        {
            panic!("HANGING...");
        }
        
        smmu_freetable(smmu_hyp, 0);
        memcpy32(smmu_hyp, smmu_hos, 0x1000);
        
        //printf("(core {}) retranslate ASID {:x} ({:016x}, {:016x} {:08x})\n\r", get_core(), flushing_asid, flushing_addr, smmu_hyp, val);
        smmu_translatetlb(smmu_hyp, 0, level);
        
        if (matched_page == 0)
        {
            println!("(core {}) FAILED TO MATCH SMMU PAGE! FORCING FLUSH ALL...", get_core());
            val = val & !1;
        }
        else
        {
            val = (smmu_hyp & 0xFFFFF000) as u32 | (val & 0xFFF);
        }

        //val = (flushing_addr & 0xFFFFF000) | (val & 0xFFF);
    }
    else if (reg == MC_SMMU_TLB_FLUSH) // lookaside buffer flush
    {
        let va = ((val >> 2) & 0x1FFFF) << 15;
        let asid_flush = ((val >> 24) & 0x1F) as u8;
        //println!("(core {}) ASID {:x} buffer flush VA {:08x} {:02x}\n\r", get_core(), SMMU_CURRENT_ASID, va, asid_flush);
    }
    else if (reg == MC_SMMU_PTB_ASID)
    {
        SMMU_CURRENT_ASID = (val & 0x7F) as u8;
        //println!("(core {}) set ASID {:x}", get_core(), SMMU_CURRENT_ASID);
    }
    else if (reg == MC_SMMU_PTC_FLUSH_1)
    {
        LAST_MC_SMMU_PTC_FLUSH_HI = val;
        //println!("(core {}) ASID {:x} ptbl cache flush addr upper", get_core(), SMMU_CURRENT_ASID);
    }
    else if (reg == MC_SMMU_CONFIG)
    {
        //smmu_flush_tlb(PTB_HTB_ASIDS[SMMU_CURRENT_ASID], 0, 0, SMMU_CURRENT_ASID);
    }
    else if (reg == MC_SMMU_DC_ASID)
    {
        println!("DC ASID: {:08x}", val);
        DC_ASID = val & 0x3F;
        //smmu_print_err();
    }
    else if (reg == MC_SMMU_SDMMC1A_ASID)
    {
        println!("SDMMC1A ASID: {:08x}", val);
        //smmu_print_err();
    }
    else if (reg == MC_SMMU_SDMMC2A_ASID)
    {
        println!("SDMMC2A ASID: {:08x}", val);
        SDMMC_ASID = val & 0x3F;
        //smmu_print_err();
    }
    else if (reg == MC_SMMU_SDMMC3A_ASID)
    {
        println!("SDMMC3A ASID: {:08x}", val);
        //smmu_print_err();
    }
    else if (reg == MC_SMMU_SDMMC4A_ASID)
    {
        println!("SDMMC4A ASID: {:08x}", val);
        //smmu_print_err();
    }

    ctx[3] = (val & 0xFFFFFFFF) as u64;

    return false;
}
}


pub fn smmu_freepage(page: u64)
{
    unsafe
    {
        let pages_ptr = to_u64ptr!(SMMU_PAGES.0.as_mut_ptr());
        let idx = ((page - pages_ptr) / 0x1000) as usize;

        let bit = (idx & 0x7) as u8;
        SMMU_PAGE_ALLOCBITMAP[idx>>3] &= !bit!(bit);
        smmu_unmap_page(page);
    }
}

pub fn smmu_allocpage() -> u64
{
    unsafe
    {
        let pages_ptr = to_u64ptr!(SMMU_PAGES.0.as_mut_ptr());
        for i in 0..(SMMU_NUM_PAGES/8)
        {
            let bits = SMMU_PAGE_ALLOCBITMAP[i];
            if (bits == 0xFF) {
                continue;
            }
            
            let mut bit = 0xFF;
            for j in 0..8
            {
                if ((bits & bit!(j)) != 0) {
                    continue;
                }
                
                bit = j;
                break;
            }
            
            if (bit == 0xFF) {
                continue;
            }
            
            SMMU_PAGE_ALLOCBITMAP[i] |= bit!(bit);
            let offs = ((i*8)+bit)*0x1000;
            let page = (pages_ptr + offs as u64);
            memset32(page, 0, 0x1000);
            return page;
        }
        
        println_core!("!! Exhausted SMMU pages !!");
        return 0;
    }
}

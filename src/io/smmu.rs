/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::util::*;
use crate::arm::threading::*;
use crate::arm::cache::*;
use crate::logger::*;

pub const AHB_BASE: u32 = 0x6000C000;

pub const AHB_ARBITRATION_DISABLE: u32 = (AHB_BASE + 0x004);

pub const MC_BASE: u64 = (0x70019000);
pub const MC_END: u64 = (0x7001A000);

pub const MC_SMMU_CONFIG: u64 = (MC_BASE + 0x10);
pub const MC_SMMU_TLB_CONFIG: u64 = (MC_BASE + 0x14);
pub const MC_SMMU_PTC_CONFIG: u64 = (MC_BASE + 0x18);
pub const MC_SMMU_PTB_ASID: u64 = (MC_BASE + 0x1C);
pub const MC_SMMU_PTB_DATA: u64 = (MC_BASE + 0x20);
pub const MC_SMMU_TLB_FLUSH: u64 = (MC_BASE + 0x30);
pub const MC_SMMU_PTC_FLUSH: u64 = (MC_BASE + 0x34);
pub const MC_SMMU_PPCS1_ASID: u64 = (MC_BASE + 0x298);
pub const MC_SMMU_SDMMC2A_ASID: u64 = (MC_BASE + 0xA98);
pub const MC_SMMU_SDMMC3A_ASID: u64 = (MC_BASE + 0xA9C);

struct SmmuPageMatch
{
    hos_page: u64,
    hyp_page: u64,
    valid: bool,
}

pub const SMMU_NUM_PAGES: usize = 0x100;

const SMMU_PAGE_MATCH_DEF: SmmuPageMatch = SmmuPageMatch {
    hos_page: 0,
    hyp_page: 0,
    valid: false
};

static mut SE_BUFFER: u64 = 0;
static mut SE_BUFFER_ADJ: u32 = 0;

static mut SDMMC_ASID: u32 = 6;
static mut SDMMC_BUFFER: u64 = 0;
static mut SDMMC_BUFFER_ADJ: u32 = 0;

static mut DC_BUFFER: u64 = 0;
static mut DC_BUFFER_ADJ: u32 = 0;

static mut SMMU_CURRENT_ASID: u8 = 0;
static mut SMMU_PAGE_MAPPINGS: [SmmuPageMatch; SMMU_NUM_PAGES] = [SMMU_PAGE_MATCH_DEF; SMMU_NUM_PAGES];

static mut PTB_HOS_ASIDS: [u64; 0x80] = [0; 0x80];
static mut PTB_HTB_ASIDS: [u64; 0x80] = [0; 0x80];
static mut ASID_BUFFERS: [u64; 0x80] = [0; 0x80];
static mut ASID_BASES: [u32; 0x80] = [0; 0x80];
static mut SMMU_PAGE_ALLOCBITMAP: [u8; SMMU_NUM_PAGES/8] = [0; SMMU_NUM_PAGES/8];

/*
u8 smmu_pages[SMMU_NUM_PAGES * 0x1000] __attribute__ ((section (".bss"))) __attribute__ ((aligned (0x1000)));
*/

pub fn smmu_init()
{
    let ahb_arb_disable: MMIOReg = MMIOReg::new(AHB_ARBITRATION_DISABLE);
    
    // TODO actual init
    
    // Allow usbd regs to be arbitrated
    // (SMMU will still be locked out but there's a workaround)
    ahb_arb_disable.w32(0);
}

/*
pub fn smmu_get_se_buffer() -> u64
{
    return SE_BUFFER;
}

pub fn smmu_get_se_buffer_adj() -> u32
{
    return SE_BUFFER_ADJ;
}

pub fn smmu_get_sdmmc_buffer() -> u64
{
    return SDMMC_BUFFER;//ASID_BUFFERS[SDMMC_ASID];
}

pub fn smmu_get_sdmmc_buffer_adj() -> u32
{
    return SDMMC_BUFFER_ADJ;//ASID_BASES[SDMMC_ASID];
}

pub fn smmu_map_pages(hos: u64, hyp: u64)
{
    for i in 0..SMMU_NUM_PAGES
    {
        if (!SMMU_PAGE_MAPPINGS[i].valid)
        {
            SMMU_PAGE_MAPPINGS[i].hos_page = hos;
            SMMU_PAGE_MAPPINGS[i].hyp_page = hyp;
            SMMU_PAGE_MAPPINGS[i].valid = 1;
            break;
        }
    }
}

pub fn smmu_unmap_page(hyp: u64)
{
    for i in 0..SMMU_NUM_PAGES
    {
        if (SMMU_PAGE_MAPPINGS[i].valid && SMMU_PAGE_MAPPINGS[i].hyp_page == hyp)
        {
            SMMU_PAGE_MAPPINGS[i].valid = 0;
        }
    }
}

pub fn smmu_find_hyp_mapping_from_hos(hos: u64) -> u64
{
    for i in 0..SMMU_NUM_PAGES
    {
        if (SMMU_PAGE_MAPPINGS[i].valid && SMMU_PAGE_MAPPINGS[i].hos_page == hos) {
            return SMMU_PAGE_MAPPINGS[i].hyp_page;
        }
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

        //printf("freeing @ lv{} (asid %02x): {:016x}: {:08x}\n\r", level, SMMU_CURRENT_ASID, curaddr, tblval);
        
        let smmu_pa = ((tblval & 0x3fffff) as u64) << 12;

        if (tblval & 0x10000000) // page table
        {
            //printf("freeing @ lv{} (asid %02x): lv{} page table {:016x}\n\r", level, SMMU_CURRENT_ASID, level+1, smmu_pa);
            smmu_freetable(smmu_pa, level + 1);
            smmu_freepage(smmu_pa);
        }
        poke32(curaddr, 0);
    }
}

pub fn smmu_translatetlb_ent(smmu_tlb: u64)
{
    let curaddr = smmu_tlb;
    let tblval = peek32(curaddr);
    if (tblval == 0) {
        return;
    }

    let smmu_ipa = ((tblval & 0x3fffff) as u64) << 12;
    let smmu_pa = ipaddr_to_paddr(smmu_ipa);

    if (tblval & 0x10000000) // page table
    {
        peek32(curaddr) = (tblval & !0x3fffff) | (smmu_pa >> 12);
    }
    else
    {
        peek32(curaddr) = (tblval & !0x3fffff) | (smmu_pa >> 12);
    }
    dcache_flush(smmu_tlb,0x4);
}

pub fn smmu_translatetlb(smmu_tlb: u64, baseaddr: u32, level: i32)
{
    for i in 0..(0x1000/4)
    {
        void* curaddr = smmu_tlb + i*4;
        let deviceaddr = baseaddr + (i * (if level == 0 { 0x400000 } else { 0x1000 })) as u32;
        let tblval = peek32(curaddr);
        if (tblval == 0) {
            continue;
        }
        
        let smmu_ipa = ((tblval & 0x3fffff) as u64) << 12;
        let smmu_pa = ipaddr_to_paddr(smmu_ipa);
        
        //if (level == 0)
        //    printf("translating @ lv{} (asid %02x): {:016x}: {:08x} (ipa {:08x} pa {:08x} va {:08x})\n\r", level, SMMU_CURRENT_ASID, curaddr, tblval, smmu_ipa, smmu_pa, deviceaddr);
        
        if ((tblval & 0x10000000) != 0) // page table
        {
            let newpage = smmu_allocpage();
            if (newpage == 0)
            {
                println!("COULDN'T ALLOC SMMU PAGE!");
                unsafe { t210_reset(); }
            }
            
            smmu_map_pages(smmu_pa, newpage);
            memcpy32(newpage, smmu_pa, 0x1000);
            //printf("translating @ lv{} (asid %02x): lv{} page table translated: {:016x} -> {:016x}, supplanting {:016x}\n\r", level, SMMU_CURRENT_ASID, level+1, smmu_ipa, smmu_pa, newpage);
            
            poke32(curaddr, (tblval & !0x3fffff) | (newpage >> 12));
            smmu_translatetlb(newpage, deviceaddr, level + 1);
        }
        else
        {
            //if ((!peek32(curaddr+4) || (i && !peek32(curaddr-4))) /*&& SMMU_CURRENT_ASID < 6*/)
            //    printf("translating @ lv{} (asid %02x): lv{} block translated: {:016x} -> {:016x}, device addr {:08x}\n\r", level, SMMU_CURRENT_ASID, level, smmu_ipa, smmu_pa, deviceaddr);

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
                if (!ASID_BUFFERS[SMMU_CURRENT_ASID])
                {
                    println!("(core {}) ASID {} buffer: IPADDR {:016x} -> PADDR {:016x}, SMMU addr {:08x}", get_core(), SMMU_CURRENT_ASID, smmu_ipa, smmu_pa, deviceaddr);
                    ASID_BUFFERS[SMMU_CURRENT_ASID] = ASID_BUFFERS;
                    ASID_BASES[SMMU_CURRENT_ASID] = deviceaddr;
                }
            }
            
            poke32(curaddr, (tblval & !0x3fffff) | (smmu_pa >> 12));
            //if (SMMU_CURRENT_ASID == 5)
            //    poke32(curaddr, (tblval & !0x3fffff) | (smmu_ipa >> 12));
        }
    }
    dcache_flush(smmu_tlb,0x1000);
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
        
        
        let ret1 = smmu_writereg(0x700199b8, addr_to_flush >> 32);
        let ret2 = smmu_writereg(0x70019034, (addr_to_flush & 0xFFFFFFF0) | 1);
        if (ret1 || ret2)
        {
            println!("failed to write reg, {:08x} {:08x}", ret1, ret2);
        }

        let deviceaddr: u32 = (baseaddr + (i * (if level == 0 { 0x400000 } else { 0x1000 })) as u32) as u32;
        let smmu_pa = ((tblval & 0x3fffff) << 12) as u64;
        if (tblval & 0x10000000) // page table
        {
            smmu_flush_tlb(smmu_pa, deviceaddr, level + 1, asid);
        }
        
        let ret3 = smmu_writereg(0x70019030, BIT(31) | (asid << 24) | ((deviceaddr >> 14) << 2) | 2);
        if (ret3)
        {
            println!("failed to write reg {:08x}", ret3);
        }
    }
    
    smmu_readreg(0x70019010);
}

pub fn smmu_match_asid(addr: u64) -> i32
{
    for i in 0..0x80
    {
        if (PTB_HOS_ASIDS[i] == addr) {
            return i;
        }
    }
    
    return -1;
}*/

pub fn smmu_handle_rwreg(ctx: &mut [u64])
{
    /*let reg = (ctx[1] & 0xFFFFFFFF) as u32;
    let is_write = (ctx[2] == 0xFFFFFFFF);
    let val = (ctx[3] & 0xFFFFFFFF) as u32;

    //printf("(core {}) smmu: rwreg {:08x} %s {:08x}\n\r", get_core(), reg, (is_write) ? "<-" : "->", val);
    
    if (!is_write) {
        return;
    }
    
    if (reg == 0x70019020) // PTB_DATA
    {
        let smmu_ipa = (val & 0x3fffff) << 12;
        let smmu_pa = ipaddr_to_paddr(smmu_ipa);

        PTB_HOS_ASIDS[SMMU_CURRENT_ASID] = smmu_pa;
        let mut matched_page = smmu_find_hyp_mapping_from_hos(smmu_pa);
        if (matched_page == 0)
        {
            PTB_HTB_ASIDS[SMMU_CURRENT_ASID] = smmu_allocpage();
            smmu_map_pages(smmu_pa, PTB_HTB_ASIDS[SMMU_CURRENT_ASID]);
            matched_page = PTB_HTB_ASIDS[SMMU_CURRENT_ASID];
        }


        //val = (val & !0x3fffff) | (PTB_HTB_ASIDS[SMMU_CURRENT_ASID] >> 12);
        val = (val & !0x3fffff) | (matched_page >> 12);
        //printf("core {}: translating IPA {:016x} -> PA {:016x}\n\r", get_core(), smmu_ipa, smmu_pa);
    }
    else if (reg == 0x70019034 && val) // PTC_FLUSH
    {
        let flushing_addr = val & 0xFFFFF000;
        flushing_addr = ipaddr_to_paddr(flushing_addr);

        let matched_page = smmu_find_hyp_mapping_from_hos(flushing_addr);
        if (matched_page == 0)
        {
            println!("(core {}) FAILED TO MATCH SMMU PAGE! FORCING FLUSH ALL...", get_core());
            val &= !1;
            val &= 0xFFFFFFF0;
            return;
        }

        //TODO use upper val too
        let flushing_asid = smmu_match_asid(flushing_addr);
        let mut level = 0;
        if (flushing_asid == -1)
        {
            //println!("(core {}) FAILED TO IDENTIFY SMMU ASID! FALLBACK...", get_core());
            
            flushing_asid = SMMU_CURRENT_ASID;
            level = 1;
        }
        
        let smmu_hos = flushing_addr;
        let smmu_hyp = matched_page; // TODO?
        
        if (smmu_hyp == 0)
        {
            printf("HANGING...\n\r");
            t210_reset();
        }
        
        smmu_freetable(smmu_hyp, 0);
        memcpy32(smmu_hyp, smmu_hos, 0x1000);
        
        //printf("(core {}) retranslate ASID %x ({:016x}, {:016x} {:08x})\n\r", get_core(), flushing_asid, flushing_addr, smmu_hyp, val);
        smmu_translatetlb(smmu_hyp, 0, level);
        
        if (!matched_page)
        {
            //printf("(core {}) FAILED TO MATCH SMMU PAGE! FORCING FLUSH ALL...\n\r", get_core());
            val = val & !1;
        }
        else
        {
            val = (smmu_hyp & 0xFFFFF000) | (val & 0xFFF);
        }

        //val = (flushing_addr & 0xFFFFF000) | (val & 0xFFF);
    }
    else if (reg == 0x70019030) // lookaside buffer flush
    {
        let va = ((val >> 2) & 0x1FFFF) << 15;
        let asid_flush = ((val >> 24) & 0x1F) as u8;
        //printf("(core {}) ASID %x buffer flush VA {:08x} %02x\n\r", get_core(), SMMU_CURRENT_ASID, va, asid_flush);
    }
    else if (reg == 0x7001901c)
    {
        SMMU_CURRENT_ASID = val & 0x7F;
        //printf("(core {}) set ASID %x\n\r", get_core(), SMMU_CURRENT_ASID);
    }
    else if (reg == 0x700199b8)
    {
        //printf("(core {}) ASID %x ptbl cache flush addr upper\n\r", get_core(), SMMU_CURRENT_ASID);
    }
    else if (reg == 0x70019010)
    {
        //smmu_flush_tlb(PTB_HTB_ASIDS[SMMU_CURRENT_ASID], 0, 0, SMMU_CURRENT_ASID);
    }
    else if (reg == 0x70019A94)
    {
        println!("SDMMC1A ASID: {:08x}", val);
    }
    else if (reg == 0x70019A98)
    {
        println!("SDMMC2A ASID: {:08x}", val);
        //SDMMC_ASID = val & 0x3F;
    }
    else if (reg == 0x70019A9C)
    {
        println!("SDMMC3A ASID: {:08x}", val);
    }
    else if (reg == 0x70019AA0)
    {
        println!("SDMMC4A ASID: {:08x}", val);
    }*/

    return;
}

/*
pub fn smmu_freepage(page: u64)
{
    let idx = ((page - smmu_pages) / 0x1000) as u16;

    let bit = (idx & 0x7) as u8;
    SMMU_PAGE_ALLOCBITMAP[idx>>3] &= ~bit!(bit);
    smmu_unmap_page(page);
}

pub fn smmu_allocpage() -> u64
{
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
        let page = (smmu_pages + ((i*8)+bit)*0x1000);
        memset32(page, 0, 0x1000);
        return page;
    }
    
    return 0;
}*/

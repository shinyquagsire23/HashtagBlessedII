/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
use core::ops;
use core::mem;
use core::str;

extern "C" {
    pub fn t210_reset();
}

#[macro_use]
mod util {
    macro_rules! bit {
        ($a:expr) => {
            (1 << $a)
        }
    }
    
    macro_rules! to_u64ptr {
        ($a:expr) => {
            (($a as *const _) as u64)
        }
    }
    
    macro_rules! sysreg_read {
        ($a:expr) => {
            unsafe { let mut out: u64 = 0; asm!(concat!("mrs {0}, ", $a), out(reg) out); out }
        }
    }
    
    macro_rules! sysreg_write {
        ($a:expr, $b:expr) => {
            unsafe { let val: u64 = ($b) as u64; asm!(concat!(concat!("msr ", $a), ", {0}"), in(reg) val) }
        }
    }
    
    macro_rules! sysreg_or64 {
        ($a:expr, $b:expr) => {
            sysreg_write!($a, sysreg_read!($a) | ($b))
        }
    }
    
    macro_rules! sysreg_and64 {
        ($a:expr, $b:expr) => {
            sysreg_write!($a, sysreg_read!($a) & ($b))
        }
    }
    macro_rules! kstr {
        ($a:expr) => {
            (str_from_null_terminated_utf8_u64ptr_unchecked(crate::arm::mmu::translate_el1_stage12($a)))
        }
    }
    
    macro_rules! kstr_len {
        ($a:expr, $b:expr) => {
            (str_from_null_terminated_utf8_u64ptr_unchecked_len(crate::arm::mmu::translate_el1_stage12($a), $b))
        }
    }
}

pub fn str_from_null_terminated_utf8_u64ptr_unchecked(s: u64) -> &'static str {
    unsafe {
        let s_raw = s as *const u8;
        let mut s_len = 0;
        loop
        {
            if s_raw.offset(s_len).read() == 0
            {
                break;
            }
            s_len += 1;
        }
        let s_slice: &'static [u8] = alloc::slice::from_raw_parts(s as *const u8, (s_len) as usize);
        str::from_utf8_unchecked(&s_slice)
    }
}

pub fn str_from_null_terminated_utf8_u64ptr_unchecked_len(s: u64, len: u32) -> &'static str {
    unsafe {
        let s_raw = s as *const u8;
        let mut s_len = 0;
        loop
        {
            if s_raw.offset(s_len).read() == 0
            {
                break;
            }
            s_len += 1;
        }
        
        if s_len > s_len {
            s_len = len as isize;
        }
        
        let s_slice: &'static [u8] = alloc::slice::from_raw_parts(s as *const u8, (s_len) as usize);
        str::from_utf8_unchecked(&s_slice)
    }
}

pub fn str_from_null_terminated_utf8_unchecked(s: &[u8]) -> &str {
    unsafe { str::from_utf8_unchecked(s) }
}

#[inline(always)]
pub fn peekio32(addr: u32) -> u32 {
    unsafe {
        let mut_reg: *mut u32 = (addr) as _;
        return mut_reg.read_volatile();
    }
}

#[inline(always)]
pub fn pokeio32(addr: u32, val: u32) {
    unsafe {
        let mut_reg: *mut u32 = (addr) as _;
        mut_reg.write_volatile(val);
    }
}

#[inline(always)]
pub fn peek64(addr: u64) -> u64 {
    unsafe {
        let mut_reg: *mut u64 = (addr) as _;
        return mut_reg.read_volatile();
    }
}

#[inline(always)]
pub fn poke64(addr: u64, val: u64) {
    unsafe {
        let mut_reg: *mut u64 = (addr) as _;
        mut_reg.write_volatile(val);
    }
}

#[inline(always)]
pub fn peek32(addr: u64) -> u32 {
    unsafe {
        let mut_reg: *mut u32 = (addr) as _;
        return mut_reg.read_volatile();
    }
}

#[inline(always)]
pub fn poke32(addr: u64, val: u32) {
    unsafe {
        let mut_reg: *mut u32 = (addr) as _;
        mut_reg.write_volatile(val);
    }
}

#[inline(always)]
pub fn peek16(addr: u64) -> u16 {
    unsafe {
        let mut_reg: *mut u16 = (addr) as _;
        return mut_reg.read_volatile();
    }
}

#[inline(always)]
pub fn poke16(addr: u64, val: u16) {
    unsafe {
        let mut_reg: *mut u16 = (addr) as _;
        mut_reg.write_volatile(val);
    }
}

#[inline(always)]
pub fn peek8(addr: u64) -> u8 {
    unsafe {
        let mut_reg: *mut u8 = (addr) as _;
        return mut_reg.read_volatile();
    }
}

#[inline(always)]
pub fn poke8(addr: u64, val: u8) {
    unsafe {
        let mut_reg: *mut u8 = (addr) as _;
        mut_reg.write_volatile(val);
    }
}

#[inline(always)]
pub fn memset_iou32(addr: u64, val: u32, len: usize) {
    unsafe {
        let aligned_len = (len + 3) & !0x3;
        for i in 0..(aligned_len/4)
        {
            let mut_reg: *mut u32 = (addr + (i*4) as u64) as _;
            mut_reg.write_volatile(val);
        }
    }
}

#[inline(always)]
pub fn memcpy_iou32(dst: u64, src: u64, len: usize) {
    unsafe {
        let aligned_len = (len + 3) & !0x3;
        for i in 0..(aligned_len/4)
        {
            let mut_dst: *mut u32 = (dst + (i*4) as u64) as _;
            let mut_src: *const u32 = (src + (i*4) as u64) as _;
            let val = mut_src.read_volatile();
            mut_dst.write_volatile(val);
        }
    }
}

#[inline(always)]
pub fn memcpy32(dst: u64, src: u64, len: usize) {
    memcpy_iou32(dst, src, len);
}

#[inline(always)]
pub fn memset32(dst: u64, val: u32, len: usize) {
    memset_iou32(dst, val, len);
}

#[derive(Copy, Clone)]
pub struct MMIOReg
{
    pub addr: u32
    //mut_reg: *mut u32
}

impl ops::Add<u32> for MMIOReg {
    type Output = u32;

    fn add(self, _rhs: u32) -> u32 {
        let out: u32 = self.read() + _rhs;
        
        self.write(out);

        return out;
    }
}

impl ops::BitOr<u32> for MMIOReg {
    type Output = u32;

    fn bitor(self, _rhs: u32) -> u32 {
        let out: u32 = self.read() | _rhs;

        return out;
    }
}

impl ops::BitOrAssign<u32> for MMIOReg {
    fn bitor_assign(&mut self, _rhs: u32) {
        let out: u32 = self.read() | _rhs;
        
        self.write(out);
    }
}

impl ops::BitAnd<u32> for MMIOReg {
    type Output = u32;

    fn bitand(self, _rhs: u32) -> u32 {
        let out: u32 = self.read() & _rhs;

        return out;
    }
}

impl ops::BitAndAssign<u32> for MMIOReg {
    fn bitand_assign(&mut self, _rhs: u32) {
        let out: u32 = self.read() & _rhs;
        
        self.write(out);
    }
}

/*impl ops::Fn<(u32,)> for MMIOReg {
    extern "rust-call" fn call(&self, args: (u32,)) -> Self {
        self.write(args.0);
        return self.clone();
    }
}

impl ops::FnOnce<(u32,)> for MMIOReg {
    type Output = Self;
    extern "rust-call" fn call_once(self, args: (u32,)) -> Self {
        self.call(args);
        return self;
    }
}

impl ops::FnMut<(u32,)> for MMIOReg {
    extern "rust-call" fn call_mut(&mut self, args: (u32,)) -> Self {
        self.call(args);
        return self.clone();
    }
}*/

impl MMIOReg
{
    pub fn new(addr: u32) -> Self {
        MMIOReg {
        addr: addr
        }
    }
    
    pub fn read(&self) -> u32 {
        unsafe
        {
            let mut_reg: *mut u32 = (self.addr) as _;
            return mut_reg.read_volatile();
        }
    }
    
    pub fn write(&self, val: u32) {
        unsafe
        {
            let mut_reg: *mut u32 = (self.addr) as _;
            mut_reg.write_volatile(val);
        }
    }
    
    pub fn r32(&self) -> u32 {
        unsafe
        {
            let mut_reg: *mut u32 = (self.addr) as _;
            return mut_reg.read_volatile();
        }
    }
    
    pub fn w32(&self, val: u32) {
        unsafe
        {
            let mut_reg: *mut u32 = (self.addr) as _;
            mut_reg.write_volatile(val);
        }
    }
    
    pub fn or32(&mut self, val: u32) {
        *self |= val;
    }
    
    pub fn and32(&mut self, val: u32) {
        *self &= val;
    }
    
    pub fn r8(&self) -> u8 {
        unsafe
        {
            let mut_reg: *mut u8 = (self.addr) as _;
            return mut_reg.read_volatile();
        }
    }
    
    pub fn w8(&self, val: u8) {
        unsafe
        {
            let mut_reg: *mut u8 = (self.addr) as _;
            mut_reg.write_volatile(val);
        }
    }
    
    pub fn set8(&self, val: u8) {
        let old: u8 = self.r8();
        self.w8(old | val);
    }
    
    pub fn unset8(&self, val: u8) {
        let old: u8 = self.r8();
        self.w8(old & !val);
    }
    
    pub fn bits_set(&self, val: u32) -> bool {
        return ((self.read() & val) != 0);
    }
    
    pub fn idx8(&self, idx: u32) -> MMIOReg {
        return MMIOReg::new(self.addr + (idx as u32));
    }
    
    pub fn idx32(&self, idx: u32) -> MMIOReg {
        return MMIOReg::new(self.addr + ((idx as u32) * (mem::size_of::<u32>() as u32)));
    }
}

/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
use core::ops;
use core::mem;

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
}

#[inline(always)]
pub fn peek32(addr: u32) -> u32 {
    unsafe {
        let mut_reg: *mut u32 = (addr) as _;
        return mut_reg.read_volatile();
    }
}

#[inline(always)]
pub fn poke32(addr: u32, val: u32) {
    unsafe {
        let mut_reg: *mut u32 = (addr) as _;
        mut_reg.write_volatile(val);
    }
}

#[inline(always)]
pub fn memset_iou32(addr: u64, val: u32, len: usize) {
    unsafe {
        for i in 0..(len/4)
        {
            let mut_reg: *mut u32 = (addr + (i*4) as u64) as _;
            mut_reg.write_volatile(val);
        }
    }
}

#[inline(always)]
pub fn memcpy_iou32(dst: u64, src: u64, len: usize) {
    unsafe {
        for i in 0..(len/4)
        {
            let mut_dst: *mut u32 = (dst + (i*4) as u64) as _;
            let mut_src: *const u32 = (src + (i*4) as u64) as _;
            let val = mut_src.read_volatile();
            mut_dst.write_volatile(val);
        }
    }
}

#[derive(Copy, Clone)]
pub struct MMIOReg
{
    addr: u32
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

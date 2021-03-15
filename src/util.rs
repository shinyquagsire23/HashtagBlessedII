/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
use core::ops;

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
        
        self.write(out);

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
        
        self.write(out);

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
    
    pub fn bits_set(&self, val: u32) -> bool {
        return (self.read() & val != 0);
    }
}

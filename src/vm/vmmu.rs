/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

pub fn ipaddr_to_paddr(ipaddr: u64) -> u64
{
    let ipaddr_trunc = ipaddr & 0xFFFFFFFFF;
    if (ipaddr_trunc < 0xD0000000) {
        return ipaddr;
    }

    //if (ipaddr_trunc >= 0x84000000 && ipaddr_trunc < 0x86000000)
        //return ipaddr;
    //    return ipaddr = (ipaddr - 0x84000000 + 0xF0000000 );

/*
    if (ipaddr_trunc >= 0x90000000 && ipaddr_trunc < 0x90400000)
        return ipaddr;
    
    if (ipaddr_trunc >= 0xC0000000 && ipaddr_trunc < 0xC0400000)
        return ipaddr;
*/

    let mut paddr = (ipaddr + 0x8000000);
    
/*
    if (paddr >= 0x90000000 && paddr < 0x90400000)
        paddr += 0x400000;
    
    if (paddr >= 0xC0000000 && paddr < 0xC0400000)
        paddr += 0x400000;
*/

    //if (paddr >= 0x84000000 && paddr < 0x86000000)
    //    paddr += 0x2000000;
    
    //if (paddr >= 0xF0000000 && paddr < 0xF2000000)
    //    paddr = (paddr - 0xF0000000 + 0x84000000);

    if (paddr > 0x200000000) {
        paddr = 0;
    }

    return paddr;
}

pub fn vttbr_construct()
{
    // TODO
}

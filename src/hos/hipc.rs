/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use alloc::vec::Vec;
use crate::util::*;

pub const MAGIC_SFCI: u32 = 0x49434653;
pub const MAGIC_SFCO: u32 = 0x4F434653;

pub const HIPC_MAX_BUFS: usize = 8;
pub const HIPC_MAX_OBJS: usize = 8;

/*enum HIPCPacketType : u16
{
    HIPCPacketType_Invalid = 0,
    HIPCPacketType_LegacyRequest = 1,
    HIPCPacketType_Close = 2,
    HIPCPacketType_LegacyControl = 3,
    HIPCPacketType_Request = 4,
    HIPCPacketType_Control = 5,
    HIPCPacketType_RequestWithContext = 6,
    HIPCPacketType_ControlWithContext = 7
};

enum HIPCDomainCommand : u8
{
    HIPCDomainCommand_Send = 1,
    HIPCDomainCommand_CloseVirtualHandle = 2,
};*/

pub struct HIPCHandleDesc
{
    send_pid: bool,
    num_copy: u8,
    num_move: u8,
    pid: u8,
    copy_handles: Vec<u32>,
    move_handles: Vec<u32>
}

impl HIPCHandleDesc
{
    pub fn unpack(buf: u64) -> HIPCHandleDesc
    {
        let word0 = peek32(buf);
        
        let send_pid = (word0 & 1) != 0;
        let num_copy = ((word0 >> 1) & 0xF) as u8;
        let num_move = ((word0 >> 5) & 0xF) as u8;
        
        let pid = (if send_pid { peek32(buf + 4) } else { 0 } & 0xFF) as u8;
        
        let mut buf_inc = buf + 4;
        if send_pid
        {
            buf_inc += 4;
        }

        let mut copy_handles: Vec<u32> = Vec::new();
        for i in 0..num_copy
        {
            let handle = peek32(buf_inc);
            buf_inc += 4;
            copy_handles.push(handle);
        }
        
        let mut move_handles: Vec<u32> = Vec::new();
        for i in 0..num_move
        {
            let handle = peek32(buf_inc);
            buf_inc += 4;
            move_handles.push(handle);
        }
        
        HIPCHandleDesc
        {
            send_pid: send_pid,
            num_copy: num_copy,
            num_move: num_move,
            pid: pid,
            copy_handles: copy_handles,
            move_handles: move_handles
        }
    }
    
    pub fn packed_size(&self) -> u64
    {
        let mut ret_size = 4;
        if self.send_pid {
            ret_size += 4;
        }
        ret_size += (4 * self.num_copy);
        ret_size += (4 * self.num_move);

        return ret_size as u64;
    }
}

pub struct HIPCStaticDesc
{
    index: u16,
    addr: u64,
    size: u16,
}

impl HIPCStaticDesc
{
    pub fn unpack(buf: u64) -> HIPCStaticDesc
    {
        let word0 = peek32(buf);
        let word1 = peek32(buf+4);
        
        let index5to0 = (word0 & 0x3F) as u16;
        let addr38to36 = ((word0 >> 6) & 0x7) as u64;
        let index11to9 = ((word0 >> 9) & 0x7) as u16;
        let addr35to32 = ((word0 >> 12) & 0xF) as u64;
        let size = ((word0 >> 16) & 0xFFFF) as u16;
        let addr31to0 = word1 as u64;
        
        let addr = addr31to0 | (addr35to32 << 32) | (addr38to36 << 36);
        let index = index5to0 | (index11to9 << 9);
        
        HIPCStaticDesc
        {
            index: index,
            addr: addr,
            size: size
        }
    }
    
    pub const fn packed_size(&self) -> u64
    {
        8
    }
}

pub struct HIPCSendRecvExchDesc
{
    addr: u64,
    size: u64,
    flags: u8
}

impl HIPCSendRecvExchDesc
{
    pub fn unpack(buf: u64) -> HIPCSendRecvExchDesc
    {
        let word0 = peek32(buf);
        let word1 = peek32(buf+4);
        let word2 = peek32(buf+8);
        
        let size31to0 = word0 as u64;
        let addr31to0 = word1 as u64;
        let flags = (word2 & 3)  as u8;
        let addr38to36 = ((word2 >> 2) & 7) as u64;
        let size35to32 = ((word2 >> 24) & 0xF) as u64;
        let addr35to32 = ((word2 >> 28) & 0xF) as u64;
        
        let addr = addr31to0 | (addr35to32 << 32) | (addr38to36 << 36);
        let size = size31to0 | (size35to32 << 32);
        
        HIPCSendRecvExchDesc
        {
            addr: addr,
            size: size,
            flags: flags
        }
    }
    
    pub const fn packed_size(&self) -> u64
    {
        12
    }
}

pub struct HIPCPacket
{
    pkt_type: u16,
    num_static: u8,
    num_send: u8,
    num_recv: u8,
    num_exch: u8,
    data_size: u16,
    recv_static_flags: u8,
    unk1: u8,
    unk2: u16,
    enable_handle: bool,
    handle_desc: Option<HIPCHandleDesc>,
    static_descs: Vec<HIPCStaticDesc>,
    send_descs: Vec<HIPCSendRecvExchDesc>,
    recv_descs: Vec<HIPCSendRecvExchDesc>,
    exch_descs: Vec<HIPCSendRecvExchDesc>,
    raw_data: Vec<u8>
}

impl HIPCPacket
{
    pub fn unpack(cmd_buf: u64) -> HIPCPacket
    {
        let word0 = peek32(cmd_buf);
        let word1 = peek32(cmd_buf + 4);

        let pkt_type = (word0 & 0xFFFF) as u16;
        let num_static = ((word0 >> 16) & 0xF) as u8;
        let num_send = ((word0 >> 20) & 0xF) as u8;
        let num_recv = ((word0 >> 24) & 0xF) as u8;
        let num_exch = ((word0 >> 28) & 0xF) as u8;

        let data_size = (word1 & 0x3FF) as u16;
        let recv_static_flags = ((word1 >> 10) & 0xF) as u8;
        let unk1 = ((word1 >> 14) & 0x7F) as u8;
        let unk2 = ((word1 >> 21) & 0x3FF) as u16;
        let enable_handle = (((word1 >> 30) & 1) != 0);
        
        let mut read_ptr = cmd_buf + 8;
        
        // Unpack Handle descriptors
        let mut handle_desc: Option<HIPCHandleDesc> = None;
        if enable_handle
        {
            let unpack_handledesc: HIPCHandleDesc = HIPCHandleDesc::unpack(read_ptr);
            read_ptr += unpack_handledesc.packed_size();
            
            handle_desc = Some(unpack_handledesc);
        }
        
        // Unpack Static descriptors
        let mut static_descs: Vec<HIPCStaticDesc> = Vec::new();
        for i in 0..num_static
        {
            let unpack_static: HIPCStaticDesc = HIPCStaticDesc::unpack(read_ptr);
            read_ptr += unpack_static.packed_size();
            
            static_descs.push(unpack_static);
        }
        
        // Unpack Send descriptors
        let mut send_descs: Vec<HIPCSendRecvExchDesc> = Vec::new();
        for i in 0..num_send
        {
            let unpack_desc: HIPCSendRecvExchDesc = HIPCSendRecvExchDesc::unpack(read_ptr);
            read_ptr += unpack_desc.packed_size();
            
            send_descs.push(unpack_desc);
        }
        
        // Unpack Recv descriptors
        let mut recv_descs: Vec<HIPCSendRecvExchDesc> = Vec::new();
        for i in 0..num_recv
        {
            let unpack_desc: HIPCSendRecvExchDesc = HIPCSendRecvExchDesc::unpack(read_ptr);
            read_ptr += unpack_desc.packed_size();
            
            recv_descs.push(unpack_desc);
        }
        
        // Unpack Exchange descriptors
        let mut exch_descs: Vec<HIPCSendRecvExchDesc> = Vec::new();
        for i in 0..num_exch
        {
            let unpack_desc: HIPCSendRecvExchDesc = HIPCSendRecvExchDesc::unpack(read_ptr);
            read_ptr += unpack_desc.packed_size();
            
            exch_descs.push(unpack_desc);
        }
        
        read_ptr = (((read_ptr + 4 /* sizeof(u32) */ * 2) + 0xF) & !0xF) - 0x8; // align to 0x10
        let mut raw_data: Vec<u8> = Vec::new();
        for i in 0..data_size
        {
            raw_data.push(peek8(read_ptr));
            read_ptr += 1;
        }

        HIPCPacket
        {
            pkt_type: pkt_type,
            num_static: num_static,
            num_send: num_send,
            num_recv: num_recv,
            num_exch: num_exch,
            data_size: data_size,
            recv_static_flags: recv_static_flags,
            unk1: unk1,
            unk2: unk2,
            enable_handle: enable_handle,
            handle_desc: handle_desc,
            static_descs: static_descs,
            send_descs: send_descs,
            recv_descs: recv_descs,
            exch_descs: exch_descs,
            raw_data: raw_data,
        }
    }
    
    pub fn pack()
    {
        
    }
}

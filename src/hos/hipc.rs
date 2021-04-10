/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use alloc::vec::Vec;
use crate::util::*;
use crate::arm::threading::get_tls_el0;
use crate::arm::mmu::translate_el1_stage12;
use crate::logger::*;
use crate::vm::vsvc::vsvc_get_curpid;
use alloc::{collections::BTreeMap, sync::Arc};
use core::cmp::{Ordering, Eq};
use super::hhandle::HHandle;
use super::hport::HPort;
use super::hclientsession::{HClientSession, HClientSessionHandler};
use alloc::string::String;
use spin::mutex::Mutex;
use core::str;
use super::hdomainobj::HDomainObj;
use super::hdomainsession::HDomainSession;

pub const MAGIC_SFCI: u32 = 0x49434653;
pub const MAGIC_SFCO: u32 = 0x4F434653;

pub const HIPC_MAX_BUFS: usize = 8;
pub const HIPC_MAX_OBJS: usize = 8;

pub const PKT_TYPE_INVALID:            u16 = 0;
pub const PKT_TYPE_LEGACYREQEST:       u16 = 1;
pub const PKT_TYPE_CLOSE:              u16 = 2;
pub const PKT_TYPE_LEGACYCONTROL:      u16 = 3;
pub const PKT_TYPE_REQUEST:            u16 = 4;
pub const PKT_TYPE_CONTROL:            u16 = 5;
pub const PKT_TYPE_REQUESTWITHCONTEXT: u16 = 6;
pub const PKT_TYPE_CONTROLWITHCONTEXT: u16 = 7;

pub const DOMAIN_CMD_SEND:     u8 = 1;
pub const DOMAIN_CMD_CLOSEOBJ: u8 = 2;

#[derive(Clone)]
pub struct HExtraNone
{
}

#[derive(Clone)]
pub struct HNone
{
}

#[derive(Clone)]
pub struct HExtraString
{
    pub str: String
}

#[derive(Clone)]
pub struct HExtraU32
{
    pub val: u32
}

#[derive(Clone)]
pub enum HObject
{
    None(),
    ClientSession(Arc<Mutex<HClientSession>>),
    DomainSession(Arc<Mutex<HDomainSession>>),
    Port(Arc<Mutex<HPort>>)
}

#[derive(Clone)]
pub enum HObjectExtra
{
    None(HExtraNone),
    String(HExtraString),
    U32(HExtraU32)
}

static mut DOMAINOBJ_TO_SESSION: BTreeMap<HDomainObj, Arc<Mutex<HDomainSession>>> = BTreeMap::new();
static mut HANDLE_TO_OBJ: BTreeMap<HHandle, HObject> = BTreeMap::new();
static mut NAME_TO_SERVERPORT: BTreeMap<String, Arc<Mutex<HPort>>> = BTreeMap::new();

impl HObject
{
    pub fn get_extra(&self) -> HObjectExtra
    {
        match self
        {
            HObject::ClientSession(a) => { a.lock().get_extra() },
            HObject::DomainSession(a) => { a.lock().get_extra() },
            _ => { HObjectExtra::None(HExtraNone{}) }
        }
    }
    
    pub fn set_extra(&self, extra: HObjectExtra)
    {
        match self
        {
            HObject::ClientSession(a) => { a.lock().set_extra(extra); },
            HObject::DomainSession(a) => { a.lock().set_extra(extra); },
            _ => { HObjectExtra::None(HExtraNone{}); }
        }
    }
    
    pub fn set_extra_str(&self, str: &String)
    {
        let extra = HExtraString { str: str.clone() };
        self.set_extra(HObjectExtra::String(extra));
    }
    
    pub fn set_extra_u32(&self, val: u32)
    {
        let extra = HExtraU32 { val: val };
        self.set_extra(HObjectExtra::U32(extra));
    }
}

pub fn hipc_register_domain(obj: HDomainObj, session: Arc<Mutex<HDomainSession>>)
{
    unsafe
    {
        DOMAINOBJ_TO_SESSION.insert(obj, session);
    }
}

pub fn hipc_remove_domain(obj: HDomainObj)
{
    unsafe
    {
        DOMAINOBJ_TO_SESSION.remove(&obj);
    }
}

pub fn hipc_remove_pid_handles(pid: u32)
{
    let pid_u8 = (pid & 0xFF) as u8;
    let mut handle_vec: Vec<&HHandle> = Vec::new();
    let mut domain_vec: Vec<&HDomainObj> = Vec::new();
    
    unsafe
    {
        for (key, val) in &HANDLE_TO_OBJ {
            if key.pid == pid_u8 {
                handle_vec.push(key);
            }
        }
        
        for (key, val) in &DOMAINOBJ_TO_SESSION {
            if key.pid == pid_u8 {
                domain_vec.push(key);
            }
        }
        
        for key in handle_vec {
            HANDLE_TO_OBJ.remove(key);
        }
        
        for key in domain_vec {
            DOMAINOBJ_TO_SESSION.remove(key);
        }
    }
}

pub fn hipc_remove_domains_from_handle(hhand: &HHandle)
{
    let mut domain_vec: Vec<&HDomainObj> = Vec::new();
    
    unsafe
    {
        for (key, val) in &DOMAINOBJ_TO_SESSION {
            if key.handle == hhand.handle && key.pid == hhand.pid {
                domain_vec.push(key);
            }
        }
        
        for key in domain_vec {
            DOMAINOBJ_TO_SESSION.remove(key);
        }
    }
}

pub fn hipc_register_handle_serverport(handle: u32, port: Arc<Mutex<HPort>>)
{
    unsafe
    {
        let name = port.lock().name.clone();
        HANDLE_TO_OBJ.insert(HHandle::from_curpid(handle), HObject::Port(port.clone()));
        if let Some(name) = &name
        {
            NAME_TO_SERVERPORT.insert(name.clone(), port);
        }
    }
}

pub fn hipc_register_handle_clientsession(handle: u32, session: Arc<Mutex<HClientSession>>)
{
    unsafe
    {
        HANDLE_TO_OBJ.insert(HHandle::from_curpid(handle), HObject::ClientSession(session));
    }
}

pub fn hipc_get_domain_session(obj: HDomainObj) -> Option<Arc<Mutex<HDomainSession>>>
{
    unsafe
    {
        if let Some(arc_res) = DOMAINOBJ_TO_SESSION.get(&obj)
        {
            return Some(arc_res.clone());
        }
        return None;
    }
}

pub fn hipc_get_handle_obj(handle: u32) -> Option<HObject>
{
    unsafe
    {
        let hhandle = HHandle::from_curpid(handle);
        if let Some(arc_res) = HANDLE_TO_OBJ.get(&hhandle)
        {
            return Some(arc_res.clone());
        }
        return None;
    }
}

pub fn hipc_close_handle(handle: u32)
{
    unsafe
    {
        let hhandle = HHandle::from_curpid(handle);
        if let Some(arc_res) = HANDLE_TO_OBJ.remove(&hhandle)
        {
            hipc_remove_domains_from_handle(&hhandle);
            // TODO object delete hook?
        }
    }
}

pub fn hipc_get_handle_serverport(handle: u32) -> Option<Arc<Mutex<HPort>>>
{
    unsafe
    {
        let hhandle = HHandle::from_curpid(handle);
        if let Some(arc_res) = HANDLE_TO_OBJ.get(&hhandle)
        {
            match arc_res
            {
                HObject::Port(port) => { return Some(port.clone()); },
                _ => { return None; }
            }
        }
        return None;
    }
}

pub fn hipc_get_named_serverport(name: &String) -> Option<Arc<Mutex<HPort>>>
{
    unsafe
    {
        if let Some(arc_res) = NAME_TO_SERVERPORT.get(name)
        {
            return Some(arc_res.clone());
        }
        return None;
    }
}

pub fn hipc_get_handle_clientsession(handle: u32) -> Option<Arc<Mutex<HClientSession>>>
{
    unsafe
    {
        let hhandle = HHandle::from_curpid(handle);
        if let Some(arc_handle) = HANDLE_TO_OBJ.get(&hhandle)
        {
            match arc_handle
            {
                HObject::ClientSession(session) => { return Some(session.clone()); },
                _ => { return None; }
            }
        }
        return None;
    }
}

enum HIPCPayload
{
    Domain(HIPCDomainPayload),
    Session(HIPCDataPayload)
}

pub struct HIPCDomainPayload
{
    cmd: u8,
    num_objs: u8,
    data_size: u16,
    obj_id: u32,
    token: u32,
    data: HIPCDataPayload,
    objs: Vec<u32>,
}

impl HIPCDomainPayload
{
    pub fn unpack(buf: u64, buf_size: u16) -> HIPCDomainPayload
    {
        let word0 = peek32(buf);
        
        let cmd = (word0 & 0xFF) as u8;
        let num_objs = ((word0 >> 8) & 0xFF) as u8;
        let data_size = ((word0 >> 16) & 0xFFFF) as u16;
        
        // TODO verify buf_size
        
        let obj_id = peek32(buf + 4);
        let pad = peek32(buf + 8);
        let token = peek32(buf + 12);
        
        let mut buf_objs = buf + 16 + (data_size as u64);
        
        let buf_data = buf + 16;
        let data = HIPCDataPayload::unpack(buf_data, buf_size-0x10);
        
        let mut objs: Vec<u32> = Vec::with_capacity(num_objs as usize);
        for i in 0..num_objs
        {
            objs.push(peek32(buf_objs));
            buf_objs += 4;
        }
        
        HIPCDomainPayload
        {
            cmd: cmd,
            num_objs: num_objs,
            data_size: data_size,
            obj_id: obj_id,
            token: token,
            data: data,
            objs: objs
        }
    }
    
    pub fn packed_size(&self) -> u64
    {
        16 + self.data_size as u64 + ((self.num_objs as u64) * 4)
    }
    
    pub fn get_cmd_id(&self) -> u32
    {
        self.data.command
    }
    
    pub fn get_domain_cmd(&self) -> u8
    {
        self.cmd
    }
    
    pub fn get_domain_id(&self) -> u32
    {
        self.obj_id
    }
    
    pub fn get_domain_obj(&self, idx: usize) -> Option<u32>
    {
        if idx >= self.num_objs as usize
        {
            return None
        }
        
        return Some(self.objs[idx]);
    }
    
    pub fn read_u8(&self, offs: usize) -> u8
    {
        self.data.read_u8(offs)
    }
    
    pub fn read_u16(&self, offs: usize) -> u16
    {
        self.data.read_u16(offs)
    }
    
    pub fn read_u32(&self, offs: usize) -> u32
    {
        self.data.read_u32(offs)
    }
    
    pub fn read_u64(&self, offs: usize) -> u64
    {
        self.data.read_u64(offs)
    }
    
    pub fn read_str(&self, offs: usize) -> String
    {
        self.data.read_str(offs)
    }
    
    pub fn write_u32(&self, offs: usize, val: u32)
    {
        self.data.write_u32(offs, val);
    }
    
    pub fn print(&self)
    {
        println!("Domain Payload:");
        println!("  Cmd: {}", self.cmd);
        println!("  Num Objs: {}", self.num_objs);
        println!("  Data Size: {}", self.data_size);
        println!("  Obj Id: {}", self.obj_id);
        println!("  Token: {}", self.token);
        
        self.data.print();
        
        println!("Domain Objs:");
        for obj in &self.objs
        {
            println!("  {}", obj);
        }
    }
}

pub struct HIPCDataPayload
{
    magic: u32,
    version: u32,
    command: u32, // also error
    token: u32,
    //data: Vec<u8>,
    data_ptr: u64,
    data_len: u16
}

impl HIPCDataPayload
{
    pub fn unpack(buf: u64, data_size: u16) -> HIPCDataPayload
    {
        let mut buf_data = buf + 16;
        /*let mut data: Vec<u8> = Vec::with_capacity((data_size-0x10) as usize);
        for i in 0..(data_size-0x10)
        {
            data.push(peek8(buf_data));
            buf_data += 1;
        }*/
        
        HIPCDataPayload
        {
            magic: peek32(buf),
            version: peek32(buf + 4),
            command: peek32(buf + 8),
            token: peek32(buf + 12),
            data_ptr: buf + 16,
            data_len: (data_size - 16)
            //data: data
        }
    }
    
    pub fn packed_size(&self) -> u64
    {
        16 + self.data_len as u64 //self.data.len() as u64
    }
    
    pub fn get_cmd_id(&self) -> u32
    {
        self.command
    }
    
    pub fn read_u8(&self, offs: usize) -> u8
    {
        peek8(self.data_ptr + (offs as u64))
        //self.data[offs]
    }
    
    pub fn read_u16(&self, offs: usize) -> u16
    {
        /*let mut bytes: [u8; 2] = [0;2];
        for i in offs..offs+2
        {
            bytes[i] = self.data[i];
        }
        u16::from_le_bytes(bytes)*/
        peek16(self.data_ptr + (offs as u64))
    }
    
    pub fn read_u32(&self, offs: usize) -> u32
    {
        /*let mut bytes: [u8; 4] = [0;4];
        for i in offs..offs+4
        {
            bytes[i] = self.data[i];
        }
        u32::from_le_bytes(bytes)*/
        peek32(self.data_ptr + (offs as u64))
    }
    
    pub fn read_u64(&self, offs: usize) -> u64
    {
        /*let mut bytes: [u8; 8] = [0;8];
        for i in offs..offs+8
        {
            bytes[i] = self.data[i];
        }
        u64::from_le_bytes(bytes)*/
        peek64(self.data_ptr + (offs as u64))
    }
    
    pub fn read_str(&self, offs: usize) -> String
    {
        let mut s_len = 0;
        for i in offs..(self.data_len as usize)
        {
            if peek8(self.data_ptr + i as u64) == 0
            {
                break;
            }
            s_len += 1;
        }
        unsafe { String::from(str_from_null_terminated_utf8_u64ptr_unchecked_len(self.data_ptr + (offs as u64), s_len)) }
    }
    
    pub fn write_u32(&self, offs: usize, val: u32)
    {
        poke32(self.data_ptr + (offs as u64), val);
    }
    
    pub fn print(&self)
    {
        println!("Data Payload:");
        println!("  Magic: {:x}", self.magic);
        println!("  Version: {}", self.version);
        println!("  Command/Error: {:x}", self.command);
        println!("  Token: {:x}", self.token);
        //hexdump_vec("Data Buf", &self.data);
        // TODO data hexdump
    }
}

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

        let mut copy_handles: Vec<u32> = Vec::with_capacity(num_copy as usize);
        for i in 0..num_copy
        {
            let handle = peek32(buf_inc);
            buf_inc += 4;
            copy_handles.push(handle);
        }
        
        let mut move_handles: Vec<u32> = Vec::with_capacity(num_move as usize);
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
    
    pub fn get_handle(&self, idx: usize) -> Option<u32>
    {
        if idx < self.copy_handles.len()
        {
            return Some(self.copy_handles[idx]);
        }
        else if idx - self.copy_handles.len() < self.move_handles.len()
        {
            return Some(self.move_handles[idx - self.copy_handles.len()]);
        }
        return None;
    }
    
    pub fn print(&self)
    {
        println!("Handle Desc:");
        if self.send_pid
        {
            println!("  PID: {}", self.pid);
        }
        println!("  Copied ({}):", self.num_copy);
        for i in 0..(self.num_copy as usize)
        {
            println!("    {:x}", self.copy_handles[i]);
        }
        println!("  Moved  ({}):", self.num_move);
        for i in 0..(self.num_move as usize)
        {
            println!("    {:x}", self.move_handles[i]);
        }
    }
}

#[derive(Copy, Clone)]
pub struct HIPCStaticDesc
{
    buf: u64,
    pub index: u16,
    pub addr: u64,
    pub size: u16,
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
            buf: buf,
            index: index,
            addr: addr,
            size: size
        }
    }
    
    pub fn pack(&self)
    {
        let index5to0 = (self.index & 0x3F) as u32;
        let addr38to36 = ((self.addr >> 36) & 0x7) as u32;
        let index11to9 = ((self.index >> 9) & 0x7) as u32;
        let addr35to32 = ((self.addr >> 32) & 0xF) as u32;
        let addr31to0 = (self.addr & 0xFFFFFFFF) as u32;

        let word0 = ((self.size as u32) << 16) | (addr38to36 << 6) | (index11to9 << 9) | (addr35to32 << 12) | (index5to0);
        
        poke32(self.buf, word0);
        poke32(self.buf, addr31to0);
    }
    
    pub const fn packed_size(&self) -> u64
    {
        8
    }
    
    pub fn read_str(&self) -> String
    {
        String::from(kstr_len!(self.addr, self.size))
    }
    
    pub fn read_str_at(&self, offs: u64) -> String
    {
        if (offs as u16) > self.size {
            return String::from("");
        }
        return String::from(kstr_len!(self.addr + offs, self.size - offs as u16));
    }
    
    pub fn put_str_at(&mut self, offs: u64, strval: String)
    {
        if (offs as u16) > self.size {
            return;
        }
        let bytes = strval.into_bytes();
        let addr_el2 = translate_el1_stage12(self.addr);
        for i in 0..bytes.len()
        {
            poke8(addr_el2 + offs + (i as u64), bytes[i]);
        }
        poke8(addr_el2 + offs + bytes.len() as u64, 0);
        //self.size = 1 + bytes.len() as u16;
    }
    
    pub fn get_addr_el2(&self) -> u64
    {
        return translate_el1_stage12(self.addr);
    }
}

pub struct HIPCSendRecvExchDesc
{
    pub addr: u64,
    pub size: u64,
    pub flags: u8
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
    
    pub fn read_u32(&self, offs: usize) -> u32
    {
        peek32(translate_el1_stage12(self.addr + (offs as u64)))
    }
    
    pub fn read_u64(&self, offs: usize) -> u64
    {
        peek64(translate_el1_stage12(self.addr + (offs as u64)))
    }
    
    pub fn read_str(&self, offs: usize) -> String
    {
        String::from(kstr_len!(self.addr + (offs as u64), self.size - (offs as u64)))
    }
    
    pub fn is_ascii(&self) -> bool
    {
        let hyp_addr = translate_el1_stage12(self.addr);
        for i in 0..(self.size as u64)
        {
            let val = peek8(hyp_addr + i);
            
            if val == 0 {
                break;
            }
            
            if (val >= 1 && val <= 0x20) || (val >= 0x7f) {
                return false;
            }
        }
        return true;
    }
    
    pub fn get_addr_el2(&self) -> u64
    {
        return translate_el1_stage12(self.addr);
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
    
    static_desc_start: u64,
    send_desc_start: u64,
    recv_desc_start: u64,
    exch_desc_start: u64,
    
    /*static_descs: Vec<HIPCStaticDesc>,
    send_descs: Vec<HIPCSendRecvExchDesc>,
    recv_descs: Vec<HIPCSendRecvExchDesc>,
    exch_descs: Vec<HIPCSendRecvExchDesc>,*/
    
    data_payload: HIPCPayload,
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
        let enable_handle = ((word1 & bit!(31)) != 0);
        
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
        let static_desc_start = read_ptr;
        read_ptr += 8 * num_static as u64;
        
        // Unpack Send descriptors
        let send_desc_start = read_ptr;
        read_ptr += 12 * num_send as u64;
        
        // Unpack Recv descriptors
        let recv_desc_start = read_ptr;
        read_ptr += 12 * num_recv as u64;
        
        // Unpack Exchange descriptors
        let exch_desc_start = read_ptr;
        read_ptr += 12 * num_exch as u64;
        
        let hipc_payload: HIPCPayload;

        read_ptr = ((read_ptr + 0xF) & !0xF); // align to 0x10
        let magic = peek32(read_ptr);
        if magic == MAGIC_SFCI || magic == MAGIC_SFCO
        {
            let payload = HIPCDataPayload::unpack(read_ptr, data_size * 4);
            read_ptr += payload.packed_size();
            
            hipc_payload = HIPCPayload::Session(payload);
        }
        else
        {
            let payload = HIPCDomainPayload::unpack(read_ptr, data_size * 4);
            read_ptr += payload.packed_size();
            
            hipc_payload = HIPCPayload::Domain(payload);
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
            static_desc_start: static_desc_start,
            send_desc_start: send_desc_start,
            recv_desc_start: recv_desc_start,
            exch_desc_start: exch_desc_start,
            data_payload: hipc_payload,
        }
    }
    
    pub fn is_domain(&self) -> bool
    {
        match &self.data_payload
        {
            HIPCPayload::Session(session) => {
                false
            },
            HIPCPayload::Domain(domain) => {
                true
            }
        }
    }
    
    pub fn get_cmd_id(&self) -> u32
    {
        match &self.data_payload
        {
            HIPCPayload::Session(session) => {
                session.get_cmd_id()
            },
            HIPCPayload::Domain(domain) => {
                domain.get_cmd_id()
            }
        }
    }
    
    pub fn get_domain_cmd(&self) -> u8
    {
        match &self.data_payload
        {
            HIPCPayload::Session(session) => {
                0
            },
            HIPCPayload::Domain(domain) => {
                domain.get_domain_cmd()
            }
        }
    }
    
    pub fn get_domain_id(&self) -> u32
    {
        match &self.data_payload
        {
            HIPCPayload::Session(session) => {
                0
            },
            HIPCPayload::Domain(domain) => {
                domain.get_domain_id()
            }
        }
    }
    
    pub fn get_domain_obj(&self, idx: usize) -> Option<u32>
    {
        match &self.data_payload
        {
            HIPCPayload::Session(session) => {
                None
            },
            HIPCPayload::Domain(domain) => {
                domain.get_domain_obj(idx)
            }
        }
    }
    
    pub fn read_u8(&self, offs: usize) -> u8
    {
        match &self.data_payload
        {
            HIPCPayload::Session(session) => {
                session.read_u8(offs)
            },
            HIPCPayload::Domain(domain) => {
                domain.read_u8(offs)
            }
        }
    }
    
    pub fn read_u16(&self, offs: usize) -> u16
    {
        match &self.data_payload
        {
            HIPCPayload::Session(session) => {
                session.read_u16(offs)
            },
            HIPCPayload::Domain(domain) => {
                domain.read_u16(offs)
            }
        }
    }
    
    pub fn read_u32(&self, offs: usize) -> u32
    {
        match &self.data_payload
        {
            HIPCPayload::Session(session) => {
                session.read_u32(offs)
            },
            HIPCPayload::Domain(domain) => {
                domain.read_u32(offs)
            }
        }
    }
    
    pub fn read_u64(&self, offs: usize) -> u64
    {
        match &self.data_payload
        {
            HIPCPayload::Session(session) => {
                session.read_u64(offs)
            },
            HIPCPayload::Domain(domain) => {
                domain.read_u64(offs)
            }
        }
    }
    
    pub fn read_str(&self, offs: usize) -> String
    {
        match &self.data_payload
        {
            HIPCPayload::Session(session) => {
                session.read_str(offs)
            },
            HIPCPayload::Domain(domain) => {
                domain.read_str(offs)
            }
        }
    }
    
    pub fn write_u32(&self, offs: usize, val: u32)
    {
        match &self.data_payload
        {
            HIPCPayload::Session(session) => {
                session.write_u32(offs, val);
            },
            HIPCPayload::Domain(domain) => {
                domain.write_u32(offs, val);
            }
        }
    }
    
    pub fn get_handle(&self, idx: usize) -> Option<u32>
    {
        if let Some(desc) = &self.handle_desc {
            return desc.get_handle(idx);
        }
        return None;
    }
    
    pub fn get_static(&self, idx: usize) -> Option<HIPCStaticDesc>
    {
        if idx < self.num_static as usize {
            return Some(HIPCStaticDesc::unpack(self.static_desc_start + (idx*8) as u64));
        }
        return None;
    }
    
    pub fn get_send(&self, idx: usize) -> Option<HIPCSendRecvExchDesc>
    {
        if idx < self.num_send as usize {
            return Some(HIPCSendRecvExchDesc::unpack(self.send_desc_start + (idx*12) as u64));
        }
        return None;
    }
    
    pub fn get_recv(&self, idx: usize) -> Option<HIPCSendRecvExchDesc>
    {
        if idx < self.num_recv as usize {
            return Some(HIPCSendRecvExchDesc::unpack(self.recv_desc_start + (idx*12) as u64));
        }
        return None;
    }
    
    pub fn get_exch(&self, idx: usize) -> Option<HIPCSendRecvExchDesc>
    {
        if idx < self.num_exch as usize {
            return Some(HIPCSendRecvExchDesc::unpack(self.exch_desc_start + (idx*12) as u64));
        }
        return None;
    }
    
    pub fn hook_first_handle(&self, session_handle: u32, handler: HClientSessionHandler) -> bool
    {
        if let Some(mut hsession) = hipc_get_handle_clientsession(session_handle)
        {
            // HOS signals for a process to only use domains by returning a domain pkt w/ cmd 0?
            if self.is_domain() && self.get_domain_cmd() == 1
            {
                // Domain obj out
                let obj = self.read_u32(0);
            
                let hsession_locked = hsession.lock();
                let conv = hsession_locked.convert_to_domain(session_handle, obj);
                
                // Registration pair
                let domain_obj = conv.0;
                let mut domain_sess = conv.1;
                
                domain_sess.set_handler(handler);
                
                hipc_register_domain(domain_obj, Arc::new(Mutex::new(domain_sess)));
                return true;
            }
            else
            {
                if let Some(handle) = self.get_handle(0) {
                    // TODO: Copied handles may not actually belong to parent
                    let mut service_hsession = hsession.lock().new_from_parent();
                    
                    service_hsession.set_handler(handler);
                    
                    // Link new HClientSession to HOS handle
                    hipc_register_handle_clientsession(handle, Arc::new(Mutex::new(service_hsession)));
                    return true;
                }
            }
        }
        return false;
    }
    
    pub fn get_first_handle_obj(&self, session_handle: u32) -> Option<HObject>
    {
        if let Some(mut hsession) = hipc_get_handle_clientsession(session_handle)
        {
            // HOS signals for a process to only use domains by returning a domain pkt w/ cmd 0?
            if self.is_domain() && self.get_domain_cmd() == 1
            {
                // Domain obj out
                let obj = self.read_u32(0);
            
                if let Some(mut hsession) = hipc_get_domain_session(HDomainObj::from_curpid(session_handle, obj))
                {
                    return Some(HObject::DomainSession(hsession.clone()));
                }
            }
            else
            {
                if let Some(handle) = self.get_handle(0) {
                    if let Some(mut hsession) = hipc_get_handle_clientsession(handle)
                    {
                        return Some(HObject::ClientSession(hsession.clone()));
                    }
                }
            }
        }
        return None;
    }
    
    pub fn get_type(&self) -> u16
    {
        self.pkt_type
    }
    
    pub fn pack(&self)
    {
        
    }
    
    pub fn print(&self)
    {
        println!("HIPCPacket:");
        println!("  Type: {}", self.pkt_type);
        println!("  Num Static: {}", self.num_static);
        println!("  Num Send: {}", self.num_send);
        println!("  Num Recv: {}", self.num_recv);
        println!("  Data Size: {}", self.data_size);
        println!("  RecvStatic Flags: {}", self.recv_static_flags);
        println!("  Enable Handle: {}", self.enable_handle);

        if let Some(desc) = &self.handle_desc {
            desc.print();
        }

        match &self.data_payload
        {
            HIPCPayload::Session(session) => {
                session.print();
            },
            HIPCPayload::Domain(domain) => {
                domain.print();
            }
        }
    }
}

pub fn hipc_get_packet() -> HIPCPacket
{
    HIPCPacket::unpack(translate_el1_stage12(get_tls_el0()))
}

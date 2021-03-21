/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */
 
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use crate::usbd::usbd::*;
use core::mem;
use core::str;
use crate::arm::threading::*;
use alloc::vec::Vec;
use crate::logger::*;
use crate::util::t210_reset;
use alloc::string::String;

pub const ACM_SEND_ENCAPSULATED_COMMAND: u8 = (0x00);
pub const ACM_GET_ENCAPSULATED_RESPONSE: u8 = (0x01);
pub const ACM_SET_COMM_FEATURE: u8 = (0x02);
pub const ACM_GET_COMM_FEATRUE: u8 = (0x03);
pub const ACM_CLEAR_COMM_FEATURE: u8 = (0x04);
pub const ACM_SET_LINE_ENCODING: u8 = (0x20);
pub const ACM_GET_LINE_ENCODING: u8 = (0x21);
pub const ACM_SET_CONTROL_LINE_STATE: u8 = (0x22);
pub const ACM_SEND_BREAK: u8 = (0x23);

pub const CDC_INTR_PKT_SIZE: u16 = (64);
pub const CDC_BULK_PKT_SIZE: u16 = (64);

#[repr(C, packed(1))]
struct AcmLineEncoding
{
    dter: u32,
    stop_bits: u8,
    parity: u8,
    data_bits: u8,
}

struct CdcExtraDescData
{
    classHeaderDesc: UsbDtClassHeaderFunc,
    classCallMgmtDesc: UsbDtClassCallMgmt,
    classAbstractControlDesc: UsbDtClassAbstractControl,
    classUnionFunctionDesc: UsbDtClassUnionFunction
}

pub struct CdcGadget
{
    isactive: bool,
    enabled: bool,
    lineState: u16,
    cdc_if0: u8,
    cdc_if1: u8,
    if0_epInterruptOut: u8,
    if0_epIn: u8,
    if1_epBulkOut: u8,
    if1_epBulkIn: u8,
    setup_hook_idx: usize,
    cmd_buf: String,
    
    cdc_if0_extraDesc: CdcExtraDescData,
}

impl CdcGadget
{
    pub const fn empty() -> Self
    {
        CdcGadget
        {
            isactive: false,
            enabled: false,
            lineState: 0,
            cdc_if0: 0xff,
            cdc_if1: 0xff,
            if0_epInterruptOut: 0xff,
            if0_epIn: 0xff,
            if1_epBulkOut: 0xff,
            if1_epBulkIn: 0xff,
            setup_hook_idx: usize::MAX,
            cmd_buf: String::new(),
            cdc_if0_extraDesc: CdcExtraDescData
            {
                classHeaderDesc: UsbDtClassHeaderFunc
                {
                    bFunctionLength: 5,
                    bDescriptorType: CS_INTERFACE,
                    bDescriptorSubtype: USB_ST_HEADER,
                    bcdCDC: 0x110
                },
                classCallMgmtDesc: UsbDtClassCallMgmt
                {
                    bFunctionLength: 5,
                    bDescriptorType: CS_INTERFACE,
                    bDescriptorSubtype: USB_ST_CMF,
                    bmCapabilities: 0,
                    bDataInterface: 0,
                },
                classAbstractControlDesc: UsbDtClassAbstractControl
                {
                    bFunctionLength: 4,
                    bDescriptorType: CS_INTERFACE,
                    bDescriptorSubtype: USB_ST_ACMF,
                    bmCapabilities: 0,
                },
                classUnionFunctionDesc: UsbDtClassUnionFunction
                {
                    bFunctionLength: 5,
                    bDescriptorType: CS_INTERFACE,
                    bDescriptorSubtype: USB_ST_UF,
                    bMasterInterface: 0,
                    bSlaveInterface0: 0,
                }
            }
        }
    }
}

// This doesn't really matter, it just wants something valid
// if it asks
const cdc_lineEncResp: AcmLineEncoding = AcmLineEncoding
{
    dter: 115200,
    stop_bits: 0x00,
    parity: 0x00,
    data_bits: 0x08
};

static mut CDC: CdcGadget = CdcGadget::empty();

pub fn cdc_process_cmd()
{
    let cdc = get_cdc();
    
    if (cdc.cmd_buf == "rcm")
    {
        unsafe {t210_reset();}
        loop {}
    }
    else if (cdc.cmd_buf == "irqshow")
    {
        //irq_show();
    }
    else
    {
        println!("> Unknown command `{}`", cdc.cmd_buf);
    }
    
    cdc.cmd_buf.clear();
}

pub fn cdc_disable()
{
    //mutexLock(&cdc_send_mutex);
    let cdc = get_cdc();
    cdc.enabled = false;
    //mutexUnlock(&cdc_send_mutex);
}

pub fn cdc_enable()
{
    //mutexLock(&cdc_send_mutex);
    let cdc = get_cdc();
    cdc.enabled = true;
    //mutexUnlock(&cdc_send_mutex);
}

pub fn cdc_active() -> bool
{
    let cdc = get_cdc();
    return cdc.isactive;
}

pub fn cdc_send(usbd: &mut UsbDevice, data: &[u8], len: usize)
{
    let cdc = get_cdc();
    
    if (!cdc.isactive) { return; }

    let is_enabled = cdc.enabled && cdc.isactive && (get_core() == 0);

    if (len == 0)
    {
        //mutexUnlock(&cdc_send_mutex);
        return;
    }

    if (is_enabled /*&& mutexTryLock(&cdc_usb_mutex)*/)
    {
        let mut bytes_to_send = len;
        //mutexUnlock(&cdc_send_mutex);

        let mut i = 0;
        loop
        {
            if (i >= bytes_to_send) { break; }
            
            let mut to_send = bytes_to_send;
            if (to_send > 512) {
                to_send = 512;
            }
            if(usbd.ep_tx(cdc.if1_epBulkIn, to_u64ptr!(&data[0]) + (i as u64), to_send, true) != UsbdError::Success) {
                break;
            }
            bytes_to_send -= to_send;
            i += 512;
        }
        //mutexUnlock(&cdc_usb_mutex);

        return;
    }
    //mutexUnlock(&cdc_send_mutex);
}

pub fn cdc_if1_recvcomplete(usbd: &mut UsbDevice, epNum: u8)
{
    unsafe
    {
    let cdc = get_cdc();

    let p_pkt_data = usbd.get_xferbuf(cdc.if1_epBulkOut);
    let pkt_data: *mut u8 = p_pkt_data as _;
    let len = usbd.get_bytes_received(cdc.if1_epBulkOut);
    
    let mut to_send: Vec<u8> = Vec::with_capacity(CDC_BULK_PKT_SIZE as usize);
    let p_to_send = to_u64ptr!(to_send.as_mut_ptr());
    
    // Convert the strings or whatever
    for i in 0..(len as usize)
    {
        let val = pkt_data.offset(i as isize).read();
        cdc.cmd_buf.push(val as char);
        if (val == '\r' as u8)
        {
            cdc.cmd_buf.pop();
            cdc_process_cmd();
        }

        if (val == '\r' as u8)
        {
            to_send.push('\n' as u8);
        }
        to_send.push(val);
    }
    
    to_send.push(0);
    //printf("%s", to_send);

    // Send our data
    let old_en = cdc.enabled;
    cdc.enabled = true;
    cdc_send(usbd, to_send.as_slice() as &[u8], to_send.len());
    cdc.enabled = old_en;
    
    usbd.ep_idle(cdc.if1_epBulkOut);
    usbd.ep_txfer_start(cdc.if1_epBulkOut, CDC_BULK_PKT_SIZE as usize, false);
    }
}

pub fn cdc_if1_sendcomplete(usbd: &mut UsbDevice, epNum: u8)
{
    let cdc = get_cdc();

    // Ask for more data
    usbd.ep_idle(cdc.if1_epBulkOut);
    usbd.ep_txfer_start(cdc.if1_epBulkOut, CDC_BULK_PKT_SIZE as usize, false);
}

pub fn cdc_setup_hook(usbd: &mut UsbDevice, pkt: UsbSetupPacket) -> bool
{
    let cdc = get_cdc();

    if (pkt.bmRequestType == UsbSetupRequestType::DEV2HOST_INTERFACE_CLASS as u8)
    {
        // Apparently Windows sends it here?
        if (pkt.bRequest == ACM_GET_LINE_ENCODING)
        {
            usbd.setup_transact(pkt, to_u64ptr!(&cdc_lineEncResp), mem::size_of::<AcmLineEncoding>());
            return true;
        }
    }
    else if (pkt.bmRequestType == UsbSetupRequestType::HOST2DEV_INTERFACE_CLASS as u8)
    {
        if (pkt.bRequest == ACM_SET_LINE_ENCODING
            || pkt.bRequest == ACM_SEND_ENCAPSULATED_COMMAND
            || pkt.bRequest == ACM_GET_ENCAPSULATED_RESPONSE
            || pkt.bRequest == ACM_SET_CONTROL_LINE_STATE)
        {            
            // We don't do anything with is, but I don't think it's good to leave
            // the rest of the transaction unread
            if (pkt.wLength != 0)
            {
                usbd.setup_getdata(0, pkt.wLength);
            }
            
            // ACK
            usbd.setup_ack();
            
            
            
            if (pkt.bRequest == ACM_SET_LINE_ENCODING && !cdc.isactive)
            {
                // start transacting
                usbd.ep_txfer_start(cdc.if0_epInterruptOut, CDC_INTR_PKT_SIZE as usize, false);
                usbd.ep_txfer_start(cdc.if1_epBulkOut, CDC_BULK_PKT_SIZE as usize, false);
            
                cdc.isactive = true;
                
                cdc.cmd_buf.clear();
            }
            
            if (pkt.bRequest == ACM_SET_CONTROL_LINE_STATE)
            {
                cdc.lineState = pkt.wValue;
                
                cdc.isactive = true;
                let old_en = cdc.enabled;
                cdc.enabled = true;
                cdc_send(usbd, b"\n\r\n\r\n\r\x1b[2J", 6);
                cdc.enabled = old_en;
            }
            else if (pkt.bRequest == ACM_SET_LINE_ENCODING /*&& cdc.lineState == 3*/)
            {
                
            }

            return true;
        }
        else if (pkt.bRequest == ACM_GET_LINE_ENCODING)
        {
            usbd.setup_transact(pkt, to_u64ptr!(&cdc_lineEncResp), mem::size_of::<AcmLineEncoding>());
            return true;
        }
    }
    else if (pkt.bmRequestType == UsbSetupRequestType::HOST2DEV_ENDPOINT as u8)
    {
        if (pkt.bRequest == ACM_GET_ENCAPSULATED_RESPONSE)
        {
            // Just ignore it for now
            usbd.setup_ack();
            return true;
        }
    }
    
    return false;
}

pub fn cdc_reset_hook(usbd: &mut UsbDevice)
{
    let cdc = get_cdc();
    
    cdc.isactive = false;
    cdc.enabled = false;
    cdc.lineState = 0;
    cdc.cmd_buf.clear();
}

pub fn get_cdc() -> &'static mut CdcGadget
{
    unsafe
    {
        &mut CDC
    }
}

pub fn cdc_init()
{
    /*UsbDtClassHeaderFunc* classHeaderDesc = &cdc_if0_extraDesc.classHeaderDesc;
    UsbDtClassCallMgmt* classCallMgmtDesc = &cdc_if0_extraDesc.classCallMgmtDesc;
    UsbDtClassAbstractControl* classAbstractControlDesc = &cdc_if0_extraDesc.classAbstractControlDesc;
    UsbDtClassUnionFunction* classUnionFunctionDesc = &cdc_if0_extraDesc.classUnionFunctionDesc;*/
    
    let usbd = get_usbd();
    let cdc = get_cdc();
    
    // We allocate two interfaces, one has an interrupt EP (unused?) 
    // and the other has two bulk endpoints for each direction
    cdc.cdc_if0 = usbd.create_interface(2);
    cdc.cdc_if1 = usbd.create_interface(2);
    
    // We associate the former interface w/ the latter
    // (adds a device descriptor associating the two)
    usbd.get_interface(cdc.cdc_if0).associatedNum = 2;
    
    // Required metadata
    cdc.cdc_if0_extraDesc.classCallMgmtDesc.bDataInterface = usbd.get_interface(cdc.cdc_if1).interfaceNumber;
    
    cdc.cdc_if0_extraDesc.classUnionFunctionDesc.bMasterInterface = usbd.get_interface(cdc.cdc_if0).interfaceNumber;
    cdc.cdc_if0_extraDesc.classUnionFunctionDesc.bSlaveInterface0 = usbd.get_interface(cdc.cdc_if1).interfaceNumber;

    // Interface0 info
    usbd.get_interface(cdc.cdc_if0).class = 2; // CDC
    usbd.get_interface(cdc.cdc_if0).subclass = 2; // ACM
    usbd.get_interface(cdc.cdc_if0).protocol = 1; // V25TER
    usbd.get_interface(cdc.cdc_if0).extra_desc_data = to_u64ptr!(&cdc.cdc_if0_extraDesc);
    usbd.get_interface(cdc.cdc_if0).extra_desc_size = mem::size_of::<CdcExtraDescData>();
    
    // Interface1 data
    usbd.get_interface(cdc.cdc_if1).class = 10; // CDC data
    usbd.get_interface(cdc.cdc_if1).subclass = 0;
    usbd.get_interface(cdc.cdc_if1).protocol = 0;

    // Set up if0 endpoints (in direction is disabled)
    cdc.if0_epInterruptOut = usbd.get_interface(cdc.cdc_if0).endpointStart+0;
    cdc.if0_epIn = usbd.get_interface(cdc.cdc_if0).endpointStart+1;
    usbd.get_ep(cdc.if0_epInterruptOut).ep_construct(CDC_INTR_PKT_SIZE, USB_EPATTR_TTYPE_INTR, 9);
    usbd.get_ep(cdc.if0_epIn).ep_construct(0, USB_EPATTR_TTYPE_INTR, 0);
    
    // Set up if1 endpoints
    cdc.if1_epBulkOut = usbd.get_interface(cdc.cdc_if1).endpointStart+0;
    cdc.if1_epBulkIn = usbd.get_interface(cdc.cdc_if1).endpointStart+1;
    usbd.get_ep(cdc.if1_epBulkOut).ep_construct(CDC_BULK_PKT_SIZE, USB_EPATTR_TTYPE_BULK, 0);
    usbd.get_ep(cdc.if1_epBulkIn).ep_construct(CDC_BULK_PKT_SIZE, USB_EPATTR_TTYPE_BULK, 0);
    
    // Register all of our handlers
    usbd.register_handler(cdc.if1_epBulkOut, cdc_if1_recvcomplete);
    usbd.register_handler(cdc.if1_epBulkIn, cdc_if1_sendcomplete);
    cdc.setup_hook_idx = usbd.register_setup_hook(cdc_setup_hook);

    //mutexInit(&cdc_send_mutex);
    //mutexInit(&cdc_usb_mutex);
    
    usbd.register_reset_hook(cdc_reset_hook);
}

pub fn cdc_fini()
{
    let usbd = get_usbd();
    let cdc = get_cdc();
    
    usbd.remove_setup_hook(cdc.setup_hook_idx);
    usbd.remove_handler(cdc.if1_epBulkOut);
    usbd.remove_handler(cdc.if1_epBulkIn);
}

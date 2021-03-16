/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use crate::util::*;
use crate::io::car::*;
use crate::io::timer::*;
use crate::io::pmc::*;
use core::mem;

pub const USB2D_BASE: u32 = (0x7D000000);

pub const USB2D_USBCMD:             u32 = (0x4C);     // 0x130
pub const USB2D_USBSTS:             u32 = (0x4D);     // 0x134
pub const USB2D_USBINTR:            u32 = (0x4E);     // 0x138
pub const USB2D_PERIODICLISTBASE:   u32 = (0x51);     // 0x144
pub const USB2D_ASYNCLISTADDR:      u32 = (0x52);     // 0x148
pub const USB2D_HOSTPC1_DEVLC:      u32 = (0x6D);     // 0x1B4
pub const USB2D_OTGSC:              u32 = (0x7D);     // 0x1F4
pub const USB2D_USBMODE:            u32 = (0x7E);     // 0x1F8
pub const USB2D_ENDPTNAK:           u32 = (0x80);     // 0x200
pub const USB2D_ENDPTSETUPSTAT:     u32 = (0x82);     // 0x208
pub const USB2D_ENDPTPRIME:         u32 = (0x83);     // 0x20C
pub const USB2D_ENDPTFLUSH:         u32 = (0x84);     // 0x210
pub const USB2D_ENDPTSTAT:          u32 = (0x85);     // 0x214
pub const USB2D_ENDPTCOMPLETE:      u32 = (0x86);     // 0x218
pub const USB_SUSP_CTRL:            u32 = (0x100);    // 0x400
pub const USB_ULPIS2S_CTRL:         u32 = (0x101);    // 0x404
pub const USB1_UTMIP_XCVR_CFG0:     u32 = (0x202);    // 0x808
pub const USB1_UTMIP_BIAS_CFG0:     u32 = (0x203);    // 0x80C
pub const USB1_UTMIP_HSRX_CFG0:     u32 = (0x204);    // 0x810
pub const USB1_UTMIP_HSRX_CFG1:     u32 = (0x205);    // 0x814
pub const USB1_UTMIP_TX_CFG0:       u32 = (0x208);    // 0x820
pub const USB1_UTMIP_MISC_CFG1:     u32 = (0x20A);    // 0x828
pub const USB1_UTMIP_DEBOUNCE_CFG0: u32 = (0x20B);    // 0x82C
pub const USB1_UTMIP_SPARE_CFG0:    u32 = (0x20D);    // 0x834
pub const USB1_UTMIP_XCVR_CFG1:     u32 = (0x20E);    // 0x838
pub const USB1_UTMIP_BIAS_CFG1:     u32 = (0x20F);    // 0x83C
pub const USB1_UTMIP_BIAS_CFG2:     u32 = (0x214);    // 0x850
pub const USB1_UTMIP_XCVR_CFG2:     u32 = (0x215);    // 0x854
pub const USB1_UTMIP_XCVR_CFG3:     u32 = (0x216);    // 0x858
pub const USB2D_QH_EP_n_OUT:        u32 = (0x400);    // 0x1000

// USB2D_HOSTPC1_DEVLC
pub const USB2D_PSPD_MASK:  u32 = (bit!(26) | bit!(25));
pub const USB2D_PSPD_SHIFT: u32 = (25);
pub const DEVLC_AUTOLP:     u32 = (17);

// USB2D_OTGSC
pub const OTGSC_AVV: u32 = (bit!(9)); // A vbus valid

// USB2D_USBMODE
pub const USBMODE_CM_MASK:   u32 = (0x3);
pub const USBMODE_CM_DEVICE: u32 = (0x2);

// USB2D_USBSTS
pub const USBSTS_DCSUSPEND: u32 = bit!(8);
pub const USBSTS_USBSOF:    u32 = bit!(7);
pub const USBSTS_USBRST:    u32 = bit!(6);
pub const USBSTS_USBPORT:   u32 = bit!(2);
pub const USBSTS_USBINT:    u32 = bit!(0);

// USB2D_USBCMD
pub const USBCMD_FS2:     u32 = bit!(15);
pub const USBCMD_RST:     u32 = bit!(1);
pub const USBCMD_RUNSTOP: u32 = bit!(0);

// USB2D_ENDPTCTRL
pub const CTRL_TXT_MASK: u32 = (bit!(16+2) | bit!(16+3));
pub const CTRL_TXT_CTRL: u32 = (0);
pub const CTRL_TXT_BULK: u32 = (bit!(16+3));
pub const CTRL_TXT_INTR: u32 = (bit!(16+3) | bit!(16+3));
pub const CTRL_TXENABLE: u32 = (bit!(16+7));
pub const CTRL_TXRESET:  u32 = (bit!(16+6));
pub const CTRL_TXSTALL:  u32 = (bit!(16+0));
pub const CTRL_RXT_MASK: u32 = (bit!(2) | bit!(3));
pub const CTRL_RXT_CTRL: u32 = (0);
pub const CTRL_RXT_BULK: u32 = (bit!(3));
pub const CTRL_RXT_INTR: u32 = (bit!(3) | bit!(3));
pub const CTRL_RXENABLE: u32 = (bit!(7));
pub const CTRL_RXRESET:  u32 = (bit!(6));
pub const CTRL_RXSTALL:  u32 = (bit!(0));

// USB_SUSP_CTRL
pub const SUSPCTRL_UTMIP_PHY_ENB:     u32 = (bit!(12));
pub const SUSPCTRL_UTMIP_RESET:       u32 = (bit!(11));
pub const SUSPCTRL_USB_PHY_CLK_VALID: u32 = (bit!(7));

pub const USBD_EPNUM_MAX:      usize = (32);
pub const USBD_EPIDX_MAX:        u32 = (16);

pub const USBD_CTRL_PKT_MAX:     u16 = (64);

pub const USB_EPATTR_TTYPE_BULK: u32 = (2);
pub const USB_EPATTR_TTYPE_INTR: u32 = (3);

pub const CS_INTERFACE:  u32 = 0x24;
pub const USB_ST_HEADER: u32 = 0x00;
pub const USB_ST_CMF:    u32 = 0x01;
pub const USB_ST_ACMF:   u32 = 0x02;
pub const USB_ST_UF:     u32 = 0x06;

pub const USBD_XFERBUF_SIZE: u32 = (0x1000);

#[macro_use]
mod usbd {
    // 0x21c + n*4
    macro_rules! USB2D_ENDPTCTRL {
        ($n:expr) => {
            (0x87 + $n)
        }
    }
    
    macro_rules! USB_EPATTR_TTYPE {
        ($attr:expr) => {
            (attr & 0x3)
        }
    }
    
    macro_rules! USB2D_IS_EP_TX {
        ($ep:expr) => {
            ((($ep & bit!(0)) != 0))
        }
    }
    
    macro_rules! USB2D_EPIDX {
        ($ep:expr) => {
            (ep >> 1)
        }
    }
    
    macro_rules! USB2D_EPBIT {
        ($ep:expr) => {
            if ($ep == USB_EP_ALL) { 0xFFFFFFFF } else { if USB2D_IS_EP_TX!($ep) { bit!(16) << USB2D_EPIDX!($ep) } else { bit!(0) << USB2D_EPIDX!($ep) } }
        }
    }
    
    macro_rules! USB2D_EPADDR_IS_TX {
        ($epaddr:expr) => {
            ($epaddr & 0x80) != 0
        }
    }
    
    macro_rules! USB2D_EPADDR_TO_EPNUM {
        ($epaddr:expr) => {
            (($epaddr & 0x7F) << 1 | (if USB2D_EPADDR_IS_TX!($epaddr) { 1 } else { 0 }))
        }
    }
    
    macro_rules! USB2D_EPNUM_TO_EPADDR {
        ($epnum:expr) => {
            (($epnum >> 1) | (if USB2D_IS_EP_TX!($epnum) { 0 } else { 0x80 }))
        }
    }
    
    macro_rules! CTRL_EPT_MASK {
        ($ep:expr) => {
            (if USB2D_IS_EP_TX!($ep) { CTRL_TXT_MASK } else { CTRL_RXT_MASK })
        }
    }
    
    macro_rules! CTRL_EPT_CTRL {
        ($ep:expr) => {
            (if USB2D_IS_EP_TX!($ep) { CTRL_TXT_CTRL } else { CTRL_RXT_CTRL })
        }
    }
    
    macro_rules! CTRL_EPT_BULK {
        ($ep:expr) => {
            (if USB2D_IS_EP_TX!($ep) { CTRL_TXT_BULK } else { CTRL_RXT_BULK })
        }
    }
    
    macro_rules! CTRL_EPT_INTR {
        ($ep:expr) => {
            (if USB2D_IS_EP_TX!($ep) { CTRL_TXT_INTR } else { CTRL_RXT_INTR })
        }
    }
    
    macro_rules! CTRL_EPENABLE {
        ($ep:expr) => {
            (if USB2D_IS_EP_TX!($ep) { CTRL_TXENABLE } else { CTRL_RXENABLE })
        }
    }
    
    macro_rules! CTRL_EPRESET {
        ($ep:expr) => {
            (if USB2D_IS_EP_TX!($ep) { CTRL_TXRESET } else { CTRL_RXRESET })
        }
    }
    
    macro_rules! CTRL_EPSTALL {
        ($ep:expr) => {
            (if USB2D_IS_EP_TX!($ep) { CTRL_TXSTALL } else { CTRL_RXSTALL })
        }
    }
    
    // EpCapabilities
    macro_rules! EPCAP_MAX_PKT {
        ($n:expr) => {
            ($n << 16)
        }
    }
    
    // DtdInfo
    macro_rules! DTD_GETINFO_BYTES {
        ($val:expr) => {
            ($val >> 16)
        }
    }
    
    macro_rules! INFO_BYTES {
        ($n:expr) => {
            (($n & 0xFFFF) << 16)
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum UsbEpStatus
{
    // Transfer on the endpoint is completed
    UsbEpStatus_TxfrComplete = 0,
    // Transfer on the endpoint is still active
    UsbEpStatus_TxfrActive = 1,
    // Transfer on the endpoint failed
    UsbEpStatus_TxfrFail = 2,
    // Endpoint is idle, ready for new data transfers
    UsbEpStatus_TxfrIdle = 3,
    // Endpoint stalled
    UsbEpStatus_Stalled = 4,
    // Endpoint is not configured yet
    UsbEpStatus_NotConfigured = 5,
}

#[repr(i32)]
#[derive(Copy, Clone)]
pub enum UsbEpNum
{
    // Control Out endpoint number, mapped to ep0
    CTRL_OUT = 0,
    // Control In endpoint number, mapped to ep0
    CTRL_IN  = 1,
    
    // Bulk out endpoint number, mapped to ep1
    BULK_OUT = 2,
    // Bulk In endpoint number, mapped to ep1
    BULK_IN  = 3,
    
    // All endpoints
    EP_ALL      = -1,
}

// Start of dynamically configured endpoints
pub const USB_EP_CONFIGURABLE_BEGIN: i32 = 2;

#[repr(i32)]
#[derive(Copy, Clone)]
pub enum DeviceRequestTypes
{
    GET_STATUS        = 0,
    CLEAR_FEATURE     = 1,
    SET_FEATURE       = 3,
    SET_ADDRESS       = 5,
    GET_DESCRIPTOR    = 6,
    SET_DESCRIPTOR    = 7,
    GET_CONFIGURATION = 8,
    SET_CONFIGURATION = 9,
    GET_INTERFACE     = 10
}

#[repr(i32)]
#[derive(Copy, Clone)]
pub enum UsbDevDescriptorType
{
    USB_DT_DEVICE             = 1,
    USB_DT_CONFIG             = 2,
    USB_DT_STRING             = 3,
    USB_DT_INTERFACE          = 4,
    USB_DT_ENDPOINT           = 5,
    USB_DT_DEVICE_QUALIFIER   = 6,

    USB_DT_OTHER_SPEED_CONFIG = 7,
    USB_DT_INTERFACE_ASSOCIATION = 11,
}

#[repr(i32)]
#[derive(Copy, Clone)]
pub enum StringDescriptorIndex
{
    USB_LANGUAGE_ID = 0,
    USB_MANF_ID = 1,
    USB_PROD_ID = 2,
    USB_SERIAL_ID = 3
}

#[repr(i32)]
#[derive(Copy, Clone)]
pub enum UsbSetupRequestType
{
    HOST2DEV_DEVICE     = 0x00,
    HOST2DEV_INTERFACE  = 0x01,
    HOST2DEV_ENDPOINT   = 0x02,
    DEV2HOST_DEVICE     = 0x80,
    DEV2HOST_INTERFACE  = 0x81,
    DEV2HOST_ENDPOINT   = 0x82,
    
    // Class-specific
    DEV2HOST_INTERFACE_CLASS  = 0xA1,
    HOST2DEV_INTERFACE_CLASS  = 0x21,
}

#[repr(i32)]
#[derive(Copy, Clone)]
pub enum UsbEpAddress
{
    // Control out endpoint address
    CTRL_OUT = 0x00,
    // Control in endpoint address
    CTRL_IN  = 0x80,
    // Bulk out endpoint address
    BULK_OUT = 0x01,
    // Bulk in endpoint address
    BULK_IN  = 0x81,
}



#[repr(C)]
pub struct UsbSetupPacket
{
    bmRequestType: u8,
    bRequest: u8,
    wValue: u16,
    wIndex: u16,
    wLength: u16
}

#[repr(C)]
pub struct UsbControllerContext
{
    UsbPortSpeed: u32,
    UsbBaseAddr: u32, // vu32*
    setupPkt: UsbSetupPacket,
    EnumerationDone: bool,
    UsbControllerEnabled: bool,
    UsbConfigurationNo: u8,
    UsbInterfaceNo: u8,
    InitializationDone: bool
}

#[repr(C)]
pub struct UsbDevQueueHead
{
    EpCapabilities: u32,
    CurrentDTDPtr: u32,
    NextDTDPtr: u32,
    DtdInfo: u32,
    BufferPtrs: [u32; 5],
    Reserved: u32,
    setupBuffer: UsbSetupPacket,
    Reserved0: u32,
    Reserved1: u32,
    Reserved2: u32,
    Reserved3: u32,
}

// EpCapabilities
pub const EPCAP_ZLT:     u32 = (1 << 29);	// stop on zero-len xfer
pub const EPCAP_IOS:     u32 = (1 << 15);	// IRQ on setup
pub const EPCAP_HISPEED: u32 = (1 << 12);

#[repr(C)]
pub struct UsbTransDesc
{
    NextDtd: u32,
    DtdInfo: u32,
    BufPtrs: [u32; 5],
    Reserved: u32,
}

// DtdInfo
pub const INFO_IOC:          u32 = (bit!(15)); // interrupt on complete
pub const INFO_ACTIVE:       u32 = (bit!(7));
pub const INFO_HALTED:       u32 = (bit!(6));
pub const INFO_BUFFER_ERROR: u32 = (bit!(5));
pub const INFO_TX_ERROR:     u32 = (bit!(3));

#[repr(C)]
pub struct UsbDtDevice
{
    bLength: u8,
    bDescriptorType: u8,
    bcdUsb: u16,
    bDeviceClass: u8,
    bDeviceSubclass: u8,
    bDeviceProtocol: u8,
    bMaxPacketSize: u8,
    idVendor: u16,
    idProduct: u16,
    bcdDevice: u16,
    iManufacturer: u8,
    iProduct: u8,
    iSerialNumber: u8,
    bNumConfigurations: u8,
}

#[repr(C)]
pub struct UsbDtString
{
    bLength: u8,
    bDescriptorType: u8,
    data: [u16; 0x1F] // packets are max 0x40 here so just alloc up to that
}

#[repr(C)]
pub struct UsbDtConfig
{
    bLength: u8,
    bDescriptorType: u8,
    wTotalLength: u16,
    bNumInterfaces: u8,
    bConfigurationValue: u8,
    iConfiguration: u8,
    bmAttributes: u8,
    bMaxPower: u8,
}

#[repr(C)]
pub struct UsbDtInterface
{
    bLength: u8,
    bDescriptorType: u8,
    bInterfaceNumber: u8,
    bAlternateSetting: u8,
    bNumEndpoints: u8,
    bInterfaceClass: u8,
    bInterfaceSubclass: u8,
    bInterfaceProtocol: u8,
    iInterface: u8,
}

#[repr(C)]
pub struct UsbDtEndpoint
{
    bLength: u8,
    bDescriptorType: u8,
    bEndpointAddress: u8,
    bmAttributes: u8,
    wMaxPacketSize: u16,
    bInterval: u8,
}

#[repr(C)]
pub struct UsbDtClassHeaderFunc
{
    bFunctionLength: u8,
    bDescriptorType: u8,
    bDescriptorSubtype: u8,  /* 0x00 */
	bcdCDC: u16,
}

#[repr(C)]
pub struct UsbDtClassCallMgmt
{
    bFunctionLength: u8,
    bDescriptorType: u8,
    bDescriptorSubtype: u8,	/* 0x01 */
    bmCapabilities: u8,
    bDataInterface: u8,
}

#[repr(C)]
pub struct UsbDtClassAbstractControl
{
    bFunctionLength: u8,
    bDescriptorType: u8,
    bDescriptorSubtype: u8,	/* 0x02 */
    bmCapabilities: u8,
}

#[repr(C)]
pub struct UsbDtClassUnionFunction
{
    bFunctionLength: u8,
    bDescriptorType: u8,
    bDescriptorSubtype: u8,	/* 0x06 */
    bMasterInterface: u8,
    bSlaveInterface0: u8,
}

#[repr(C)]
pub struct UsbDtDeviceQualifier
{
    bLength: u8,
    bDescriptorType: u8,
    bcdUsb: u16,
    bDeviceClass: u8,
    bDeviceSubclass: u8,
    bDeviceProtocol: u8,
    bMaxPacketSize0: u8,
    bNumConfigurations: u8,
    bReserved: u8,
}

#[repr(C)]
pub struct UsbDtInterfaceAssociation
{
    bLength: u8,
    bDescriptorType: u8,
    bFirstInterface: u8,
    bInterfaceCount: u8,
    bFunctionClass: u8,
    bFunctionSubclass: u8,
    bFunctionProtocol: u8,
    iFunction: u8,
}

// Endpoint info struct
pub struct UsbdEndpoint
{
    isAssigned: bool,
    isEnabled: bool,
    handler: UsbdEpHandler,
    pDataTransDesc: u64,
    //UsbTransDesc* pDataTransDesc;
    epConfigured: u32,
    bytesRequested: u32,
    
    epNum: u8,
    epAddress: u8,
    attributes: u8,
    maxPacketSize: u16,
    interval: u8,
}

// Interface info struct
pub struct UsbdInterface
{
    class: u8,
    subclass: u8,
    protocol: u8,
    interfaceNumber: u8,
    
    //void* extra_desc_data;
    //size_t extra_desc_size;
    
    associatedNum: u8,
    
    numEndpoints: u8,
    //UsbdEndpoint* endpoints;
    
    //UsbdInterface* next;
}

#[repr(i32)]
#[derive(Copy, Clone, PartialEq)]
pub enum UsbdError
{
    Success = 0,
    HwTimeOut = 3,
}

pub struct UsbdEpHandler
{
}

struct UsbPllStruct1
{
    a: u16,
    b: u16,
    c: u16,
    d: u16,
}

struct UsbPllStruct2
{
    a: u32,
    b: u32,
    c: u32,
}

struct UsbPllStruct3
{
    a: u32,
    b: u32,
}

const usb_timing_related: [u8; 13] = [2, 2, 0, 0, 2, 6, 0, 0, 2, 6, 0, 0, 4];
const usbPllStruct1Arr: [UsbPllStruct1; 13] =
[
    UsbPllStruct1 {a: 2, b: 0x33, c: 9, d: 0x7F},
    UsbPllStruct1 {a: 3, b: 66, c: 11, d: 165},
    UsbPllStruct1 {a: 0, b: 0, c: 0, d: 0},
    UsbPllStruct1 {a: 0, b: 0, c: 0, d: 0},
    UsbPllStruct1 {a: 3, b: 75, c: 12, d: 188},
    UsbPllStruct1 {a: 5, b: 150, c: 24, d: 375},
    UsbPllStruct1 {a: 0, b: 0, c: 0, d: 0},
    UsbPllStruct1 {a: 0, b: 0, c: 0, d: 0},
    UsbPllStruct1 {a: 2, b: 47,  c: 8, d: 118},
    UsbPllStruct1 {a: 6, b: 188, c: 31, d: 469},
    UsbPllStruct1 {a: 0, b: 0, c: 0, d: 0},
    UsbPllStruct1 {a: 0, b: 0, c: 0, d: 0},
    UsbPllStruct1 {a: 4, b: 102, c: 17, d: 254},
];

const usbPllStruct3Arr: [UsbPllStruct3; 13] =
[
    UsbPllStruct3 {a: 74, b: 1},
    UsbPllStruct3 {a: 57, b: 1},
    UsbPllStruct3 {a: 0, b: 0},
    UsbPllStruct3 {a: 0, b: 0},
    UsbPllStruct3 {a: 50, b: 1},
    UsbPllStruct3 {a: 25, b: 1},
    UsbPllStruct3 {a: 0, b: 0},
    UsbPllStruct3 {a: 0, b: 0},
    UsbPllStruct3 {a: 80, b: 1},
    UsbPllStruct3 {a: 40, b: 2},
    UsbPllStruct3 {a: 0, b: 0},
    UsbPllStruct3 {a: 0, b: 0},
    UsbPllStruct3 {a: 74, b: 2},
];

const usb_pll_related: [u32; 13] = [ 32500, 42000, 0, 0, 48000, 48000, 0, 0, 30000, 60000, 0, 0, 65000 ];

//
// END TYPES/STRUCTS
//

const usbdEndpoint_init: UsbdEndpoint = UsbdEndpoint {
    isAssigned: false,
    isEnabled: false,
    handler: UsbdEpHandler {},
    pDataTransDesc: 0,
    
    epConfigured: 0,
    bytesRequested: 0,
    epNum: 0,
    epAddress: 0,
    attributes: 0,
    maxPacketSize: 0,
    interval: 0
};

pub struct UsbDevice
{
    endpoints: [UsbdEndpoint; USBD_EPNUM_MAX],
    initialized: bool,
    usbfCtxt: UsbControllerContext,
}

impl UsbdEndpoint
{
    pub fn ep_construct(&mut self, maxPktSize: u16, attributes: u8, interval: u8)
    {
        self.isAssigned = true;
        self.isEnabled = (maxPktSize > 0);
        self.maxPacketSize = maxPktSize;
        self.attributes = attributes;
        self.interval = interval;
    }
    
    pub fn ep_init(&mut self)
    {
        // TODO
    }
}

impl UsbDevice
{
    pub fn new() -> Self {
        let mut retval: UsbDevice = UsbDevice {
            endpoints: [usbdEndpoint_init; USBD_EPNUM_MAX],
            initialized: false,
            usbfCtxt: { unsafe { mem::zeroed() } },
        };
        
        return retval;
    }
    
    pub fn init(&mut self)
    {
        if (self.initialized)
        {
            return;
        }
        
        for i in 0..USBD_EPNUM_MAX
        {
            let i_u8: u8 = (i & 0xFF) as u8;
            self.endpoints[i].isAssigned = false;
            self.endpoints[i].isEnabled = false;
            self.endpoints[i].handler = UsbdEpHandler { };
            
            self.endpoints[i].epNum = i_u8;
            self.endpoints[i].epAddress = USB2D_EPNUM_TO_EPADDR!(i_u8);
            self.endpoints[i].pDataTransDesc = (0x80010000 + (i*mem::size_of::<UsbTransDesc>())) as u64;//memalign(0x20, sizeof(UsbTransDesc));
        }
        
        self.endpoints_resetall();
        
        // TODO ehh
        let usbd_epCtrlOut = &mut self.endpoints[UsbEpNum::CTRL_OUT as usize];
        usbd_epCtrlOut.ep_construct(USBD_CTRL_PKT_MAX, 0, 0);
        
        let usbd_epCtrlIn = &mut self.endpoints[UsbEpNum::CTRL_IN as usize];
        usbd_epCtrlIn.ep_construct(USBD_CTRL_PKT_MAX, 0, 0);
        
        self.initialized = true;
    }
    
    pub fn endpoints_resetall(&mut self)
    {
        for i in 0..USBD_EPNUM_MAX
        {
            self.endpoints[i].epConfigured = 0;
            self.endpoints[i].bytesRequested = 0;
            ////memset(usbd_endpoints[i].pDataTransDesc, 0, sizeof(UsbTransDesc));
        }  
    }
    
    pub fn init_context(&mut self)
    {
        // TODO: There's a bit more to it than just this in nvboot
        //self.usbfCtxt.UsbPortSpeed = 2;
        self.usbfCtxt.UsbBaseAddr = USB2D_BASE;
        self.usbfCtxt.EnumerationDone = false;
        self.usbfCtxt.UsbConfigurationNo = 1;
        self.usbfCtxt.UsbInterfaceNo = 0;
        self.usbfCtxt.InitializationDone = false;
        self.usbfCtxt.setupPkt = { unsafe { mem::zeroed() } };
        
        //usbd_enable_devicemode();
        
        self.endpoints_resetall();
    }
    
    pub fn w32(&mut self, offs: u32, val: u32)
    {
        poke32(self.usbfCtxt.UsbBaseAddr + offs, val);
    }
    
    pub fn r32(&mut self, offs: u32) -> u32
    {
        return peek32(self.usbfCtxt.UsbBaseAddr + offs);
    }
    
    pub fn echo32(&mut self, offs: u32)
    {
        let old: u32 = peek32(self.usbfCtxt.UsbBaseAddr + offs);
        poke32(self.usbfCtxt.UsbBaseAddr + offs, old);
    }
    
    pub fn or32(&mut self, offs: u32, val: u32)
    {
        let old: u32 = peek32(self.usbfCtxt.UsbBaseAddr + offs);
        poke32(self.usbfCtxt.UsbBaseAddr + offs, old | val);
    }
    
    pub fn and32(&mut self, offs: u32, val: u32)
    {
        let old: u32 = peek32(self.usbfCtxt.UsbBaseAddr + offs);
        poke32(self.usbfCtxt.UsbBaseAddr + offs, old & val);
    }
    
    pub fn enable_clocks(&mut self)
    {
        let OSC_FREQ: usize = (peek32(CLK_RST_CONTROLLER_OSC_CTRL) >> 28) as usize;
        //pll_init(0xC000CC, usb_pll_related_1[3 * OSC_FREQ + 1], usb_pll_related_1[3 * OSC_FREQ], usb_pll_related_1[3 * OSC_FREQ + 2], 0, 0, &v23);
        
        let mut clk_out_enb_l: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_CLK_OUT_ENB_L);
        let mut clk_out_enb_y: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_CLK_OUT_ENB_Y);
        let mut rst_dev_l_set: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_RST_DEV_L_SET);
        let mut rst_dev_l_clr: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_RST_DEV_L_CLR);
        let mut rst_dev_w_clr: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_RST_DEV_W_CLR);
        let mut utmipll_hw_pwrdn_cfg0: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_UTMIPLL_HW_PWRDN_CFG0);
        let mut utmip_pll_cfg0: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_UTMIP_PLL_CFG0);
        let mut utmip_pll_cfg1: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_UTMIP_PLL_CFG1);
        let mut utmip_pll_cfg2: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_UTMIP_PLL_CFG2);
        let mut clk_source_usb2_hsic_trk: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_CLK_SOURCE_USB2_HSIC_TRK);
        let mut apbdev_pmc_usb_ao: MMIOReg = MMIOReg::new(APBDEV_PMC_USB_AO);
        
        clk_out_enb_l |= CLK_ENB_USBD;
        timerWait(2);
        rst_dev_l_set |= CLK_ENB_USBD;
        timerWait(2);
        rst_dev_l_clr |= CLK_ENB_USBD;
        timerWait(2);
        rst_dev_w_clr |= XUSB_PADCTL_RST;
        timerWait(2);

        self.or32(USB_SUSP_CTRL, SUSPCTRL_UTMIP_RESET);
        self.or32(USB_SUSP_CTRL, SUSPCTRL_UTMIP_PHY_ENB);
        utmipll_hw_pwrdn_cfg0 &= 0xFFFFFFFD;

        timerWait(10);

        self.and32(USB1_UTMIP_MISC_CFG1, 0xBFFFFFFF);
        utmip_pll_cfg2 &= 0xBFFFFFFF;
        self.or32(USB_ULPIS2S_CTRL, 0x1000);
        self.or32(USB_ULPIS2S_CTRL, 0x800);

        utmip_pll_cfg0 &= 0xFF00FFFF;
        utmip_pll_cfg0 |= (usbPllStruct3Arr[OSC_FREQ].a << 16) & 0xFFFFFF;
        utmip_pll_cfg0 &= 0xFFFF00FF;
        utmip_pll_cfg0 |= (usbPllStruct3Arr[OSC_FREQ].b << 8) & 0xFFFF;

        utmip_pll_cfg2 &= (0xFFFC003F & 0xFF03FFFF);
        utmip_pll_cfg2 |= ((usbPllStruct1Arr[OSC_FREQ].c as u32) << 18) & 0xFFFFFF;
        utmip_pll_cfg1 &= 0x7FFF000;
        utmip_pll_cfg1 |= ((usbPllStruct1Arr[OSC_FREQ].d as u32) & 0xFFF) | 0x8000;
        utmip_pll_cfg1 &= 0xFFFFAFFF;
        
        for i in 0..10
        {
            if (utmipll_hw_pwrdn_cfg0 & bit!(31) != 0)
            {
                break;
            }

            timerWait(1);
        }

        let fuse_usb_calib: u32 = 0x08A0A415;//FUSE_USB_CALIB_0;
        let fuse_usb_calib_ext: u32 = 0x4;//FUSE_USB_CALIB_EXT_0;
        self.and32(USB1_UTMIP_XCVR_CFG0, 0xFFFF0000);
        self.or32(USB1_UTMIP_XCVR_CFG0, fuse_usb_calib & 0xF);
        self.and32(USB1_UTMIP_XCVR_CFG0, 0xFE3FFFFF);
        
        let mut fuse_idk: u32 = (fuse_usb_calib & 0x3F) << 25;
        fuse_idk >>= 29;
        fuse_idk <<= 22;
        self.or32(USB1_UTMIP_XCVR_CFG0, fuse_idk);
        
        fuse_idk = (fuse_usb_calib << 21);
        fuse_idk >>= 28;
        fuse_idk <<= 18;
        self.and32(USB1_UTMIP_XCVR_CFG1, 0xFFC3FFFF);
        self.or32(USB1_UTMIP_XCVR_CFG1, fuse_idk);
        self.and32(USB1_UTMIP_XCVR_CFG3, 0xFFFFC1FF);
        self.or32(USB1_UTMIP_XCVR_CFG3, ((fuse_usb_calib_ext & 0x1F) << 9));
        self.and32(USB1_UTMIP_XCVR_CFG0, 0xFFDFFFFF);
        self.and32(USB1_UTMIP_XCVR_CFG2, 0xFFFFF1FF);
        self.or32(USB1_UTMIP_XCVR_CFG2, 0x400);
        timerWait(1);
        self.and32(USB1_UTMIP_DEBOUNCE_CFG0, 0xFFFF0000);
        self.or32(USB1_UTMIP_DEBOUNCE_CFG0, (usb_pll_related[OSC_FREQ] & 0xFFFF));
        if (OSC_FREQ == 5 || OSC_FREQ == 9)
        {
            self.and32(USB1_UTMIP_BIAS_CFG1, 0xFFFFC0FF);
            self.or32(USB1_UTMIP_BIAS_CFG1, 0x100);
        }
        
        // patch 2, SVC 0x26
        self.w32(USB1_UTMIP_SPARE_CFG0, 0x7D000834);
        self.w32(USB1_UTMIP_BIAS_CFG2, 0x7D000850 | bit!(1));

        self.and32(USB1_UTMIP_SPARE_CFG0, !(0x198));
        // end
        
        self.or32(USB1_UTMIP_TX_CFG0, 0x80000);
        
        // patch 5, SVC 0x42
        self.w32(USB1_UTMIP_HSRX_CFG0, 0x7D000810);
        self.w32(USB1_UTMIP_BIAS_CFG2, 0x7D000850);

        self.and32(USB1_UTMIP_HSRX_CFG0, !(0xF8000));
        self.or32(USB1_UTMIP_HSRX_CFG0, 0x88000);
        self.and32(USB1_UTMIP_HSRX_CFG0, !(0x7C00));
        self.or32(USB1_UTMIP_HSRX_CFG0, 0x4000);
        self.and32(USB1_UTMIP_HSRX_CFG0, !(0xF000000));

        self.and32(USB1_UTMIP_BIAS_CFG2, !(0x07));
        // end
     
        self.and32(USB1_UTMIP_HSRX_CFG1, !0x3E);
        self.or32(USB1_UTMIP_MISC_CFG1, 0x12 | bit!(30));
        utmip_pll_cfg2 |= bit!(30);
        clk_out_enb_y |= bit!(18);
        clk_source_usb2_hsic_trk &= !0xFF;
        clk_source_usb2_hsic_trk |= usb_timing_related[OSC_FREQ] as u32;
        
        self.and32(USB1_UTMIP_BIAS_CFG1, !0xF8);
        self.and32(USB1_UTMIP_BIAS_CFG1, 0x3FC000);
        self.or32(USB1_UTMIP_BIAS_CFG1, 0x50 | 0x78000);
        
        self.and32(USB1_UTMIP_BIAS_CFG0, !bit!(10));
        timerWait(1);
        self.and32(USB1_UTMIP_BIAS_CFG1, !bit!(0));
        self.or32(USB1_UTMIP_BIAS_CFG1, bit!(1));
        timerWait(100);
        self.or32(USB1_UTMIP_BIAS_CFG1, bit!(0));
        self.and32(USB1_UTMIP_BIAS_CFG1, !bit!(23));
        timerWait(3);
        self.and32(USB1_UTMIP_BIAS_CFG1, !bit!(0));
        timerWait(100);
        self.or32(USB1_UTMIP_BIAS_CFG1, bit!(0));
        self.and32(USB1_UTMIP_BIAS_CFG1, !bit!(23));
        clk_out_enb_y &= !bit!(18);
        utmip_pll_cfg2 &= !(bit!(0) | bit!(4) | bit!(2) | bit!(24));
        utmip_pll_cfg2 |= bit!(1) | 0x28 | 0x2000000;
        timerWait(1);
        self.and32(USB1_UTMIP_BIAS_CFG0, 0xFF3FF7FF);
        timerWait(1);
        apbdev_pmc_usb_ao &= 0xFFFFFFF3;
        timerWait(1);
        self.and32(USB1_UTMIP_XCVR_CFG0, 0xFFFFBFFF);
        timerWait(1);
        self.and32(USB1_UTMIP_XCVR_CFG0, 0xFFFEFFFF);
        timerWait(1);
        self.and32(USB1_UTMIP_XCVR_CFG0, 0xFFFBFFFF);
        timerWait(1);
        self.and32(USB1_UTMIP_XCVR_CFG1, 0xFFFFFFFB);
        timerWait(1);
        self.and32(USB1_UTMIP_XCVR_CFG1, 0xFFFFFFEF);
        timerWait(1);
        
        if(self.enable_devicemode() != UsbdError::Success)
        {
            // TODO log
            //printf("timed out enabling device mode\n\r");
        }
        self.usbfCtxt.EnumerationDone = false;
    }

    pub fn enable_devicemode(&mut self) -> UsbdError
    {
        self.usbfCtxt.UsbControllerEnabled = false;
        self.and32(USB_SUSP_CTRL, !SUSPCTRL_UTMIP_RESET);
        
        let mut timed_out: bool = false;
        for i in 0..400000
        {
            timerWait(1);

            timed_out = (self.r32(USB_SUSP_CTRL) & SUSPCTRL_USB_PHY_CLK_VALID) == 0;
            if (!timed_out) { break; }
        }
        
        if (timed_out)
        {
            //printf("clock is invalid\n\r");
            return UsbdError::HwTimeOut;
        }


        self.usbfCtxt.UsbControllerEnabled = true;
        self.w32(USB2D_PERIODICLISTBASE, 0);
        self.echo32(USB2D_ENDPTSETUPSTAT);
        self.echo32(USB2D_ENDPTCOMPLETE);
        self.and32(USB2D_USBCMD, !USBCMD_RUNSTOP);
        self.and32(USB2D_USBMODE, USBMODE_CM_MASK);
        self.or32(USB2D_USBCMD, USBCMD_RST);
        for i in 0..100000
        {
            timerWait(1);

            timed_out = (self.r32(USB2D_USBCMD) & USBCMD_RST) != 0;
            if (!timed_out) { break; }
        }

        if (timed_out)
        {
            //printf("usbcmd rst failed\n\r");
            return UsbdError::HwTimeOut;
        }

        for i in 0..100000
        {
            timerWait(1);

            timed_out = (self.r32(USB_SUSP_CTRL) & SUSPCTRL_USB_PHY_CLK_VALID) == 0;
            if (!timed_out) { break; }
        }
        
        if (timed_out)
        {
            //printf("clock is invalid 2\n\r");
            return UsbdError::HwTimeOut;
        }
        
        self.and32(USB2D_USBMODE, USBMODE_CM_MASK);
        self.or32(USB2D_USBMODE, USBMODE_CM_DEVICE);
        for i in 0..100000
        {
            timerWait(1);
            timed_out = ((self.r32(USB2D_USBMODE) & USBMODE_CM_MASK) != USBMODE_CM_DEVICE);
            if (!timed_out) { break; }
        }
        
        if (timed_out)
        {
            //printf("not in device mode\n\r");
            return UsbdError::HwTimeOut;
        }
        
        self.w32(USB2D_USBINTR, 0);
        self.w32(USB2D_OTGSC, OTGSC_AVV);
        self.w32(USB2D_USBSTS, 0x1FF);
        self.w32(USB2D_OTGSC, 0x7F0000);
        self.w32(USB2D_ENDPTSETUPSTAT, 7);
        self.and32(USB2D_USBCMD, 0xFF00FFFF);
        
        return UsbdError::Success;
    }
    
    pub fn get_ep_queue_head(&mut self) -> u32
    {
        return self.r32(USB2D_QH_EP_n_OUT);
    }
    
    pub fn init_controller(&mut self) -> UsbdError
    {
        self.init_context();
    
        memset_iou32(self.get_ep_queue_head() as u64, 0, mem::size_of::<UsbDevQueueHead>() * USBD_EPNUM_MAX);
        for i in 0..USBD_EPNUM_MAX
        {
            memset_iou32(self.endpoints[i].pDataTransDesc, 0, mem::size_of::<UsbTransDesc>());
        }

        let cur_head: u32 = self.get_ep_queue_head();
        self.w32(USB2D_ASYNCLISTADDR, cur_head);

        // Control endpoints are initialized by default for enumeration
        let usbd_epCtrlOut = &mut self.endpoints[UsbEpNum::CTRL_OUT as usize];
        usbd_epCtrlOut.ep_init();
        
        let usbd_epCtrlIn = &mut self.endpoints[UsbEpNum::CTRL_IN as usize];
        usbd_epCtrlIn.ep_init();

        // Disable automatic low-power state
        self.and32(USB2D_HOSTPC1_DEVLC, !DEVLC_AUTOLP);

        // Send attach event
        self.or32(USB2D_USBCMD, USBCMD_RUNSTOP);
        for i in 0..100000
        {
            timerWait(1);
            
            // Check whether host has acknowledged our attachment
            if ((self.r32(USB2D_USBCMD) & USBCMD_RUNSTOP) != 0)
            {
                return UsbdError::Success;
            }
        }
        
        return UsbdError::HwTimeOut;
    }
}

pub fn usbd_recover() -> UsbdError
{
    let mut ret: UsbdError = UsbdError::Success;
    let mut usbd: UsbDevice = UsbDevice::new();
    
    usbd.init();
    //cdc_init();
    
    usbd.init_context();
    usbd.enable_clocks();
    usbd.init_controller();
    
    /*debug_interface = usbd_interface_alloc(2);
    debug_interface->class = 0xFF;
    debug_interface->subclass = 0xFF;
    debug_interface->protocol = 0xFF;
    usbd_ep_construct(&debug_interface->endpoints[0], 512, USB_EPATTR_TTYPE_BULK, 0);
    usbd_ep_construct(&debug_interface->endpoints[1], 512, USB_EPATTR_TTYPE_BULK, 0);

    usbd_ep_idle(usbd_get_endpoint_from_epnum(USB_EP_BULK_OUT));
    
    //irq_bind(IRQ_USB, irq_usb);
    //irq_bind(IRQ_USB2, irq_usb);
    
    GetUsbBaseAddress()[USB2D_USBINTR] |= (USBSTS_USBINT | USBSTS_USBPORT); // usb interrupt enable
    */
    
    /*while (!GetUsbCtx()->EnumerationDone)
    {
        irq_usb();
    }*/

    return ret;
}

/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::util::*;
use crate::io::car::*;
use crate::io::timer::*;
use crate::io::pmc::*;
use crate::logger::*;
use core::mem;
use alloc::vec::Vec;
use wchar::{wch, wch_c};
use crate::usbd::debug::*;

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

pub const USBD_EPNUM_MAX:      usize = (8);
pub const USBD_EPIDX_MAX:        u32 = (16);

pub const USBD_CTRL_PKT_MAX:     u16 = (64);

pub const USB_EPATTR_TTYPE_BULK: u8 = (2);
pub const USB_EPATTR_TTYPE_INTR: u8 = (3);

pub const CS_INTERFACE:  u8 = 0x24;
pub const USB_ST_HEADER: u8 = 0x00;
pub const USB_ST_CMF:    u8 = 0x01;
pub const USB_ST_ACMF:   u8 = 0x02;
pub const USB_ST_UF:     u8 = 0x06;

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
            ($attr & 0x3)
        }
    }
    
    macro_rules! USB2D_IS_EP_TX {
        ($ep:expr) => {
            ((($ep & bit!(0)) != 0))
        }
    }
    
    macro_rules! USB2D_EPIDX {
        ($ep:expr) => {
            ($ep >> 1)
        }
    }
    
    macro_rules! USB2D_EPBIT {
        ($ep:expr) => {
            if ($ep == (UsbEpNum::EP_ALL as u8)) { 0xFFFFFFFF } else { if USB2D_IS_EP_TX!($ep) { bit!(16) << USB2D_EPIDX!($ep) } else { bit!(0) << USB2D_EPIDX!($ep) } }
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
            (($n as u32) << 16)
        }
    }
    
    // DtdInfo
    macro_rules! DTD_GETINFO_BYTES {
        ($val:expr) => {
            (($val as u32) >> 16)
        }
    }
    
    macro_rules! INFO_BYTES {
        ($n:expr) => {
            ((($n as u32) & 0xFFFF) << 16)
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
pub enum UsbEpStatus
{
    // Transfer on the endpoint is completed
    TxfrComplete = 0,
    // Transfer on the endpoint is still active
    TxfrActive = 1,
    // Transfer on the endpoint failed
    TxfrFail = 2,
    // Endpoint is idle, ready for new data transfers
    TxfrIdle = 3,
    // Endpoint stalled
    Stalled = 4,
    // Endpoint is not configured yet
    NotConfigured = 5,
}

#[repr(i32)]
#[derive(Copy, Clone, PartialEq)]
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

#[repr(u8)]
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

const USB_DT_DEVICE:u8             = 1;
const USB_DT_CONFIG:u8             = 2;
const USB_DT_STRING:u8             = 3;
const USB_DT_INTERFACE:u8          = 4;
const USB_DT_ENDPOINT:u8           = 5;
const USB_DT_DEVICE_QUALIFIER:u8   = 6;

const USB_DT_OTHER_SPEED_CONFIG:u8 = 7;
const USB_DT_INTERFACE_ASSOCIATION:u8 = 11;

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
#[derive(Copy, Clone)]
pub struct UsbSetupPacket
{
    pub bmRequestType: u8,
    pub bRequest: u8,
    pub wValue: u16,
    pub wIndex: u16,
    pub wLength: u16
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
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bcdUsb: u16,
    pub bDeviceClass: u8,
    pub bDeviceSubclass: u8,
    pub bDeviceProtocol: u8,
    pub bMaxPacketSize: u8,
    pub idVendor: u16,
    pub idProduct: u16,
    pub bcdDevice: u16,
    pub iManufacturer: u8,
    pub iProduct: u8,
    pub iSerialNumber: u8,
    pub bNumConfigurations: u8,
}

#[repr(C, packed(1))]
pub struct UsbDtString
{
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub data: [u16; 0x1F] // packets are max 0x40 here so just alloc up to that
}

#[repr(C, packed(1))]
pub struct UsbDtConfig
{
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub wTotalLength: u16,
    pub bNumInterfaces: u8,
    pub bConfigurationValue: u8,
    pub iConfiguration: u8,
    pub bmAttributes: u8,
    pub bMaxPower: u8,
}

#[repr(C, packed(1))]
pub struct UsbDtInterface
{
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bInterfaceNumber: u8,
    pub bAlternateSetting: u8,
    pub bNumEndpoints: u8,
    pub bInterfaceClass: u8,
    pub bInterfaceSubclass: u8,
    pub bInterfaceProtocol: u8,
    pub iInterface: u8,
}

#[repr(C, packed(1))]
pub struct UsbDtEndpoint
{
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bEndpointAddress: u8,
    pub bmAttributes: u8,
    pub wMaxPacketSize: u16,
    pub bInterval: u8,
}

#[repr(C, packed(1))]
pub struct UsbDtClassHeaderFunc
{
    pub bFunctionLength: u8,
    pub bDescriptorType: u8,
    pub bDescriptorSubtype: u8,  /* 0x00 */
    pub bcdCDC: u16,
}

#[repr(C, packed(1))]
pub struct UsbDtClassCallMgmt
{
    pub bFunctionLength: u8,
    pub bDescriptorType: u8,
    pub bDescriptorSubtype: u8,	/* 0x01 */
    pub bmCapabilities: u8,
    pub bDataInterface: u8,
}

#[repr(C, packed(1))]
pub struct UsbDtClassAbstractControl
{
    pub bFunctionLength: u8,
    pub bDescriptorType: u8,
    pub bDescriptorSubtype: u8,	/* 0x02 */
    pub bmCapabilities: u8,
}

#[repr(C, packed(1))]
pub struct UsbDtClassUnionFunction
{
    pub bFunctionLength: u8,
    pub bDescriptorType: u8,
    pub bDescriptorSubtype: u8,	/* 0x06 */
    pub bMasterInterface: u8,
    pub bSlaveInterface0: u8,
}

#[repr(C, packed(1))]
pub struct UsbDtDeviceQualifier
{
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bcdUsb: u16,
    pub bDeviceClass: u8,
    pub bDeviceSubclass: u8,
    pub bDeviceProtocol: u8,
    pub bMaxPacketSize0: u8,
    pub bNumConfigurations: u8,
    pub bReserved: u8,
}

#[repr(C, packed(1))]
pub struct UsbDtInterfaceAssociation
{
    pub bLength: u8,
    pub bDescriptorType: u8,
    pub bFirstInterface: u8,
    pub bInterfaceCount: u8,
    pub bFunctionClass: u8,
    pub bFunctionSubclass: u8,
    pub bFunctionProtocol: u8,
    pub iFunction: u8,
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

/*#define DEF_USB_DT_STR(name, str) \
const UsbDtString name = \
{ \
    .bLength = 2+sizeof((str)), \
    .bDescriptorType = USB_DT_STRING, \
    .data = (str), \
};*/

const deviceDesc: UsbDtDevice = UsbDtDevice
{
    bLength: 18, //mem::size_of::<UsbDtDevice>(),
    bDescriptorType: 1, //USB_DT_DEVICE,
    bcdUsb: 0x0200,
    bDeviceClass: 0,
    bDeviceSubclass: 0,
    bDeviceProtocol: 0,
    bMaxPacketSize: 64,
    idVendor: 0x057e, // Nintendo Co. Ltd
    idProduct: 0x2000, // Nintendo Switch
    bcdDevice: 0x0101,
    iManufacturer: 1,
    iProduct: 2,
    iSerialNumber: 0,
    bNumConfigurations: 1,
};

const strDescManufacturer: UsbDtString = UsbDtString {
    bLength: 16+2,
    bDescriptorType: 3, //USB_DT_STRING as u8,
    data: *wch!("Nintendo\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0")
};

const strDescProduct: UsbDtString = UsbDtString {
    bLength: 12+2,
    bDescriptorType: 7, //USB_DT_STRING as u8,
    data: *wch!("NX-HTB\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0")
};

const strDescSerial: UsbDtString = UsbDtString {
    bLength: 14+2,
    bDescriptorType: 3, //USB_DT_STRING as u8,
    data: *wch!("HAC-001\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0")
};

const strDescLang: UsbDtString = UsbDtString
{
    bLength: 4,
    bDescriptorType: 3, //USB_DT_STRING,
    data: [0x0409, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
};

const configDesc: UsbDtConfig = UsbDtConfig
{
    bLength: 9,
    bDescriptorType: 2, //USB_DT_CONFIG,
    wTotalLength: 32,
    bNumInterfaces: 0,
    bConfigurationValue: 1,
    iConfiguration: 0,
    bmAttributes: 0xC0, // usb1.1+, self-powered, no remote-wakeup
    bMaxPower: 16, //32mA
};

const deviceQualifierDesc: UsbDtDeviceQualifier = UsbDtDeviceQualifier
{
    bLength: 10,
    bDescriptorType: 6,
    bcdUsb: 0x0200,
    bDeviceClass: 8,
    bDeviceSubclass: 6,
    bDeviceProtocol: 80,
    bMaxPacketSize0: 64,
    bNumConfigurations: 1,
    bReserved: 0
};

// Endpoint info struct
pub struct UsbdEndpoint
{
    pub isAssigned: bool,
    pub isEnabled: bool,
    pub complete_handler: Option<fn(&mut UsbDevice, u8)>,
    pub idle_handler: Option<fn(&mut UsbDevice, u8)>,
    pub fail_handler: Option<fn(&mut UsbDevice, u8)>,
    pub pDataTransDesc: u64,
    //UsbTransDesc* pDataTransDesc;
    pub epConfigured: bool,
    pub bytesRequested: u32,
    
    pub epNum: u8,
    pub epAddress: u8,
    pub attributes: u8,
    pub maxPacketSize: u16,
    pub interval: u8,
}

// Interface info struct
pub struct UsbdInterface
{
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
    pub interfaceNumber: u8,
    
    pub extra_desc_data: u64,
    pub extra_desc_size: usize,
    
    pub associatedNum: u8,
    
    pub numEndpoints: u8,
    pub endpointStart: u8,
    //UsbdEndpoint* endpoints;
    
    //UsbdInterface* next;
}

#[repr(i32)]
#[derive(Copy, Clone, PartialEq)]
pub enum UsbdError
{
    Success = 0,
    HwTimeOut = 3,
    TxferFailed = 26,
    EpNotConfigured = 28
}

//
// END TYPES/STRUCTS
//

const usbdEndpoint_init: UsbdEndpoint = UsbdEndpoint {
    isAssigned: false,
    isEnabled: false,
    complete_handler: None,
    idle_handler: None,
    fail_handler: None,
    pDataTransDesc: 0,
    
    epConfigured: false,
    bytesRequested: 0,
    epNum: 0,
    epAddress: 0,
    attributes: 0,
    maxPacketSize: 0,
    interval: 0
};

const usbdCtxt_init: UsbControllerContext = UsbControllerContext {
    UsbPortSpeed: 0,
    UsbBaseAddr: 0, // vu32*
    setupPkt: UsbSetupPacket {
        bmRequestType: 0,
        bRequest: 0,
        wValue: 0,
        wIndex: 0,
        wLength: 0
    },
    EnumerationDone: false,
    UsbControllerEnabled: false,
    UsbConfigurationNo: 0,
    UsbInterfaceNo: 0,
    InitializationDone: false
};

pub struct UsbDevice
{
    endpoints: [UsbdEndpoint; USBD_EPNUM_MAX],
    initialized: bool,
    usbfCtxt: UsbControllerContext,
    interfaceCount: u8,
    interfaces: Vec<UsbdInterface>,
    usbDeviceStatus: u16,
    setup_handlers: Vec<fn(&mut UsbDevice, UsbSetupPacket)->bool>,
    reset_handlers: Vec<fn(&mut UsbDevice)>
}

impl UsbdInterface
{
    pub fn new(num_eps: u8, iface_num: u8) -> Self
    {
        let mut retval: UsbdInterface = UsbdInterface {
            class: 0,
            subclass: 0,
            protocol: 0,
            interfaceNumber: iface_num,
            extra_desc_data: 0,
            extra_desc_size: 0,
            
            associatedNum: 0,
            
            numEndpoints: num_eps,
            endpointStart: 0xFF
        };
        
        return retval;
    }
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
}

impl UsbDevice
{
    pub const fn empty() -> Self
    {
        UsbDevice {
            endpoints: [usbdEndpoint_init; USBD_EPNUM_MAX],
            initialized: false,
            usbfCtxt: usbdCtxt_init,
            interfaceCount: 0,
            interfaces: Vec::new(),
            usbDeviceStatus: 0,
            setup_handlers: Vec::new(),
            reset_handlers: Vec::new(),
        }
    }

    pub fn new() -> Self {
        let mut retval: UsbDevice = UsbDevice {
            endpoints: [usbdEndpoint_init; USBD_EPNUM_MAX],
            initialized: false,
            usbfCtxt: usbdCtxt_init,
            interfaceCount: 0,
            interfaces: Vec::new(),
            usbDeviceStatus: 0,
            setup_handlers: Vec::new(),
            reset_handlers: Vec::new(),
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
            self.endpoints[i].complete_handler = None;
            self.endpoints[i].idle_handler = None;
            self.endpoints[i].fail_handler = None;
            
            self.endpoints[i].epNum = i_u8;
            self.endpoints[i].epAddress = USB2D_EPNUM_TO_EPADDR!(i_u8);
            self.endpoints[i].pDataTransDesc = 0x7d001200 + ((i as u64) * mem::size_of::<UsbTransDesc>() as u64)
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
            self.endpoints[i].epConfigured = false;
            self.endpoints[i].bytesRequested = 0;
            ////memset(usbd_endpoints[i].pDataTransDesc, 0, sizeof(UsbTransDesc));
        }  
    }
    
    pub fn ep_alloc(&mut self) -> u8
    {
        // Iterate by Tx/Rx pairs and allocate a pair
        let mut i: usize = USB_EP_CONFIGURABLE_BEGIN as usize;
        loop
        {
            if (i >= USBD_EPNUM_MAX as usize)
            {
                break;
            }
            
            if (!self.endpoints[i].isAssigned)
            {
                self.endpoints[i].isAssigned = true;
                self.endpoints[i+1].isAssigned = true;
                return i as u8;
            }

            i += 2;
        }
        return 0xFF;
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
        
        self.endpoints_resetall();
    }
    
    pub fn w32(&mut self, offs: u32, val: u32)
    {
        pokeio32(self.usbfCtxt.UsbBaseAddr + (offs*4), val);
    }
    
    pub fn r32(&mut self, offs: u32) -> u32
    {
        return peekio32(self.usbfCtxt.UsbBaseAddr + (offs*4));
    }
    
    pub fn echo32(&mut self, offs: u32)
    {
        let old: u32 = peekio32(self.usbfCtxt.UsbBaseAddr + (offs*4));
        pokeio32(self.usbfCtxt.UsbBaseAddr + (offs*4), old);
    }
    
    pub fn or32(&mut self, offs: u32, val: u32)
    {
        let old: u32 = peekio32(self.usbfCtxt.UsbBaseAddr + (offs*4));
        pokeio32(self.usbfCtxt.UsbBaseAddr + (offs*4), old | val);
    }
    
    pub fn and32(&mut self, offs: u32, val: u32)
    {
        let old: u32 = peekio32(self.usbfCtxt.UsbBaseAddr + (offs*4));
        pokeio32(self.usbfCtxt.UsbBaseAddr + (offs*4), old & val);
    }
    
    pub fn enable_clocks(&mut self)
    {
        let OSC_FREQ: usize = (peekio32(CLK_RST_CONTROLLER_OSC_CTRL) >> 28) as usize;
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
        timer_wait(2);
        rst_dev_l_set |= CLK_ENB_USBD;
        timer_wait(2);
        rst_dev_l_clr |= CLK_ENB_USBD;
        timer_wait(2);
        rst_dev_w_clr |= XUSB_PADCTL_RST;
        timer_wait(2);

        self.or32(USB_SUSP_CTRL, SUSPCTRL_UTMIP_RESET);
        self.or32(USB_SUSP_CTRL, SUSPCTRL_UTMIP_PHY_ENB);
        utmipll_hw_pwrdn_cfg0 &= 0xFFFFFFFD;

        timer_wait(10);

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
            timer_wait(10);
            if (utmipll_hw_pwrdn_cfg0 & bit!(31) != 0)
            {
                break;
            }
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
        timer_wait(10);
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
        timer_wait(1);
        self.and32(USB1_UTMIP_BIAS_CFG1, !bit!(0));
        self.or32(USB1_UTMIP_BIAS_CFG1, bit!(1));
        timer_wait(100);
        self.or32(USB1_UTMIP_BIAS_CFG1, bit!(0));
        self.and32(USB1_UTMIP_BIAS_CFG1, !bit!(23));
        timer_wait(3);
        self.and32(USB1_UTMIP_BIAS_CFG1, !bit!(0));
        timer_wait(100);
        self.or32(USB1_UTMIP_BIAS_CFG1, bit!(0));
        self.and32(USB1_UTMIP_BIAS_CFG1, !bit!(23));
        clk_out_enb_y &= !(bit!(18));
        utmip_pll_cfg2 &= !(bit!(0) | bit!(4) | bit!(2) | bit!(24));
        utmip_pll_cfg2 |= (bit!(1) | 0x28 | 0x2000000);
        timer_wait(10);
        self.and32(USB1_UTMIP_BIAS_CFG0, 0xFF3FF7FF);
        timer_wait(10);
        apbdev_pmc_usb_ao &= 0xFFFFFFF3;
        timer_wait(10);
        self.and32(USB1_UTMIP_XCVR_CFG0, 0xFFFFBFFF);
        timer_wait(10);
        self.and32(USB1_UTMIP_XCVR_CFG0, 0xFFFEFFFF);
        timer_wait(10);
        self.and32(USB1_UTMIP_XCVR_CFG0, 0xFFFBFFFF);
        timer_wait(10);
        self.and32(USB1_UTMIP_XCVR_CFG1, 0xFFFFFFFB);
        timer_wait(10);
        self.and32(USB1_UTMIP_XCVR_CFG1, 0xFFFFFFEF);
        timer_wait(10);
        
        if(self.enable_devicemode() != UsbdError::Success)
        {
            println!("usbd: timed out enabling device mode");
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
            timer_wait(80000);

            timed_out = (self.r32(USB_SUSP_CTRL) & SUSPCTRL_USB_PHY_CLK_VALID) == 0;
            if (!timed_out) { break; }
        }
        
        if (timed_out)
        {
            println!("usbd: clock is invalid");
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
            timer_wait(80000);

            timed_out = (self.r32(USB2D_USBCMD) & USBCMD_RST) != 0;
            if (!timed_out) { break; }
        }

        if (timed_out)
        {
            //printf("usbcmd rst failed");
            return UsbdError::HwTimeOut;
        }

        for i in 0..1000
        {
            timer_wait(100);

            timed_out = (self.r32(USB_SUSP_CTRL) & SUSPCTRL_USB_PHY_CLK_VALID) == 0;
            if (!timed_out) { break; }
        }
        
        if (timed_out)
        {
            println!("usbd: clock is invalid 2");
            return UsbdError::HwTimeOut;
        }
        
        self.and32(USB2D_USBMODE, USBMODE_CM_MASK);
        self.or32(USB2D_USBMODE, USBMODE_CM_DEVICE);
        for i in 0..1000
        {
            timer_wait(100);
            timed_out = ((self.r32(USB2D_USBMODE) & USBMODE_CM_MASK) != USBMODE_CM_DEVICE);
            if (!timed_out) { break; }
        }
        
        if (timed_out)
        {
            println!("usbd: not in device mode");
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
    
    pub fn get_ep_queue_head_ptr(&mut self, ep_num: u8) -> u32
    {
        return self.usbfCtxt.UsbBaseAddr + (USB2D_QH_EP_n_OUT*4) + (mem::size_of::<UsbDevQueueHead>() * ep_num as usize) as u32;
    }
    
    pub fn get_ep_queue_head(&mut self, ep_num: u8) -> &mut UsbDevQueueHead
    {
        let p_queue_head: *mut UsbDevQueueHead = self.get_ep_queue_head_ptr(ep_num) as _;
        unsafe
        {
            let queue_head = &mut *p_queue_head;
            return queue_head;
        }
    }
    
    pub fn get_ep_transdesc(&mut self, ep_num: u8) -> &mut UsbTransDesc
    {
        let p_desc: *mut UsbTransDesc = self.endpoints[ep_num as usize].pDataTransDesc as _;
        unsafe
        {
            let desc = &mut *p_desc;
            return desc;
        }
    }
    
    pub fn ep_configure(&mut self, epNum: u8)
    {
        let p_epQueue = self.get_ep_queue_head_ptr(epNum);
        let maxPacketSize = self.endpoints[epNum as usize].maxPacketSize;
        let epQueue = self.get_ep_queue_head(epNum);

        memset_iou32(p_epQueue as u64, 0, mem::size_of::<UsbDevQueueHead>());

        if (epNum == UsbEpNum::CTRL_OUT as u8) {
            epQueue.EpCapabilities = EPCAP_IOS; // IRQ on setup
        }
        else {
            epQueue.EpCapabilities = 0;
        }

        epQueue.NextDTDPtr = 1;
        epQueue.EpCapabilities |= EPCAP_MAX_PKT!(maxPacketSize & 0x7FF) as u32 | EPCAP_ZLT | EPCAP_HISPEED;
    }
    
    pub fn ep_init(&mut self, epNum: u8)
    {
        self.ep_configure(epNum);
        
        // Clear type bits and stall bits, enable the endpoint
        self.and32(USB2D_ENDPTCTRL!(USB2D_EPIDX!(epNum) as u32), !CTRL_EPT_MASK!(epNum));
        self.and32(USB2D_ENDPTCTRL!(USB2D_EPIDX!(epNum) as u32), !CTRL_EPSTALL!(epNum));
        self.or32(USB2D_ENDPTCTRL!(USB2D_EPIDX!(epNum) as u32), CTRL_EPENABLE!(epNum));
        
        // Configure the endpoing type, hold bulk endpoints in reset
        if (USB_EPATTR_TTYPE!(self.endpoints[epNum as usize].attributes) == USB_EPATTR_TTYPE_BULK)
        {
            self.or32(USB2D_ENDPTCTRL!(USB2D_EPIDX!(epNum) as u32), CTRL_EPT_BULK!(epNum));
            self.or32(USB2D_ENDPTCTRL!(USB2D_EPIDX!(epNum) as u32), CTRL_EPRESET!(epNum));
        }
        else if (USB_EPATTR_TTYPE!(self.endpoints[epNum as usize].attributes) == USB_EPATTR_TTYPE_INTR)
        {
            self.or32(USB2D_ENDPTCTRL!(USB2D_EPIDX!(epNum) as u32), CTRL_EPT_INTR!(epNum));
            self.or32(USB2D_ENDPTCTRL!(USB2D_EPIDX!(epNum) as u32), CTRL_EPRESET!(epNum));
        }
        else if (epNum == UsbEpNum::CTRL_IN as u8 || epNum == UsbEpNum::CTRL_OUT as u8)
        {
            self.or32(USB2D_ENDPTCTRL!(USB2D_EPIDX!(epNum) as u32), CTRL_EPT_CTRL!(epNum));
        }
    }
    
    pub fn init_controller(&mut self) -> UsbdError
    {
        self.init_context();
    
        memset_iou32(self.get_ep_queue_head_ptr(0) as u64, 0, mem::size_of::<UsbDevQueueHead>() * USBD_EPNUM_MAX);
        for i in 0..USBD_EPNUM_MAX
        {
            memset_iou32(self.endpoints[i].pDataTransDesc, 0, mem::size_of::<UsbTransDesc>());
        }

        let cur_head: u32 = self.get_ep_queue_head_ptr(0);
        self.w32(USB2D_ASYNCLISTADDR, cur_head);

        // Control endpoints are initialized by default for enumeration
        self.ep_init(UsbEpNum::CTRL_OUT as u8);
        self.ep_init(UsbEpNum::CTRL_IN as u8);

        // Disable automatic low-power state
        self.and32(USB2D_HOSTPC1_DEVLC, !DEVLC_AUTOLP);

        // Send attach event
        self.or32(USB2D_USBCMD, USBCMD_RUNSTOP);
        for i in 0..1000
        {
            timer_wait(100);
            
            // Check whether host has acknowledged our attachment
            if ((self.r32(USB2D_USBCMD) & USBCMD_RUNSTOP) != 0)
            {
                return UsbdError::Success;
            }
        }
        
        println!("usbd: Controller init timed out!!");
        return UsbdError::HwTimeOut;
    }
    
    pub fn create_interface(&mut self, num_eps: u8) -> u8
    {
        let ep_num = self.ep_alloc();
        let mut interface: UsbdInterface = UsbdInterface::new(num_eps, self.interfaceCount);
        
        interface.endpointStart = ep_num;
        self.interfaceCount += 1;
        
        let retval: u8 = interface.interfaceNumber;
        self.interfaces.push(interface);
        
        return retval;
    }
    
    pub fn get_interface(&mut self, iface_num: u8) -> &mut UsbdInterface
    {
        return &mut self.interfaces[iface_num as usize];
    }
    
    pub fn get_ep(&mut self, ep_num: u8) -> &mut UsbdEndpoint
    {
        return &mut self.endpoints[ep_num as usize];
    }
    
    pub fn ep_idle(&mut self, ep_num: u8)
    {
        //println!("usbd: ep idle...");
        self.ep_flush(ep_num);
        //println!("usbd: ep idle flushed...");
        timer_wait(100);
        
        let queue_head = self.get_ep_queue_head_ptr(ep_num);
        let ep = &mut self.endpoints[ep_num as usize];
        
        memset_iou32(ep.pDataTransDesc, 0, mem::size_of::<UsbTransDesc>());
        memset_iou32(queue_head as u64, 0, mem::size_of::<UsbDevQueueHead>());

        ep.epConfigured = false;
        ep.bytesRequested = 0;

        // set endpoint transmit complete event
        self.or32(USB2D_ENDPTCOMPLETE, USB2D_EPBIT!(ep_num));
        //println!("usbd: ep idle complete");
    }
    
    pub fn ep_flush(&mut self, epNum: u8) -> UsbdError
    {
        //println!("ep flush...");
        self.w32(USB2D_ENDPTFLUSH, USB2D_EPBIT!(epNum)); // epflush
        for i in 0..1000
        {
            timer_wait(100);
            if ( (self.r32(USB2D_ENDPTFLUSH) & USB2D_EPBIT!(epNum)) == 0 )
            {
                //println!("a");
                for j in 0..1000
                {
                    timer_wait(100);
                    if ( (self.r32(USB2D_ENDPTSTAT) & USB2D_EPBIT!(epNum)) == 0 )
                    {
                        //println!("b");
                        for k in 0..1000
                        {
                            timer_wait(100);
                            if ( (self.r32(USB2D_ENDPTPRIME) & USB2D_EPBIT!(epNum)) == 0 )
                            {
                                //println!("c");
                                return UsbdError::Success;
                            }
                        }
                        //println!("usbd: ep flush timeout 3...");
                        return UsbdError::HwTimeOut;
                    }
                }
                //println!("usbd: ep flush timeout 2...");
                return UsbdError::HwTimeOut;
            }
        }
        //println!("usbd: ep flush timeout 1...");
        return UsbdError::HwTimeOut;
    }
    
    pub fn ep_status(&mut self, epNum: u8) -> UsbEpStatus
    {
        let epctrl_val = self.r32(USB2D_ENDPTCTRL!(USB2D_EPIDX!(epNum) as u32));

        if ((epctrl_val & CTRL_EPSTALL!(epNum)) != 0)
        {
            return UsbEpStatus::NotConfigured;
        }

        if ((epctrl_val & CTRL_EPENABLE!(epNum)) == 0)
        {
            return UsbEpStatus::NotConfigured;
        }
        
        let queue_head = self.get_ep_queue_head(epNum);

        if ((queue_head.DtdInfo & (INFO_HALTED | INFO_BUFFER_ERROR | INFO_TX_ERROR)) == 0)
        {
            if ((self.r32(USB2D_ENDPTPRIME)    & USB2D_EPBIT!(epNum)) != 0
                || (self.r32(USB2D_ENDPTSTAT) & USB2D_EPBIT!(epNum)) != 0) {
                return UsbEpStatus::TxfrActive;
            }
            else if (self.endpoints[epNum as usize].epConfigured)  {
                return UsbEpStatus::TxfrComplete;
            }
            else {
                return UsbEpStatus::TxfrIdle;
            }
        }

        return UsbEpStatus::TxfrFail;
    }
    
    pub fn ep_wait(&mut self, epNum: u8) -> UsbdError
    {
        let mut retval = self.ep_status(epNum);
        if (retval == UsbEpStatus::TxfrActive)
        {
            for i in 0..10000
            {
                timer_wait(100);
                if (self.ep_status(epNum) != UsbEpStatus::TxfrActive 
                    || self.ep_status(epNum) == UsbEpStatus::TxfrIdle) {
                    return UsbdError::Success;
                }
            }
            
            //log_uarta("usbd: Transfer active, but timed out!! ");
            //logu32(self.ep_status(epNum) as u32);
            
            if (self.ep_status(epNum) != UsbEpStatus::TxfrIdle) {
                self.ep_idle(epNum);
            }

            return UsbdError::HwTimeOut;
        }
        else if (retval == UsbEpStatus::NotConfigured)
        {
            self.ep_idle(epNum);
            return UsbdError::EpNotConfigured;
        }
        else 
        {
            retval = self.ep_status(epNum);
            if (retval != UsbEpStatus::TxfrComplete && retval != UsbEpStatus::TxfrIdle) {
                return UsbdError::TxferFailed;
            }
        }
        return UsbdError::Success;
    }
    
    pub fn ep_set_stall(&mut self, epNum: u8, stall: bool)
    {
        let old = self.r32(USB2D_ENDPTCTRL!(USB2D_EPIDX!(epNum) as u32));
        self.w32(USB2D_ENDPTCTRL!(USB2D_EPIDX!(epNum) as u32), (old & !CTRL_EPSTALL!(epNum)) | (if stall { CTRL_EPSTALL!(epNum) } else { 0 }));
        if (stall) {
            return;
        }
        self.or32(USB2D_ENDPTCTRL!(USB2D_EPIDX!(epNum) as u32), CTRL_EPRESET!(epNum));
    }
    
    pub fn update_port_speed(&mut self)
    {
        let devlc = self.r32(USB2D_HOSTPC1_DEVLC);
        self.usbfCtxt.UsbPortSpeed = (devlc & USB2D_PSPD_MASK) >> USB2D_PSPD_SHIFT;
    }
    
    pub fn interrupt_en(&mut self)
    {
        self.or32(USB2D_USBINTR, (USBSTS_USBINT | USBSTS_USBPORT)); // usb interrupt enable
    }
    
    pub fn is_enumeration_done(&mut self) -> bool
    {
        return self.usbfCtxt.EnumerationDone;
    }
    
    pub fn halt_activity(&mut self)
    {
        self.usbfCtxt.EnumerationDone = false;
    
        // Clear periodic list
        self.w32(USB2D_PERIODICLISTBASE, 0);
        
        // Clear all pending transactions
        self.echo32(USB2D_ENDPTSETUPSTAT);
        self.echo32(USB2D_ENDPTCOMPLETE);
        
        // Flush all endpoints
        self.ep_flush(UsbEpNum::EP_ALL as u8);
        self.endpoints_resetall();
    }
    
    pub fn disconnect(&mut self)
    {
        // Kill all transactions
        self.halt_activity();
        
        // Disable pullup on D+ to signal a disconnect to the host
        self.w32(USB2D_USBCMD, USBCMD_FS2);
        timer_wait(800);
    }
    
    pub fn epidx_exists(&mut self, epNum: u8) -> bool
    {
        if (epNum > USBD_EPNUM_MAX as u8) {
            return false;
        }
        
        return self.endpoints[epNum as usize].isEnabled;
    }
    
    pub fn get_xferbuf(&mut self, epNum: u8) -> u64
    {
        if (epNum == 4) {
            return 0x7d001400;
        }
        else if (epNum == 5) {
            return 0x7d001600;
        }

        if (epNum < USBD_EPNUM_MAX as u8)
        {
            //return (void*)(USBD_XFERBUF_START + (epNum * USBD_XFERBUF_SIZE));
            return 0x7d001300 + (epNum as u64 * 0x40);
        }
        
        return 0;
    }

    pub fn xferbuf_flush(&mut self, epNum: u8, len: usize)
    {
        let xferbuf = self.get_xferbuf(epNum);
        //if (xferbuf)
        //    dcache_flush(xferbuf, len);
    }

    pub fn xferbuf_invalidate(&mut self, epNum: u8, len: usize)
    {
        let xferbuf = self.get_xferbuf(epNum);
        //if (xferbuf)
        //    dcache_invalidate(xferbuf, len);
    }
    
    pub fn get_bytes_transmitted(&mut self, epNum: u8) -> u32
    {
        if (self.ep_status(epNum) != UsbEpStatus::TxfrComplete)
        {
            return 0;
        }
        else
        {
            let dtdInfo = self.get_ep_queue_head(epNum).DtdInfo;
            return self.endpoints[epNum as usize].bytesRequested - DTD_GETINFO_BYTES!(dtdInfo);
        }
    }
    
    pub fn get_bytes_received(&mut self, epNum: u8) -> u32
    {
        return self.get_bytes_transmitted(epNum);
    }
    
    pub fn setup_ack(&mut self) -> UsbdError
    {
        let mut result = self.ep_txfer_start(UsbEpNum::CTRL_IN as u8, 0, true);

        if (result == UsbdError::Success) {
            result = self.ep_txfer_start(UsbEpNum::CTRL_OUT as u8, 0, true);
        }

        return result;
    }
    
    pub fn ep_txfer_start(&mut self, epNum: u8, size: usize, sync: bool) -> UsbdError
    {
        let xfer_buf = self.get_xferbuf(epNum) as u32;

        self.ep_idle(epNum);
        self.ep_configure(epNum);
        
        self.get_ep_queue_head(epNum).NextDTDPtr = 0;
        self.endpoints[epNum as usize].epConfigured = true;
        self.endpoints[epNum as usize].bytesRequested = size as u32;

        let transdesc_ptr: u64 = 0x7d001200 + ((epNum as u64) * mem::size_of::<UsbTransDesc>() as u64);
        self.endpoints[epNum as usize].pDataTransDesc = transdesc_ptr;

        let pTransDesc = self.get_ep_transdesc(epNum);
        pTransDesc.NextDtd = 1;
        pTransDesc.DtdInfo = INFO_BYTES!(size) | INFO_ACTIVE | INFO_IOC;
        pTransDesc.BufPtrs[0] = xfer_buf;
        pTransDesc.BufPtrs[1] = 0;
        pTransDesc.BufPtrs[2] = 0;
        pTransDesc.BufPtrs[3] = 0;
        pTransDesc.BufPtrs[4] = 0;
        pTransDesc.Reserved = 0;

        self.get_ep_queue_head(epNum).NextDTDPtr = ((transdesc_ptr & !0x1F) & 0xFFFFFFFF) as u32;

        //dcache_flush(pTransDesc, sizeof(*pTransDesc));

        //println!("prime...");
        self.or32(USB2D_ENDPTPRIME, USB2D_EPBIT!(epNum));

        if (sync) {
            //println!("wait...");
            return self.ep_wait(epNum);
        }

        return UsbdError::Success;
    }
    
    pub fn setup_transact(&mut self, pkt: UsbSetupPacket, pDataBuffer: u64, len: usize) -> UsbdError
    {
        // Nothing to send (use usbd_setup_ack instead!)
        let mut dataLen = len;
        if (pDataBuffer == 0 || dataLen == 0) {
            return UsbdError::Success;
        }

        if (dataLen >= USBD_XFERBUF_SIZE as usize) {
            dataLen = USBD_XFERBUF_SIZE as usize;
        }

        memcpy_iou32(self.get_xferbuf(UsbEpNum::CTRL_IN as u8), pDataBuffer, dataLen);
        //println!("transact...");

        // Host requested less than we have available to provide; truncate to wLength
        if (dataLen > pkt.wLength as usize) {
            dataLen = pkt.wLength as usize;
        }

        let mut result = self.ep_txfer_start(UsbEpNum::CTRL_IN as u8, dataLen, true);
        timer_wait(8000);
        if (result == UsbdError::Success) {
            //println!("transfer success...");
            result = self.ep_txfer_start(UsbEpNum::CTRL_OUT as u8, 0, true);
        }
        
        if (result == UsbdError::Success) {
            //println!("ack success...");
        } 
        else
        {
            //println!("usbd: ack fail... {:x}", result as u32);
        }

        return result;
    }
    
    pub fn endpoints_init(&mut self)
    {
        // Iterate by Tx/Rx pairs and allocate a pair
        let mut i: usize = USB_EP_CONFIGURABLE_BEGIN as usize;
        loop
        {
            if (i >= USBD_EPNUM_MAX as usize)
            {
                break;
            }
            
            if (self.endpoints[i].isAssigned && self.endpoints[i].isEnabled)
            {
                self.ep_init(i as u8);
            }

            i += 1;
        }
    }
    
    pub fn setup_craft_configdesc(&mut self, pkt: UsbSetupPacket) -> UsbdError
    {
        unsafe
        {
        let mut ret = UsbdError::Success;
        
        let mut config_tmp_vec: Vec<u8> = Vec::with_capacity(0x10000);
        let config_tmp = to_u64ptr!(config_tmp_vec.as_mut_ptr());
        
        let mut config_cur = config_tmp;
        
        let p_config: *mut UsbDtConfig = config_cur as _;
        let config = &mut *p_config;
        
        *config = configDesc;
        
        config_cur += mem::size_of::<UsbDtConfig>() as u64;

        config.bNumInterfaces = 0;
        
        for i in 0..self.interfaces.len()
        {
            let iter = &self.interfaces[i];
            
            if (iter.associatedNum != 0)
            {
                let p_dtIterAss: *mut UsbDtInterfaceAssociation = config_cur as _;
                let dtIterAss = &mut *p_dtIterAss;
        
                dtIterAss.bLength = mem::size_of::<UsbDtInterfaceAssociation>() as u8;
                dtIterAss.bDescriptorType = USB_DT_INTERFACE_ASSOCIATION;
                dtIterAss.bFirstInterface = iter.interfaceNumber;
                dtIterAss.bInterfaceCount = iter.associatedNum;
                dtIterAss.bFunctionClass = iter.class;
                dtIterAss.bFunctionSubclass = iter.subclass;
                dtIterAss.bFunctionProtocol = iter.protocol;
                dtIterAss.iFunction = 0; // TODO strings? idk
                
                config_cur += mem::size_of::<UsbDtInterfaceAssociation>() as u64;
            }

            let p_dtInter: *mut UsbDtInterface = config_cur as _;
            let dtInter = &mut *p_dtInter;
            config_cur += mem::size_of::<UsbDtInterface>() as u64;
            
            config.bNumInterfaces += 1;
            
            dtInter.bLength = mem::size_of::<UsbDtInterface>() as u8;
            dtInter.bDescriptorType = USB_DT_INTERFACE;
            dtInter.bInterfaceNumber = iter.interfaceNumber;
            dtInter.bAlternateSetting = 0;
            dtInter.bInterfaceClass = iter.class; // vendor specific
            dtInter.bInterfaceSubclass = iter.subclass; // vendor specific
            dtInter.bInterfaceProtocol = iter.protocol; // vendor specific
            dtInter.iInterface = 0; //TODO strings? idk
            
            // TODO?
            if (iter.extra_desc_data != 0)
            {
                memcpy_iou32(config_cur, iter.extra_desc_data, iter.extra_desc_size);
                config_cur += iter.extra_desc_size as u64;
            }
            
            dtInter.bNumEndpoints = 0;
            for j in 0..iter.numEndpoints
            {
                let epNum = iter.endpointStart + j;
                if (!self.endpoints[epNum as usize].isEnabled) {
                    continue;
                }

                dtInter.bNumEndpoints += 1;

                let p_dtEp: *mut UsbDtEndpoint = config_cur as _;
                let dtEp = &mut *p_dtEp;
            
                dtEp.bLength = mem::size_of::<UsbDtEndpoint>() as u8;
                dtEp.bDescriptorType = USB_DT_ENDPOINT;
                dtEp.bEndpointAddress = self.endpoints[epNum as usize].epAddress;
                dtEp.bmAttributes = self.endpoints[epNum as usize].attributes;
                dtEp.wMaxPacketSize = self.endpoints[epNum as usize].maxPacketSize;
                dtEp.bInterval = self.endpoints[epNum as usize].interval;
                config_cur += mem::size_of::<UsbDtEndpoint>() as u64;
            }
        }
         
        config.wTotalLength = ((config_cur - config_tmp) & 0xFF) as u16;
        config.bMaxPower = (250); // 500mA*/
        ret = self.setup_transact(pkt, config_tmp, config.wTotalLength as usize);
        
        mem::drop(config_tmp_vec);
        
        return ret;
        }
    }
    
    pub fn setup_process_device_desc(&mut self, pkt: UsbSetupPacket) -> UsbdError
    {
        let descriptorIndex = (pkt.wValue & 0xFF) as u8;
        let descriptorType = ((pkt.wValue >> 8) & 0xFF) as u8;
        
        //println!("Process device desc... {}, {}", descriptorIndex, descriptorType);
        
        match descriptorType
        {
            USB_DT_DEVICE => { return self.setup_transact(pkt, to_u64ptr!(&deviceDesc), mem::size_of::<UsbDtDevice>()); }

            USB_DT_OTHER_SPEED_CONFIG 
            | USB_DT_CONFIG => {
                return self.setup_craft_configdesc(pkt);
            }

            USB_DT_STRING => {
                if (descriptorIndex == StringDescriptorIndex::USB_MANF_ID as u8) {
                    return self.setup_transact(pkt, to_u64ptr!(&strDescManufacturer), strDescManufacturer.bLength as usize);
                }
                else if (descriptorIndex == StringDescriptorIndex::USB_PROD_ID as u8) {
                    return self.setup_transact(pkt, to_u64ptr!(&strDescProduct), strDescProduct.bLength as usize);
                }
                else if (descriptorIndex == StringDescriptorIndex::USB_SERIAL_ID as u8) {
                    return self.setup_transact(pkt, to_u64ptr!(&strDescSerial), strDescSerial.bLength as usize);
                }
                else if (descriptorIndex == StringDescriptorIndex::USB_LANGUAGE_ID as u8) {
                    return self.setup_transact(pkt, to_u64ptr!(&strDescLang), strDescLang.bLength as usize);
                }
                else
                {
                    //println!("usbd: invalid string descriptor idx {}", descriptorIndex);
                    return self.setup_ack();
                }
            }

            USB_DT_DEVICE_QUALIFIER => {
                return self.setup_transact(pkt, to_u64ptr!(&deviceQualifierDesc), deviceQualifierDesc.bLength as usize);
            }

            _ => {
                //println!("usbd: invalid string descriptor type {}", descriptorType);
                return self.setup_ack();
            }
        }
    }
    
    pub fn setup_process_pkt(&mut self, pkt: UsbSetupPacket) -> UsbdError
    {
        let mut epStatus = UsbEpStatus::Stalled;

        let mut endpointStatus: [u8; 2] = [0,0];
        let mut  interfaceStatus: [u8; 2] = [0,0];

        // Run through setup handlers; If one of them has a valid response,
        // then return it, otherwise the defaults following this will run.
        for i in 0..self.setup_handlers.len()
        {
            let handlerRet = self.setup_handlers[i](self, pkt);
            if (handlerRet) {
                return UsbdError::Success;
            }
        }
        
        //println!("Process setup {:#x}, {:#x}", pkt.bmRequestType, pkt.bRequest);

        if (pkt.bmRequestType == UsbSetupRequestType::HOST2DEV_DEVICE as u8)
        {
            if (pkt.bRequest == DeviceRequestTypes::SET_ADDRESS as u8)
            {
                let result = self.ep_txfer_start(UsbEpNum::CTRL_IN as u8, 0, true);
                timer_wait(800);
                if (result == UsbdError::Success)
                {
                    let old = self.r32(USB2D_PERIODICLISTBASE);
                    self.w32(USB2D_PERIODICLISTBASE, (old & 0x1FFFFFF) | ((pkt.wValue as u32) << 25));
                }
                return result;
            }
            else if (pkt.bRequest == DeviceRequestTypes::SET_CONFIGURATION as u8)
            {
                let result = self.ep_txfer_start(UsbEpNum::CTRL_IN as u8, 0, true);
                if (result == UsbdError::Success)
                {
                    self.usbfCtxt.UsbConfigurationNo = (pkt.wValue & 0xFF) as u8;
                    self.endpoints_init();
                    self.usbfCtxt.EnumerationDone = true;
                }
                return result;
            }
            else if (pkt.bRequest == DeviceRequestTypes::GET_STATUS as u8)
            {
                return self.setup_transact(pkt, to_u64ptr!(&self.usbDeviceStatus), mem::size_of::<u16>());
            }
            else
            {
                //println!("usbd: invalid HOST2DEV_DEVICE bRequest {:x}", pkt.bRequest);
                return self.setup_ack();
            }
        }
        else if (pkt.bmRequestType == UsbSetupRequestType::DEV2HOST_DEVICE as u8)
        {
            if (pkt.bRequest == DeviceRequestTypes::GET_STATUS as u8)
            {
                return self.setup_transact(pkt, to_u64ptr!(&self.usbDeviceStatus), mem::size_of::<u16>());
            }
            else if (pkt.bRequest == DeviceRequestTypes::GET_CONFIGURATION as u8)
            {
                return self.setup_transact(pkt, to_u64ptr!(&self.usbfCtxt.UsbConfigurationNo), mem::size_of::<u8>()); 
            }
            else if (pkt.bRequest == DeviceRequestTypes::GET_DESCRIPTOR as u8)
            {
                return self.setup_process_device_desc(pkt);
            }
            else
            {
                //println!("usbd: invalid DEV2HOST_DEVICE bRequest {:x}", pkt.bRequest);
                return self.setup_ack();
            }
        }
        else if (pkt.bmRequestType == UsbSetupRequestType::HOST2DEV_INTERFACE as u8)
        {
            if (self.ep_txfer_start(UsbEpNum::CTRL_IN as u8, 0, true) == UsbdError::Success) {
                self.usbfCtxt.UsbInterfaceNo = (pkt.wValue & 0xFF) as u8;
            }
        }
        else if (pkt.bmRequestType == UsbSetupRequestType::DEV2HOST_INTERFACE as u8)
        {
            if (pkt.bRequest == DeviceRequestTypes::GET_STATUS as u8) {
                return self.setup_transact(pkt, to_u64ptr!(&interfaceStatus), mem::size_of::<u16>());
            }
            else if (pkt.bRequest == DeviceRequestTypes::GET_INTERFACE as u8) {
                return self.setup_transact(pkt, to_u64ptr!(&self.usbfCtxt.UsbInterfaceNo), mem::size_of::<u8>());
            }
            else
            {
                //println!("usbd: invalid DEV2HOST_INTERFACE bRequest {:x}", pkt.bRequest);
                return self.setup_ack();
            }
        }
        else if (pkt.bmRequestType == UsbSetupRequestType::DEV2HOST_ENDPOINT as u8)
        {
            if (pkt.bRequest == DeviceRequestTypes::GET_STATUS as u8)
            {
                let epAddr: u8 = (pkt.wIndex & 0xFF) as u8;
                if(!self.epidx_exists(epAddr))
                {
                    //printf("usbd: malformed GET_STATUS, invalid endpoint %02x\n", pkt.wIndex & 0xFF);
                    return self.setup_ack();
                }
                
                epStatus = self.ep_status(USB2D_EPADDR_TO_EPNUM!(epAddr));
                if (epStatus == UsbEpStatus::Stalled) {
                    endpointStatus[0] = 1;
                }
                else {
                    endpointStatus[0] = 0;
                }

                return self.setup_transact(pkt, to_u64ptr!(&endpointStatus[0]), mem::size_of::<u16>());
            }
            else
            {
                //println!("usbd: invalid DEV2HOST_EP bRequest {:x}", pkt.bRequest);
                return self.setup_ack();
            }
        }
        else if (pkt.bmRequestType == UsbSetupRequestType::HOST2DEV_ENDPOINT as u8)
        {
            if (pkt.bRequest == DeviceRequestTypes::CLEAR_FEATURE as u8
                || pkt.bRequest == DeviceRequestTypes::SET_FEATURE as u8)
            {
                if (pkt.wValue == 0)
                {
                    //println!("usbd: invalid [CLEAR|SET]_FEATURE wValue {:x}", pkt.wValue);
                    return self.setup_ack();
                }
                
                let epAddr: u8 = (pkt.wIndex & 0xFF) as u8;
                if(!self.epidx_exists(epAddr))
                {
                    //println!("usbd: invalid [CLEAR|SET]_FEATURE wIndex {:x}", pkt.wIndex);
                    return self.setup_ack();
                }
                
                self.ep_set_stall(UsbEpNum::CTRL_OUT as u8, false);
                return self.setup_ack();
            }
            else
            {
                //println!("usbd: invalid HOST2DEV_EP bRequest {:x}", pkt.bRequest);
                return self.setup_ack();
            }
        }
        else if (pkt.bmRequestType == UsbSetupRequestType::DEV2HOST_INTERFACE_CLASS as u8)
        {
            //println!("usbd: invalid DEV2HOST_INTERFACE_CLASS bRequest {:x}", pkt.bRequest);
            return self.setup_ack();
        }
        else if (pkt.bmRequestType == UsbSetupRequestType::HOST2DEV_INTERFACE_CLASS as u8)
        {
            //println!("usbd: invalid HOST2DEV_INTERFACE_CLASS bRequest {:x}", pkt.bRequest);
            return self.setup_ack();
        }
        else
        {
            //printf("usbd: unhandled bmRequestType %02x\n", pkt.bmRequestType);
        }

        return UsbdError::Success;
    }
    
    pub fn handle_control_req(&mut self) -> UsbdError
    {
        let mut result = UsbdError::Success;
        
        let epSetupStat = self.r32(USB2D_ENDPTSETUPSTAT);
        if ((epSetupStat & bit!(0)) != 0)
        {
            self.w32(USB2D_ENDPTSETUPSTAT, epSetupStat);

            let setupBuffer = to_u64ptr!(&self.get_ep_queue_head(UsbEpNum::CTRL_OUT as u8).setupBuffer);
            memcpy_iou32(to_u64ptr!(&self.usbfCtxt.setupPkt), setupBuffer, mem::size_of::<UsbSetupPacket>());
            result = self.setup_process_pkt(self.usbfCtxt.setupPkt);
        }
        
        return result;
    }
    
    pub fn handle_endpoints(&mut self) -> UsbdError
    {
        // Iterate by Tx/Rx pairs and allocate a pair
        let mut i: usize = USB_EP_CONFIGURABLE_BEGIN as usize;
        loop
        {
            if (i >= USBD_EPNUM_MAX as usize)
            {
                break;
            }
            
            if (!self.endpoints[i].isEnabled) { i += 1; continue; }
            
            let stat = self.ep_status(i as u8) as u8;
            
            if (self.ep_status(i as u8) == UsbEpStatus::TxfrComplete)
            {
                match (self.endpoints[i].complete_handler)
                {
                    Some(p) => {
                        p(self, i as u8);
                        self.ep_idle(i as u8);
                    },
                    None => {}
                }
            }
            
            if (self.ep_status(i as u8) == UsbEpStatus::TxfrIdle)
            {
                match (self.endpoints[i].idle_handler)
                {
                    Some(p) => {
                        p(self, i as u8);
                    },
                    None => {}
                }
            }
            
            if (self.ep_status(i as u8) == UsbEpStatus::TxfrFail)
            {
                match (self.endpoints[i].fail_handler)
                {
                    Some(p) => {
                        p(self, i as u8);
                        self.ep_idle(i as u8);
                    },
                    None => {}
                }
            }

            i += 1;
        }
        return UsbdError::Success;
    }
    
    pub fn register_complete_handler(&mut self, epNum: u8, handler: fn(&mut UsbDevice, u8))
    {
        self.endpoints[epNum as usize].complete_handler = Some(handler);
    }
    
    pub fn remove_complete_handler(&mut self, epNum: u8)
    {
        self.endpoints[epNum as usize].complete_handler = None;
    }
    
    pub fn register_idle_handler(&mut self, epNum: u8, handler: fn(&mut UsbDevice, u8))
    {
        self.endpoints[epNum as usize].idle_handler = Some(handler);
    }
    
    pub fn remove_idle_handler(&mut self, epNum: u8)
    {
        self.endpoints[epNum as usize].idle_handler = None;
    }
    
    pub fn register_fail_handler(&mut self, epNum: u8, handler: fn(&mut UsbDevice, u8))
    {
        self.endpoints[epNum as usize].fail_handler = Some(handler);
    }
    
    pub fn remove_fail_handler(&mut self, epNum: u8)
    {
        self.endpoints[epNum as usize].fail_handler = None;
    }
    
    pub fn register_setup_hook(&mut self, handler: fn(&mut UsbDevice, UsbSetupPacket)->bool) -> usize
    {
        let retval = self.setup_handlers.len();
        self.setup_handlers.push(handler);
        
        return retval;
    }
    
    pub fn remove_setup_hook(&mut self, handler: usize)
    {
        self.setup_handlers.remove(handler);
    }
    
    pub fn register_reset_hook(&mut self, handler: fn(&mut UsbDevice)) -> usize
    {
        let retval = self.reset_handlers.len();
        self.reset_handlers.push(handler);
        
        return retval;
    }
    
    pub fn setup_getdata(&mut self, dst: u64, len: u16) -> UsbdError
    {
        let result = self.ep_txfer_start(UsbEpNum::CTRL_OUT as u8, len as usize, true);
        if (dst != 0) {
            memcpy_iou32(dst, self.get_xferbuf(UsbEpNum::CTRL_OUT as u8), len as usize);
        }
        
        return result;
    }
    
    pub fn ep_tx(&mut self, endpoint: u8, buf: u64, len: usize, sync: bool) -> UsbdError
    {
        let mut _len = len;
        if (_len > USBD_XFERBUF_SIZE as usize) {
            _len = USBD_XFERBUF_SIZE as usize;
        }

        memcpy_iou32(self.get_xferbuf(endpoint), buf, _len);
        self.xferbuf_flush(endpoint, _len);

        return self.ep_txfer_start(endpoint, _len, sync);
    }
    
    pub fn ep_rx(&mut self, endpoint: u8, buf: u64, len: usize, sync: bool) -> UsbdError
    {
        let mut result = UsbdError::Success;

        let mut _len = len;
        if (_len > USBD_XFERBUF_SIZE as usize) {
            _len = USBD_XFERBUF_SIZE as usize;
        }

        memset_iou32(buf, 0, _len);

        result = self.ep_txfer_start(endpoint, _len, sync);

        if (result == UsbdError::Success && sync)
        {
            self.xferbuf_invalidate(endpoint, _len);
            memcpy_iou32(buf, self.get_xferbuf(endpoint), self.get_bytes_received(endpoint) as usize);
        }

        return result;
    }
}

static mut USBD: UsbDevice = UsbDevice::empty();

pub fn get_usbd() -> &'static mut UsbDevice
{
    unsafe
    {
        &mut USBD
    }
}

pub fn irq_usb()
{
    let mut result: UsbdError = UsbdError::Success;
    
    let usbd = get_usbd();

    // Check status, then write back to clear any pending interrupts
    let usbSts = usbd.r32(USB2D_USBSTS);
    usbd.w32(USB2D_USBSTS, usbSts);
    
    //if (!(usbSts & 0xFFFF))
    if (usbSts == 0) {
        return;
    }
    
    // A USB reset was requested
    if ((usbSts & USBSTS_USBRST) != 0)
    {
        println_uarta!("usbd: reset requested");
        usbd.halt_activity();
        
        // Run through reset handlers
        for i in 0..usbd.reset_handlers.len()
        {
            usbd.reset_handlers[i](usbd);
        }
        
        // Initialize USB context
        usbd.update_port_speed();
        
        result = usbd.init_controller();
        if (result != UsbdError::Success) {
            return;
        }
    }
    
    // Cable was reinserted
    if ((usbSts & USBSTS_USBPORT) != 0)
    {
        println_uarta!("usbd: cable reinserted");
    }
    
    // Check for incoming setup packets and handle them
    result = usbd.handle_control_req();
    if ((result != UsbdError::Success) && !usbd.is_enumeration_done()) {
        return;
    }
    
    // Don't talk to endpoints until we're enumerated
    if (!usbd.is_enumeration_done()) {
        return;
    }
    
    // Call endpoint handlers as appropriate
    result = usbd.handle_endpoints();
    if (result != UsbdError::Success) {
        return;
    }
    
    return;
}

pub fn usbd_recover() -> UsbdError
{
    let mut ret: UsbdError = UsbdError::Success;
    let usbd = get_usbd();
    
    println_uarta!("usbd: Begin init");
    usbd.init();
    debug_init();
    //cdc_init();
    
    println_uarta!("usbd: Begin init context");
    usbd.init_context();
    usbd.enable_clocks();
    usbd.init_controller();
    println_uarta!("usbd: USB controller initialized...");
    
    usbd.interrupt_en();

    return ret;
}

pub fn usbd_suspend()
{
    let mut clk_out_enb_l: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_CLK_OUT_ENB_L);
    let mut rst_dev_l_set: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_RST_DEV_L_SET);
    let mut rst_dev_l_clr: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_RST_DEV_L_CLR);
    let mut clk_out_enb_w: MMIOReg = MMIOReg::new(CLK_RST_CONTROLLER_CLK_OUT_ENB_W);
    let mut apbdev_pmc_usb_ao: MMIOReg = MMIOReg::new(APBDEV_PMC_USB_AO);
    
    rst_dev_l_set |= CLK_ENB_USBD;
    timer_wait(2);
    clk_out_enb_l &= !CLK_ENB_USBD;
    timer_wait(2);
    clk_out_enb_l |= bit!(8);
    clk_out_enb_w |= bit!(21);
    
    apbdev_pmc_usb_ao |= 0xC;
}

pub fn usbd_is_enumerated() -> bool
{
    let usbd = get_usbd();
    
    // keep compiler from optimizing this in a dumb way
    unsafe { asm!("add xzr, xzr, {0}", in(reg) &usbd); }
    
    return usbd.is_enumeration_done();
}

/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use crate::util::*;

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

pub const USBD_EPNUM_MAX:        u32 = (32);
pub const USBD_EPIDX_MAX:        u32 = (16);

pub const USBD_CTRL_PKT_MAX:     u32 = (64);

pub const USB_EPATTR_TTYPE_BULK: u32 = (2);
pub const USB_EPATTR_TTYPE_INTR: u32 = (3);

pub const CS_INTERFACE:  u32 = 0x24;
pub const USB_ST_HEADER: u32 = 0x00;
pub const USB_ST_CMF:    u32 = 0x01;
pub const USB_ST_ACMF:   u32 = 0x02;
pub const USB_ST_UF:     u32 = 0x06;

pub const USBD_XFERBUF_SIZE: u32 = (0x1000);

#[macro_use]
mod util {
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
            (!!(ep & bit!(0)))
        }
    }
    
    macro_rules! USB2D_EPIDX {
        ($ep:expr) => {
            (ep >> 1)
        }
    }
    
    macro_rules! USB2D_EPBIT {
        ($ep:expr) => {
            ((ep == USB_EP_ALL) ? 0xFFFFFFFF : (USB2D_IS_EP_TX!(ep) ? (bit!(16) << USB2D_EPIDX!(ep)) : (bit!(0) << USB2D_EPIDX!(ep))))
        }
    }
    
    macro_rules! USB2D_EPADDR_IS_TX {
        ($ep:expr) => {
            (epaddr & 0x80)
        }
    }
    
    macro_rules! USB2D_EPADDR_TO_EPNUM {
        ($ep:expr) => {
            ((epaddr & 0x7F) << 1 | (USB2D_EPADDR_IS_TX(epaddr) ? 1 : 0))
        }
    }
    
    macro_rules! USB2D_EPNUM_TO_EPADDR {
        ($ep:expr) => {
            ((epnum >> 1) | (USB2D_IS_EP_TX(epnum) ? 0 : 0x80))
        }
    }
    
    macro_rules! CTRL_EPT_MASK {
        ($ep:expr) => {
            (USB2D_IS_EP_TX!(ep) ? CTRL_TXT_MASK : CTRL_RXT_MASK)
        }
    }
    
    macro_rules! CTRL_EPT_CTRL {
        ($ep:expr) => {
            (USB2D_IS_EP_TX!(ep) ? CTRL_TXT_CTRL : CTRL_RXT_CTRL)
        }
    }
    
    macro_rules! CTRL_EPT_BULK {
        ($ep:expr) => {
            (USB2D_IS_EP_TX!(ep) ? CTRL_TXT_BULK : CTRL_RXT_BULK)
        }
    }
    
    macro_rules! CTRL_EPT_INTR {
        ($ep:expr) => {
            (USB2D_IS_EP_TX!(ep) ? CTRL_TXT_INTR : CTRL_RXT_INTR)
        }
    }
    
    macro_rules! CTRL_EPENABLE {
        ($ep:expr) => {
            (USB2D_IS_EP_TX!(ep) ? CTRL_TXENABLE : CTRL_RXENABLE)
        }
    }
    
    macro_rules! CTRL_EPRESET {
        ($ep:expr) => {
            (USB2D_IS_EP_TX!(ep) ? CTRL_TXRESET : CTRL_RXRESET)
        }
    }
    
    macro_rules! CTRL_EPSTALL {
        ($ep:expr) => {
            (USB2D_IS_EP_TX!(ep) ? CTRL_TXSTALL : CTRL_RXSTALL)
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
    USB_EP_CTRL_OUT = 0,
    // Control In endpoint number, mapped to ep0
    USB_EP_CTRL_IN  = 1,
    
    // Bulk out endpoint number, mapped to ep1
    USB_EP_BULK_OUT = 2,
    // Bulk In endpoint number, mapped to ep1
    USB_EP_BULK_IN  = 3,
    
    // All endpoints
    USB_EP_ALL      = -1,
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


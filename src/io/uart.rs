/*
 * Copyright (c) 2015-2021, SALT.
 * This file is part of HashtagBlessedII and is distributed under the 3-clause BSD license.
 * See LICENSE.md for terms of use.
 */

use crate::io::car::CAR_INFO_UART_A;
use crate::io::car::CAR_INFO_UART_B;
use crate::io::car::CAR_INFO_UART_C;
use crate::io::car::CAR_INFO_UART_D;
use crate::io::pinmux::*;
use crate::util::*;
use crate::io::timer::*;

const UART_OFFSETS: [u32; 5] = [0, 0x40, 0x200, 0x300, 0x400];

pub const UART_PADDR: u32 = (0x70006000);
pub const UART_VADDR: u64 = (0xffffffff70006000);
//pub const UART_BASE(index)                 ((void*)UART_PADDR + UART_OFFSETS[index])

// UART_IER_DLAB
pub const UART_IE_EORD:       u32 = bit!(5);
pub const UART_IE_RX_TIMEOUT: u32 = bit!(4);
pub const UART_IE_MSI:        u32 = bit!(3);
pub const UART_IE_RXS:        u32 = bit!(2);
pub const UART_IE_THR:        u32 = bit!(1);
pub const UART_IE_RHR:        u32 = bit!(0);
pub const UART_IE_ALL:        u32 = (UART_IE_EORD | UART_IE_RX_TIMEOUT | UART_IE_MSI | UART_IE_RXS | UART_IE_THR | UART_IE_RHR);

// UART_IIR_FCR
pub const UART_FCR_EN_FIFO: u32 = bit!(0);
pub const UART_RX_CLR:      u32 = bit!(1);
pub const UART_TX_CLR:      u32 = bit!(2);

// UART_LCR
pub const UART_DLAB_ENABLE:      u32 = bit!(7);
pub const UART_PARITY_EVEN:      u32 = bit!(5);
pub const UART_PARITY_ENABLE:    u32 = bit!(7);
pub const UART_STOP_BITS_DOUBLE: u32 = bit!(2);
pub const UART_WORD_LENGTH_8:    u32 = (3);

// UART_MCR
pub const UART_RTS_EN:          u32 = bit!(6);
pub const UART_CTS_EN:          u32 = bit!(5);
pub const UART_LOOPBACK_ENABLE: u32 = bit!(4);
pub const UART_FORCE_RTS_HI_LO: u32 = bit!(1);
pub const UART_FORCE_CTS_HI_LO: u32 = bit!(0);

// UART_LSR
pub const UART_RX_FIFO_EMPTY: u32 = bit!(9);
pub const UART_TX_FIFO_FULL:  u32 = bit!(8);
pub const UART_TMTY:          u32 = bit!(6);
pub const UART_RDR:           u32 = bit!(0);

// uart_irda_csr
pub const UART_BAUD_PULSE_4_14: u32 = bit!(6);
pub const UART_INVERT_RTS:      u32 = bit!(3);
pub const UART_INVERT_CTS:      u32 = bit!(2);
pub const UART_INVERT_TXD:      u32 = bit!(1);
pub const UART_INVERT_RXD:      u32 = bit!(0);
pub const UART_CSR_ALL:         u32 = (!0);

// UART_RX_FIFO_CFG
//pub const UART_RX_FIFO_TRIG(level) (level & 0x3F)

// Misc
//pub const UART_BAUDRATE_CALC(rate) (((8 * rate + 408000000) / (16 * rate)))

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum UARTDevicePort {
    UartA = 0,
    UartB = 1,
    UartC = 2,
    UartD = 3,
    UartE = 4,
}

pub fn uart_configure_a() {
    unsafe
    {
        PINMUX_AUX_UART1_TX_0.write_volatile(0);
        PINMUX_AUX_UART1_RX_0.write_volatile(PINMUX_INPUT);
        PINMUX_AUX_UART1_RTS_0.write_volatile(0);
        PINMUX_AUX_UART1_CTS_0.write_volatile(PINMUX_INPUT);
    }
    CAR_INFO_UART_A.enable();
}

pub fn uart_configure_b() {
    unsafe
    {
        PINMUX_AUX_UART2_TX_0.write_volatile(0);
        PINMUX_AUX_UART2_RX_0.write_volatile(PINMUX_INPUT);
        PINMUX_AUX_UART2_RTS_0.write_volatile(0);
        PINMUX_AUX_UART2_CTS_0.write_volatile(PINMUX_INPUT);
    }
    CAR_INFO_UART_B.enable();
}

pub fn uart_configure_c() {
    unsafe
    {
        PINMUX_AUX_UART3_TX_0.write_volatile(0);
        PINMUX_AUX_UART3_RX_0.write_volatile(PINMUX_INPUT);
        PINMUX_AUX_UART3_RTS_0.write_volatile(0);
        PINMUX_AUX_UART3_CTS_0.write_volatile(PINMUX_INPUT);
    }
    CAR_INFO_UART_C.enable();
}

pub fn uart_configure_d() {
    unsafe
    {
        PINMUX_AUX_UART4_TX_0.write_volatile(0);
        PINMUX_AUX_UART4_RX_0.write_volatile(PINMUX_INPUT);
        PINMUX_AUX_UART4_RTS_0.write_volatile(0);
        PINMUX_AUX_UART4_CTS_0.write_volatile(PINMUX_INPUT);
    }
    CAR_INFO_UART_D.enable();
}

pub struct UARTDevice
{
    port: UARTDevicePort,
    uart_thr_dlab: MMIOReg,
    uart_ier_dlab: MMIOReg,
    uart_iir_fcr: MMIOReg,
    uart_lcr: MMIOReg,
    uart_mcr: MMIOReg,
    uart_lsr: MMIOReg,
    uart_msr: MMIOReg,
    uart_spr: MMIOReg,
    uart_irda_csr: MMIOReg,
    uart_rx_fifo_cfg: MMIOReg,
    uart_mie: MMIOReg,
    uart_vendor_status: MMIOReg,
    uart_asr: MMIOReg,
}

impl UARTDevice
{
    pub fn new(port: UARTDevicePort) -> Self {
        let uart_base: u32 = UART_PADDR + UART_OFFSETS[port as usize];
    
        let mut retval: UARTDevice = UARTDevice {
            port: port,
            uart_thr_dlab: MMIOReg::new(uart_base + 0x0),
            uart_ier_dlab: MMIOReg::new(uart_base + 0x4),
            uart_iir_fcr: MMIOReg::new(uart_base + 0x8),
            uart_lcr: MMIOReg::new(uart_base + 0xC),
            uart_mcr: MMIOReg::new(uart_base + 0x10),
            uart_lsr: MMIOReg::new(uart_base + 0x14),
            uart_msr: MMIOReg::new(uart_base + 0x18),
            uart_spr: MMIOReg::new(uart_base + 0x1C),
            uart_irda_csr: MMIOReg::new(uart_base + 0x20),
            uart_rx_fifo_cfg: MMIOReg::new(uart_base + 0x24),
            uart_mie: MMIOReg::new(uart_base + 0x28),
            uart_vendor_status: MMIOReg::new(uart_base + 0x2C),
            uart_asr: MMIOReg::new(uart_base + 0x3C),
        };

        return retval;
    }
    
    pub fn init(&mut self, baudrate: u32)
    {
        match self.port {
            UARTDevicePort::UartA => uart_configure_a(),
            UARTDevicePort::UartB => uart_configure_b(),
            UARTDevicePort::UartC => uart_configure_c(),
            UARTDevicePort::UartD => uart_configure_d(),
            UARTDevicePort::UartE => {},
        }
        
        self.set_baudrate(baudrate);
        self.interrupt_disable(UART_IE_ALL);
        self.csr_set(UART_BAUD_PULSE_4_14);
    }
    
    pub fn shutdown(&mut self)
    {
        self.interrupt_disable(UART_IE_ALL);
        self.csr_unset(UART_CSR_ALL);
        self.uart_lcr |= UART_DLAB_ENABLE;
        self.uart_thr_dlab.w32(0);
        self.uart_ier_dlab.w32(0);
        self.uart_lcr &= !UART_DLAB_ENABLE;
        self.uart_lcr.w32(0);
        self.uart_ier_dlab.w32(0);
        self.uart_iir_fcr.w32(0);
        
        timer_wait(8000);
        
        self.uart_mcr.write(0);
        self.uart_msr.write(0);
        self.uart_irda_csr.write(0);
        self.uart_rx_fifo_cfg.write(1);
        self.uart_mie.write(0xf);

        match self.port {
            UARTDevicePort::UartA => CAR_INFO_UART_A.disable(),
            UARTDevicePort::UartB => CAR_INFO_UART_B.disable(),
            UARTDevicePort::UartC => CAR_INFO_UART_C.disable(),
            UARTDevicePort::UartD => CAR_INFO_UART_D.disable(),
            UARTDevicePort::UartE => {},
        }
        
        unsafe
        {
        PINMUX_AUX_UART1_TX_0.write_volatile(0x74);
        PINMUX_AUX_UART1_RX_0.write_volatile(0x74);
        PINMUX_AUX_UART1_RTS_0.write_volatile(0x74);
        PINMUX_AUX_UART1_CTS_0.write_volatile(0x74);
        }
    }
    
    pub fn set_baudrate(&mut self, baudrate: u32)
    {
        let baudrate_calc: u32 = (((8 * baudrate + 408000000) / (16 * baudrate)));

        self.uart_lcr |= UART_DLAB_ENABLE;
        self.uart_thr_dlab.write(baudrate_calc & 0xFF);
        self.uart_ier_dlab.write((baudrate_calc >> 8) & 0xFF);
        self.uart_lcr &= !UART_DLAB_ENABLE;
        self.uart_iir_fcr.write(UART_FCR_EN_FIFO | UART_RX_CLR | UART_TX_CLR | (3 << 4));

        // Perform one read
        self.uart_lsr.read();

        // Wait a bit
        timer_wait(8000);

        self.uart_lcr |= UART_WORD_LENGTH_8;
        self.uart_mcr.write(0);
        self.uart_msr.write(0);
        self.uart_irda_csr.write(0);
        self.uart_rx_fifo_cfg.write(1);
        self.uart_mie.write(0);
    }
    
    pub fn csr_set(&mut self, bits: u32)
    {
        loop
        {
            loop
            {
                if (self.uart_mie.read() == 0) { break; }
            }
            self.uart_irda_csr |= bits;
            
            if (self.uart_mie.read() == 0) { break; }
        }
    }
    
    pub fn csr_unset(&mut self, bits: u32)
    {
        loop
        {
            loop
            {
                if (self.uart_mie.read() == 0) { break; }
            }
            self.uart_irda_csr &= !bits;
            
            if (self.uart_mie.read() == 0) { break; }
        }
    }
    
    pub fn enable_rts(&mut self)
    {
        self.uart_mcr |= UART_RTS_EN;
    }
    
    pub fn enable_cts(&mut self)
    {
        self.uart_mcr |= UART_CTS_EN;
    }
    
    pub fn interrupt_enable(&mut self, interrupts: u32)
    {
        // Enable IER
        self.uart_lcr &= !UART_DLAB_ENABLE;

        // Unmask interrupts
        self.uart_ier_dlab |= interrupts;
    }
    
    pub fn interrupt_disable(&mut self, interrupts: u32)
    {
        // Enable IER
        self.uart_lcr &= !UART_DLAB_ENABLE;

        // Mask interrupts
        self.uart_ier_dlab &= !interrupts;
    }
    
    pub fn enable_double_stop_bits(&mut self)
    {
        self.uart_lcr |= UART_STOP_BITS_DOUBLE;
    }
    
    pub fn disable_double_stop_bits(&mut self)
    {
        self.uart_lcr &= !UART_STOP_BITS_DOUBLE;
    }
    
    pub fn write_str(&mut self, data: &str)
    {
        for byte in data.bytes() {
            while (self.uart_lsr.bits_set(UART_TX_FIFO_FULL)) {}

            self.uart_thr_dlab.write(byte as u32);
        }
    }
    
    pub fn write_bytes(&mut self, data: &[u8])
    {
        for byte in data {
            while (self.uart_lsr.bits_set(UART_TX_FIFO_FULL)) {}

            self.uart_thr_dlab.write(*byte as u32);
        }
    }
    
    pub fn wait_for_write(&mut self)
    {
        // Flush TX
        //self.uart_iir_fcr |= UART_TX_CLR;

        for i in 1..80 {
            if (self.uart_lsr.bits_set(UART_TMTY)) { break };
            timer_wait(1);
        }
        
        timer_wait(100);
    }
}

/*
u64 uart_lock = 0;
u64 uart_print_lock = 0;
char log_buf[0x200];

void uart_shutdown(int id)
{
    uart_interrupt_disable(id, UART_IE_ALL);
    uart_csr_unset(id, UART_CSR_ALL);

    switch(id)
    {
        case UART_A:
            car_disable_uart_a();
            break;
        case UART_B:
            car_disable_uart_b();
            break;
        case UART_C:
            car_disable_uart_c();
            break;
        case UART_D:
            car_disable_uart_d();
            break;
    }
}

int uart_read_blocking(int id, u8 *data, u32 size, int timeout)
{
    for (int i = 0; i < size; i++)
    {
        if (timeout)
        {
            if (uart_lsr & UART_RX_FIFO_EMPTY)
                timer_wait(timeout);

            if (uart_lsr & UART_RX_FIFO_EMPTY)
                return i;
        }
        else
        {
            while (uart_lsr & UART_RX_FIFO_EMPTY);
        }

        data[i] = uart_thr_dlab;
    }

    return size;
}

int uart_read_nonblocking(int id, u8 *data, u32 size)
{
    for (int i = 0; i < size; i++)
    {
        if (uart_lsr & UART_RX_FIFO_EMPTY)
            return i;

        data[i] = uart_thr_dlab;
    }

    return size;
}*/

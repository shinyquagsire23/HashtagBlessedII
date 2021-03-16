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

// UART_IER_DLAB_0_0
pub const UART_IE_EORD:       u32 = bit!(5);
pub const UART_IE_RX_TIMEOUT: u32 = bit!(4);
pub const UART_IE_MSI:        u32 = bit!(3);
pub const UART_IE_RXS:        u32 = bit!(2);
pub const UART_IE_THR:        u32 = bit!(1);
pub const UART_IE_RHR:        u32 = bit!(0);
pub const UART_IE_ALL:        u32 = (UART_IE_EORD | UART_IE_RX_TIMEOUT | UART_IE_MSI | UART_IE_RXS | UART_IE_THR | UART_IE_RHR);

// UART_IIR_FCR_0
pub const UART_FCR_EN_FIFO: u32 = bit!(0);
pub const UART_RX_CLR:      u32 = bit!(1);
pub const UART_TX_CLR:      u32 = bit!(2);

// UART_LCR_0
pub const UART_DLAB_ENABLE:      u32 = bit!(7);
pub const UART_PARITY_EVEN:      u32 = bit!(5);
pub const UART_PARITY_ENABLE:    u32 = bit!(7);
pub const UART_STOP_BITS_DOUBLE: u32 = bit!(2);
pub const UART_WORD_LENGTH_8:    u32 = (3);

// UART_MCR_0
pub const UART_RTS_EN:          u32 = bit!(6);
pub const UART_CTS_EN:          u32 = bit!(5);
pub const UART_LOOPBACK_ENABLE: u32 = bit!(4);
pub const UART_FORCE_RTS_HI_LO: u32 = bit!(1);
pub const UART_FORCE_CTS_HI_LO: u32 = bit!(0);

// UART_LSR_0
pub const UART_RX_FIFO_EMPTY: u32 = bit!(9);
pub const UART_TX_FIFO_FULL:  u32 = bit!(8);
pub const UART_TMTY:          u32 = bit!(6);
pub const UART_RDR:           u32 = bit!(0);

// UART_IRDA_CSR_0
pub const UART_BAUD_PULSE_4_14: u32 = bit!(6);
pub const UART_INVERT_RTS:      u32 = bit!(3);
pub const UART_INVERT_CTS:      u32 = bit!(2);
pub const UART_INVERT_TXD:      u32 = bit!(1);
pub const UART_INVERT_RXD:      u32 = bit!(0);
pub const UART_CSR_ALL:         u32 = (!0);

// UART_RX_FIFO_CFG_0
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
    UART_THR_DLAB: MMIOReg,
    UART_IER_DLAB: MMIOReg,
    UART_IIR_FCR: MMIOReg,
    UART_LCR: MMIOReg,
    UART_MCR: MMIOReg,
    UART_LSR: MMIOReg,
    UART_MSR: MMIOReg,
    UART_SPR: MMIOReg,
    UART_IRDA_CSR: MMIOReg,
    UART_RX_FIFO_CFG: MMIOReg,
    UART_MIE: MMIOReg,
    UART_VENDOR_STATUS: MMIOReg,
    UART_ASR: MMIOReg,
}

impl UARTDevice
{
    pub fn new(port: UARTDevicePort) -> Self {
        let uart_base: u32 = UART_PADDR + UART_OFFSETS[port as usize];
    
        let mut retval: UARTDevice = UARTDevice {
            port: port,
            UART_THR_DLAB: MMIOReg::new(uart_base + 0x0),
            UART_IER_DLAB: MMIOReg::new(uart_base + 0x4),
            UART_IIR_FCR: MMIOReg::new(uart_base + 0x8),
            UART_LCR: MMIOReg::new(uart_base + 0xC),
            UART_MCR: MMIOReg::new(uart_base + 0x10),
            UART_LSR: MMIOReg::new(uart_base + 0x14),
            UART_MSR: MMIOReg::new(uart_base + 0x18),
            UART_SPR: MMIOReg::new(uart_base + 0x1C),
            UART_IRDA_CSR: MMIOReg::new(uart_base + 0x20),
            UART_RX_FIFO_CFG: MMIOReg::new(uart_base + 0x24),
            UART_MIE: MMIOReg::new(uart_base + 0x28),
            UART_VENDOR_STATUS: MMIOReg::new(uart_base + 0x2C),
            UART_ASR: MMIOReg::new(uart_base + 0x3C),
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
        
        self.setBaudrate(baudrate);
        self.interruptDisable(UART_IE_ALL);
        self.csrSet(UART_BAUD_PULSE_4_14);
    }
    
    pub fn setBaudrate(&mut self, baudrate: u32)
    {
        let baudrate_calc: u32 = (((8 * baudrate + 408000000) / (16 * baudrate)));

        self.UART_LCR |= UART_DLAB_ENABLE;
        self.UART_THR_DLAB.write(baudrate_calc & 0xFF);
        self.UART_IER_DLAB.write((baudrate_calc >> 8) & 0xFF);
        self.UART_LCR &= !UART_DLAB_ENABLE;
        self.UART_IIR_FCR.write(UART_FCR_EN_FIFO | UART_RX_CLR | UART_TX_CLR | (3 << 4));

        // Perform one read
        self.UART_LSR.read();

        // Wait a bit
        timerWait(8000);

        self.UART_LCR |= UART_WORD_LENGTH_8;
        self.UART_MCR.write(0);
        self.UART_MSR.write(0);
        self.UART_IRDA_CSR.write(0);
        self.UART_RX_FIFO_CFG.write(1);
        self.UART_MIE.write(0);
    }
    
    pub fn csrSet(&mut self, bits: u32)
    {
        loop
        {
            loop
            {
                if (self.UART_MIE.read() == 0) { break; }
            }
            self.UART_IRDA_CSR |= bits;
            
            if (self.UART_MIE.read() == 0) { break; }
        }
    }
    
    pub fn csrUnset(&mut self, bits: u32)
    {
        loop
        {
            loop
            {
                if (self.UART_MIE.read() == 0) { break; }
            }
            self.UART_IRDA_CSR &= !bits;
            
            if (self.UART_MIE.read() == 0) { break; }
        }
    }
    
    pub fn enableRts(&mut self)
    {
        self.UART_MCR |= UART_RTS_EN;
    }
    
    pub fn enableCts(&mut self)
    {
        self.UART_MCR |= UART_CTS_EN;
    }
    
    pub fn interruptEnable(&mut self, interrupts: u32)
    {
        // Enable IER
        self.UART_LCR &= !UART_DLAB_ENABLE;

        // Unmask interrupts
        self.UART_IER_DLAB |= interrupts;
    }
    
    pub fn interruptDisable(&mut self, interrupts: u32)
    {
        // Enable IER
        self.UART_LCR &= !UART_DLAB_ENABLE;

        // Mask interrupts
        self.UART_IER_DLAB &= !interrupts;
    }
    
    pub fn enableDoubleStopBits(&mut self)
    {
        self.UART_LCR |= UART_STOP_BITS_DOUBLE;
    }
    
    pub fn disableDoubleStopBits(&mut self)
    {
        self.UART_LCR &= !UART_STOP_BITS_DOUBLE;
    }
    
    pub fn writeStr(&mut self, data: &str)
    {
        for byte in data.bytes() {
            while (self.UART_LSR.bits_set(UART_TX_FIFO_FULL)) {}

            self.UART_THR_DLAB.write(byte as u32);
        }
    }
    
    pub fn writeBytes(&mut self, data: &[u8])
    {
        for byte in data {
            while (self.UART_LSR.bits_set(UART_TX_FIFO_FULL)) {}

            self.UART_THR_DLAB.write(*byte as u32);
        }
    }
    
    pub fn waitForWrite(&mut self)
    {
        // Flush TX
        //self.UART_IIR_FCR |= UART_TX_CLR;

        for i in 1..80 {
            if (self.UART_LSR.bits_set(UART_TMTY)) { break };
            timerWait(1);
        }
        
        timerWait(100);
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
            if (UART_LSR & UART_RX_FIFO_EMPTY)
                timer_wait(timeout);

            if (UART_LSR & UART_RX_FIFO_EMPTY)
                return i;
        }
        else
        {
            while (UART_LSR & UART_RX_FIFO_EMPTY);
        }

        data[i] = UART_THR_DLAB;
    }

    return size;
}

int uart_read_nonblocking(int id, u8 *data, u32 size)
{
    for (int i = 0; i < size; i++)
    {
        if (UART_LSR & UART_RX_FIFO_EMPTY)
            return i;

        data[i] = UART_THR_DLAB;
    }

    return size;
}*/

/***************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **************************************************************************************************/

//! # Low-Level interface access to timer register
//!

use ruspiro_mmio_register::define_mmio_register;

// MMIO peripheral base address based on the target family provided with the custom target config file.
#[cfg(feature = "pi3")]
const PERIPHERAL_BASE: usize = 0x0_3F00_0000;
#[cfg(feature = "pi4_lowperi")]
const PERIPHERAL_BASE: usize = 0x0_FE00_0000;
#[cfg(feature = "pi4_highperi")]
const PERIPHERAL_BASE: usize = 0x4_7E00_0000;

// Base address of system timer MMIO register
#[allow(dead_code)]
const SYS_TIMER_BASE: usize = PERIPHERAL_BASE + 0x3000;
// Base address of ARM timer MMIO register
#[allow(dead_code)]
const ARM_TIMER_BASE: usize = PERIPHERAL_BASE + 0xB000;

// Define the MMIO timer register
define_mmio_register![
    /// system timer control register, keep in mind that actually only timer 1 and 3 are free on RPi
    pub SYS_TIMERCS<ReadWrite<u32>@(SYS_TIMER_BASE)> {
        /// system timer 0 match flag
        M0 OFFSET(0) [
            MATCH = 1,
            CLEAR = 0
        ],
        /// system timer 1 match flag
        M1 OFFSET(1) [
            MATCH = 1,
            CLEAR = 0
        ],
        /// system timer 2 match flag
        M2 OFFSET(2) [
            MATCH = 1,
            CLEAR = 0
        ],
        /// system timer 3 match flag
        M3 OFFSET(3) [
            MATCH = 1,
            CLEAR = 0
        ]
    },
    /// system timer free running counter lower 32Bit value
    pub SYS_TIMERCLO<ReadOnly<u32>@(SYS_TIMER_BASE + 0x04)>,
    /// system timer free running counter higher 32Bit value
    pub SYS_TIMERCHI<ReadOnly<u32>@(SYS_TIMER_BASE + 0x08)>,
    /// system timer compare value register
    pub SYS_TIMERC0<ReadWrite<u32>@(SYS_TIMER_BASE + 0x0C)>,
    pub SYS_TIMERC1<ReadWrite<u32>@(SYS_TIMER_BASE + 0x10)>,
    pub SYS_TIMERC2<ReadWrite<u32>@(SYS_TIMER_BASE + 0x14)>,
    pub SYS_TIMERC3<ReadWrite<u32>@(SYS_TIMER_BASE + 0x18)>,

    /// ARM timer load value that is put into the value register once it counted to 0
    /// Writing to this register will immediately load the value into the counter
    pub ARM_TIMERLOAD<ReadWrite<u32>@(ARM_TIMER_BASE + 0x400)>,
    /// ARM timer current counter value
    pub ARM_TIMERVALUE<ReadOnly<u32>@(ARM_TIMER_BASE + 0x404)>,
    /// ARM timer control register
    pub ARM_TIMERCTRL<ReadWrite<u32>@(ARM_TIMER_BASE + 0x408)> {
        /// width of the timer counter values
        WIDTH OFFSET(1) [
            _16Bit = 0,
            _32Bit = 1
        ],
        /// pre-scaler bits
        PRESCALER OFFSET(2) BITS(2) [
            CLOCK_DIV_1 = 0b00,
            CLOCK_DIV_16 = 0b01,
            CLOCK_DIV_256 = 0b10
        ],
        /// flag to enable timer interrupts beein raised
        IRQ OFFSET(5) [
            ENABLED = 1,
            DISABLED = 0
        ],
        /// flag to enable the ARM timer
        TIMER OFFSET(7) [
            ENABLED = 1,
            DISABLED = 0
        ],
        /// flag to indicate if timer should stop or keep running in debug halted mode
        DEBUG OFFSET(8) [
            STOP = 1,
            RUN = 0
        ],
        /// flag to enable the free-running counter
        FREERUN OFFSET(9) [
            ENABLED = 1,
            DISABLED = 0
        ],
        /// free running counter pre-scaler = FREQUENCY = SYS_CLOCK/(FR_PRESCALER+1)
        FR_PRESCALER OFFSET(16) BITS(8)
    },
    /// ARM timer interrupt acknowledge register
    pub ARM_TIMERACKN<WriteOnly<u32>@(ARM_TIMER_BASE + 0x40C)>,
    pub ARM_TIMERRAWIRQ<ReadOnly<u32>@(ARM_TIMER_BASE + 0x410)> {
        PENDING OFFSET(0) [
            SET = 1,
            CLEAR = 0
        ]
    },
    /// masked interrupt assertion value (defacto = RAWIRQ logical AND IRQENABLE)
    pub ARM_TIMERMASKIRQ<ReadOnly<u32>@(ARM_TIMER_BASE + 0x414)> {
        FLAG OFFSET(0) [
            NOT_ASSERTED = 0,
            ASSERTED = 1
        ]
    },
    /// copy of the ARM timer load value register. But writing a value to this register does not trigger
    /// immediate re-load of the timer value - it's only loaded once the counter reaches 0
    pub ARM_TIMERRELOAD<ReadWrite<u32>@(ARM_TIMER_BASE + 0x418)>,
    /// ARM timer pre-devide value, timer_clock = apb_clock/(pre_devider + 1), default value = 0x7d (125), gives a divide by 126
    pub ARM_TIMERPREDIV<ReadWrite<u32>@(ARM_TIMER_BASE + 0x41C)> {
        VALUE OFFSET(0) BITS(10)
    },
    /// ARM timer free running counter value
    pub ARM_TIMERFRCOUNTER<ReadOnly<u32>@(ARM_TIMER_BASE + 0x420)>
];

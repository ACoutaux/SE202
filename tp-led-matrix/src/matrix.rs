//! This module builds matrix object and implements associated functions

use crate::{Color, Image};
use stm32l4xx_hal::gpio::Speed::VeryHigh;
use stm32l4xx_hal::gpio::*;
use stm32l4xx_hal::prelude::_embedded_hal_blocking_delay_DelayMs;
use stm32l4xx_hal::rcc::Clocks;

pub struct Matrix {
    sb: PC5<Output<PushPull>>,
    lat: PC4<Output<PushPull>>,
    rst: PC3<Output<PushPull>>,
    sck: PB1<Output<PushPull>>,
    sda: PA4<Output<PushPull>>,
    c0: PB2<Output<PushPull>>,
    c1: PA15<Output<PushPull>>,
    c2: PA2<Output<PushPull>>,
    c3: PA7<Output<PushPull>>,
    c4: PA6<Output<PushPull>>,
    c5: PA5<Output<PushPull>>,
    c6: PB0<Output<PushPull>>,
    c7: PA3<Output<PushPull>>,
}

/// Implements functions for matrix structure
impl Matrix {
    /// Create a new matrix from the control registers and the individual
    /// unconfigured pins. SB and LAT will be set high by default, while
    /// other pins will be set low. After 100ms, RST will be set high, and
    /// the bank 0 will be initialized by calling `init_bank0()` on the
    /// newly constructed structure.
    /// The pins will be set to very high speed mode.
    #[allow(clippy::too_many_arguments)] // Necessary to avoid a clippy warning
    /// Creates a new matrix
    pub fn new(
        pa2: PA2<Analog>,
        pa3: PA3<Analog>,
        pa4: PA4<Analog>,
        pa5: PA5<Analog>,
        pa6: PA6<Analog>,
        pa7: PA7<Analog>,
        pa15: PA15<Alternate<PushPull, 0>>,
        pb0: PB0<Analog>,
        pb1: PB1<Analog>,
        pb2: PB2<Analog>,
        pc3: PC3<Analog>,
        pc4: PC4<Analog>,
        pc5: PC5<Analog>,
        gpioa_moder: &mut MODER<'A'>,
        gpioa_otyper: &mut OTYPER<'A'>,
        gpiob_moder: &mut MODER<'B'>,
        gpiob_otyper: &mut OTYPER<'B'>,
        gpioc_moder: &mut MODER<'C'>,
        gpioc_otyper: &mut OTYPER<'C'>,
        clocks: Clocks,
    ) -> Self {
        // Use .into_push_pull_output_in_state(…) to set an initial state on pins
        let mut init_matrix = Matrix {
            sb: pc5
                .into_push_pull_output_in_state(gpioc_moder, gpioc_otyper, PinState::High)
                .set_speed(VeryHigh),
            lat: pc4
                .into_push_pull_output_in_state(gpioc_moder, gpioc_otyper, PinState::High)
                .set_speed(VeryHigh),
            rst: pc3
                .into_push_pull_output_in_state(gpioc_moder, gpioc_otyper, PinState::Low)
                .set_speed(VeryHigh),
            sck: pb1
                .into_push_pull_output_in_state(gpiob_moder, gpiob_otyper, PinState::Low)
                .set_speed(VeryHigh),
            sda: pa4
                .into_push_pull_output_in_state(gpioa_moder, gpioa_otyper, PinState::Low)
                .set_speed(VeryHigh),
            c0: pb2
                .into_push_pull_output_in_state(gpiob_moder, gpiob_otyper, PinState::Low)
                .set_speed(VeryHigh),
            c1: pa15
                .into_push_pull_output_in_state(gpioa_moder, gpioa_otyper, PinState::Low)
                .set_speed(VeryHigh),
            c2: pa2
                .into_push_pull_output_in_state(gpioa_moder, gpioa_otyper, PinState::Low)
                .set_speed(VeryHigh),
            c3: pa7
                .into_push_pull_output_in_state(gpioa_moder, gpioa_otyper, PinState::Low)
                .set_speed(VeryHigh),
            c4: pa6
                .into_push_pull_output_in_state(gpioa_moder, gpioa_otyper, PinState::Low)
                .set_speed(VeryHigh),
            c5: pa5
                .into_push_pull_output_in_state(gpioa_moder, gpioa_otyper, PinState::Low)
                .set_speed(VeryHigh),
            c6: pb0
                .into_push_pull_output_in_state(gpiob_moder, gpiob_otyper, PinState::Low)
                .set_speed(VeryHigh),
            c7: pa3
                .into_push_pull_output_in_state(gpioa_moder, gpioa_otyper, PinState::Low)
                .set_speed(VeryHigh),
        };

        let mut x = stm32l4xx_hal::delay::DelayCM::new(clocks);
        x.delay_ms(100u8);

        init_matrix.rst.set_high();

        init_matrix.init_bank0();

        init_matrix
    }

    /// Make a brief high pulse of the SCK pin
    fn pulse_sck(&mut self) {
        self.sck.set_high();
        self.sck.set_low();
    }

    /// Make a brief low pulse of the LAT pin
    fn pulse_lat(&mut self) {
        self.lat.set_low();
        self.lat.set_high();
    }

    /// Set the given row output in the chosen state
    fn row(&mut self, row: usize, state: PinState) {
        match row {
            1 => self.c0.set_state(state),
            2 => self.c1.set_state(state),
            3 => self.c2.set_state(state),
            4 => self.c3.set_state(state),
            5 => self.c4.set_state(state),
            6 => self.c5.set_state(state),
            7 => self.c6.set_state(state),
            8 => self.c7.set_state(state),
            _ => self.c7.set_state(state),
        }
    }

    /// Send a byte on SDA starting with the MSB and pulse SCK high after each bit
    fn send_byte(&mut self, pixel: u8) {
        for i in (0..8).rev() {
            self.sda.set_state((pixel & (1 << i) != 0).into());
            self.pulse_sck();
        }
    }

    /// Send a full row of bytes in BGR order and pulse LAT low. Gamma correction
    /// must be applied to every pixel before sending them. The previous row must
    /// be deactivated and the new one activated.
    pub fn send_row(&mut self, row: usize, pixels: &[Color]) {
        for (i, pixel) in pixels.iter().map(Color::gamma_correct).rev().enumerate() {
            self.send_byte(pixel.b);
            self.send_byte(pixel.g);
            if i == 4 {
                self.row((row + 7) % 8, PinState::Low); //turn off row at 5e beetween bg and r send
            }
            self.send_byte(pixel.r);
        }
        self.pulse_lat();
        self.row(row, PinState::High);
    }

    /// Initialize bank0 by temporarily setting SB to low and sending 144 one bits,
    /// pulsing SCK high after each bit and pulsing LAT low at the end. SB is then
    /// restored to high.
    fn init_bank0(&mut self) {
        self.sb.set_low();
        for _ in 1..=144 {
            self.sda.set_state(PinState::High);
            self.pulse_sck();
        }
        self.pulse_lat();
        self.sb.set_high();
    }

    /// Display a full image, row by row, as fast as possible.
    pub fn display_image(&mut self, image: &Image) {
        // Do not forget that image.row(n) gives access to the content of row n,
        // and that self.send_row() uses the same format.
        for i in 1..=8 {
            self.send_row(i, image.row(i));
        }
    }
}

//! This module in the main function and displays images in the matrix

#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use defmt_rtt as _;
use dwt_systick_monotonic::DwtSystick;
use dwt_systick_monotonic::ExtU32;
use panic_probe as _;
use stm32l4xx_hal::pac::USART1;
use stm32l4xx_hal::serial::{Config, Event, Rx, Serial};
use stm32l4xx_hal::{pac, prelude::*};
use tp_led_matrix::{matrix::Matrix, Color, Image};

use heapless::pool::{Box, Node, Pool};

#[rtic::app(device = stm32l4xx_hal::pac, dispatchers = [USART2,USART3])]
mod app {

    use super::*;

    #[shared]
    struct Shared {
        next_image: Option<Box<Image>>,
        pool: Pool<Image>,
    }

    #[local]
    struct Local {
        matrix: Matrix,
        usart1_rx: Rx<USART1>,
        current_image: Box<Image>,
        rx_image: Box<Image>,
    }

    #[init]
    /// Init ports and clocks and local shared structures
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("defmt correctly initialized");

        // Init hardware
        let mut cp = cx.core;
        let dp = cx.device;

        // Get high-level representations of hardware modules
        let mut rcc = dp.RCC.constrain();
        let mut flash = dp.FLASH.constrain();
        let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

        // Setup the clocks at 80MHz using HSI (by default since HSE/MSI are not configured).
        // The flash wait states will be configured accordingly.
        let clocks = rcc.cfgr.sysclk(80.MHz()).freeze(&mut flash.acr, &mut pwr);

        // Transfer GPIO to the HAL
        let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
        let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);
        let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);

        let rx =
            gpiob
                .pb7
                .into_alternate::<7>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl); //configure reception port pb7
        let tx =
            gpiob
                .pb6
                .into_alternate::<7>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl); //configure transmission port pb6

        let mut struct_serial_config = stm32l4xx_hal::serial::Config::default(); //default structure Config
        struct_serial_config = struct_serial_config.baudrate(38400.bps()); //default structure Config with correct baudrate

        // Config serial port with clocks and usart1
        let mut port_serie = Serial::usart1(
            dp.USART1,
            (tx, rx),
            struct_serial_config,
            clocks,
            &mut rcc.apb2,
        );

        port_serie.listen(Event::Rxne); //triggers an interrpution when a character is received

        let (_, usart1_rx) = port_serie.split(); //get received character

        // Init matrix object
        let matrix = Matrix::new(
            gpioa.pa2,
            gpioa.pa3,
            gpioa.pa4,
            gpioa.pa5,
            gpioa.pa6,
            gpioa.pa7,
            gpioa.pa15,
            gpiob.pb0,
            gpiob.pb1,
            gpiob.pb2,
            gpioc.pc3,
            gpioc.pc4,
            gpioc.pc5,
            &mut gpioa.moder,
            &mut gpioa.otyper,
            &mut gpiob.moder,
            &mut gpiob.otyper,
            &mut gpioc.moder,
            &mut gpioc.otyper,
            clocks,
        );

        let mut mono = DwtSystick::new(&mut cp.DCB, cp.DWT, cp.SYST, 80_000_000);
        //let image = Image::default();
        //let image2 = Image::default();

        display::spawn(mono.now()).unwrap();

        //rotate_image::spawn(0).unwrap();

        // Init structure shared and local
        let pool: Pool<Image> = Pool::new();
        unsafe {
            static mut MEMORY: MaybeUninit<[Node<Image>; 3]> = MaybeUninit::uninit();
            pool.grow_exact(&mut MEMORY); // static mut access is unsafe
        }
        let current_image = pool.alloc().unwrap().init(Image::default());
        let rx_image = pool.alloc().unwrap().init(Image::default());
        let next_image = None;

        (
            Shared { next_image, pool },
            Local {
                matrix,
                usart1_rx,
                current_image,
                rx_image,
            },
            init::Monotonics(mono),
        )
    }

    #[task(local = [matrix, current_image, next_line: usize = 1],shared = [next_image,pool], priority = 2)] //start to 1 because row() is implemented for strict positive numbers in image.rs
    /// Displays image with matrix row by row
    fn display(mut cx: display::Context, at: Instant) {
        // Display line next_line (cx.local.next_line) of
        // the image (cx.local.image) on the matrix (cx.local.matrix).
        // All those are mutable references.
        /*cx.shared.image.lock(|image| {
            cx.local.matrix.send_row(*cx.local.next_line, image.row(*cx.local.next_line)); //test with first line of gradient image
        });*/

        if *cx.local.next_line == 1 {
            cx.shared.next_image.lock(|next_image| {
                if next_image.is_some() {
                    cx.shared.pool.lock(|pool| {
                        if let Some(mut image) = next_image.take() {
                            core::mem::swap(cx.local.current_image, &mut image);
                            pool.free(image);
                        }
                    });
                }
            });
        }

        //Sends current_row to matrix to be displayed
        cx.local.matrix.send_row(
            *cx.local.next_line,
            cx.local.current_image.row(*cx.local.next_line),
        );

        // Increment next_line up to 8 and wraparound to 1
        if *cx.local.next_line < 8 {
            *cx.local.next_line = *cx.local.next_line + 1;
        } else {
            *cx.local.next_line = 1;
        }

        //Displays rows evry period
        let time_to_disp = at + 1.secs() / (8 * 60);
        display::spawn_at(time_to_disp, time_to_disp).unwrap();
    }

    #[idle()]
    /// When no task is currently running, infinite loop maintains program
    fn idle(_cx: idle::Context) -> ! {
        loop {}
    }

    #[task(binds = USART1, local = [usart1_rx, rx_image, next_pos: usize = 0], shared = [next_image,pool])]
    /// Manages the byte received and light up a R G B led depending on received byte value
    fn receive_byte(cx: receive_byte::Context) {
        let next_pos: &mut usize = cx.local.next_pos;
        if let Ok(b) = cx.local.usart1_rx.read() {
            // Handle the incoming byte according to the SE203 protocol
            // and update next_image
            // Do not forget that next_image.as_mut() might be handy here!

            if b == 0xff {
                // Return to position 0 case
                *next_pos = 0;
            } else if *next_pos < 3 * 64 {
                let colonne = (*next_pos % 24) / 3;
                let ligne = *next_pos / 24;

                // Assigns R G B led for one pixel
                match *next_pos % 3 {
                    0 => cx.local.rx_image[(ligne + 1, colonne + 1)].r = b,
                    1 => cx.local.rx_image[(ligne + 1, colonne + 1)].g = b,
                    2 => cx.local.rx_image[(ligne + 1, colonne + 1)].b = b,
                    _ => panic!("Indice RGB hors de 0 1 2"),
                }

                *next_pos += 1; //update next position

                // If the received image is complete, make it available to
                // the display task.
                if *next_pos == 3 * 64 {
                    // max position
                    (cx.shared.next_image, cx.shared.pool).lock(|next_image, pool| {
                        if let Some(image_nt_displayed) = next_image.take() {
                            pool.free(image_nt_displayed);
                        }
                        let mut future_image =
                            pool.alloc().unwrap().init(Image::gradient(Color::BLUE));

                        core::mem::swap(&mut future_image, cx.local.rx_image);

                        *next_image = Some(future_image);
                    });

                    // Next position reset
                    *next_pos = 0;
                }
            }
        }
    }

    /*
    #[task(shared = [image])]
    fn rotate_image(mut cx: rotate_image::Context, color_index: usize) {
        cx.shared.image.lock(|image| {
            *image = match color_index {
                0 => Image::gradient(Color::RED),
                1 => Image::gradient(Color::GREEN),
                2 => Image::gradient(Color::BLUE),
                _ => Image::default()
            }
        });


        rotate_image::spawn_after(1.secs(),(color_index+1)%3).unwrap();
    }
    */

    #[monotonic(binds = SysTick, default = true)]
    type MyMonotonic = DwtSystick<80_000_000>;
    type Instant = <MyMonotonic as rtic::Monotonic>::Instant;
}

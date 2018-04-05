#![deny(unsafe_code)]
#![deny(warnings)]
#![allow(unused_imports)]
#![no_std]

#![feature(lang_items)]
#![feature(nll)]
#![feature(proc_macro)]

#[macro_use]
extern crate cortex_m;
extern crate cortex_m_rtfm as rtfm;
extern crate embedded_hal;
#[macro_use(block)]
extern crate nb;
extern crate serial_mp3_player_driver as player;
extern crate stm32f103xx_hal as hal;
//extern crate stm32f103xx;

use cortex_m::asm;
use cortex_m::peripheral::{DWT, ITM};
use hal::delay::Delay;
use hal::gpio::{Alternate, Floating, Input, PushPull};
use hal::gpio::gpioa::{PA0, PA10, PA2, PA3, PA9};
use hal::prelude::*;
use hal::serial::{Pins, Rx, Serial, Tx};
use hal::stm32f103xx;
use hal::stm32f103xx::GPIOA;
use hal::stm32f103xx::USART1;
use hal::stm32f103xx::USART2;
use player::Mp3Player;
use rtfm::{app, Resource, Threshold};


app! {
    device: stm32f103xx,

    resources: {
        static SLEEP: u32 = 0;
        static EXTI: stm32f103xx::EXTI;
        static ITM: ITM;
        static PLAYER: Mp3Player<Tx<USART1>>;
        static RX: Rx<USART2>;
        static TX: Tx<USART2>;
        static INT: PA0<Input<Floating>>;
    },

    idle: {
        resources: [SLEEP],
    },

    tasks: {
        EXTI0: {
            path: exti0,
            resources: [EXTI, ITM, PLAYER, RX, TX, INT],
        },

        SYS_TICK: {
            path: sys_tick,
            resources: [SLEEP, ITM],
        },
    },
}



fn init(p: init::Peripherals, _r: init::Resources) -> init::LateResources {
    //let cp = cortex_m::Peripherals::take().unwrap();
    //let dp = stm32f103xx::Peripherals::take().unwrap();

    let dp: stm32f103xx::Peripherals = p.device;

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let _channels = dp.DMA1.split(&mut rcc.ahb);

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut _gpiob = dp.GPIOB.split(&mut rcc.apb2);

    // USART1
    // let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    // let rx = gpioa.pa10;

    // USART1
    // let tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
    // let rx = gpiob.pb7;

    // USART2
    // let tx = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    // let rx = gpioa.pa3;

    // USART1
    let tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx = gpioa.pa10;

    let serial = Serial::usart1(
        dp.USART1,
        (tx, rx),
        &mut afio.mapr,
        9_600.bps(),
        clocks,
        &mut rcc.apb2,
    );

    let (tx, _) = serial.split();

    let pl = Mp3Player::new(tx);

    // USART2
    let txdbg = gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl);
    let rxdbg = gpioa.pa3;

    let serial_dbg = Serial::usart2(
        dp.USART2,
        (txdbg, rxdbg),
        &mut afio.mapr,
        9_600.bps(),
        clocks,
        &mut rcc.apb1,
    );

    //let mut delay = Delay::new(dp.core.SYST, clocks);


    // configure EXTI0 interrupt
    // FIXME turn this into a higher level API
    let int = gpioa.pa0.into_floating_input(&mut gpioa.crl);
    dp.EXTI.imr.write(|w| w.mr0().set_bit()); // unmask the interrupt (EXTI)
    dp.EXTI.ftsr.write(|w| w.tr0().set_bit()); // trigger interrupt on falling edge
    // TODO: bind the EXTIn handler and enable EXTIn in the NVIC


    let (tx, rx) = serial_dbg.split();

    init::LateResources {
        EXTI: dp.EXTI,
        ITM: p.core.ITM,
        PLAYER: pl,
        RX: rx,
        TX: tx,
        INT: int
    }
}


fn idle(t: &mut Threshold, mut r: idle::Resources) -> ! {
    loop {
        rtfm::atomic(t, |t| {
            let before = DWT::get_cycle_count();
            rtfm::wfi();
            let after = DWT::get_cycle_count();

            *r.SLEEP.borrow_mut(t) += after.wrapping_sub(before);
        });

        // interrupts are serviced here
    }
}

fn exti0(_t: &mut Threshold, mut r: EXTI0::Resources) {
    let mut pl = r.PLAYER;
    let _stim = &mut r.ITM.stim[0];
    let mut tx = r.TX;
    let _rx = r.RX;


    let bytes = "exti0 triggered\n";
    for char in bytes.as_bytes().iter() {
        block!(tx.write(*char)).ok();
    }

    pl.set_volume(30).ok();
    pl.play_with_folder_and_file_name(1, 1).ok();
//    let mut list = [0u8; 18];
//    let mut idx: usize = 1;
//    for i in 1..10 {
//        list[idx] = 0x01;
//        list[idx + 1] = i as u8;
//        idx += 2;
//    }
//    pl.play_combined(&list).ok();
    //pl.play_combined(&[1, 1, 1, 2, 1, 3, 1, 4, 1, 5, 1, 6, 1, 7, 1, 8, 1, 9]).ok();
}

fn sys_tick(_t: &mut Threshold, mut r: SYS_TICK::Resources) {
    let _stim = &mut r.ITM.stim[1];

    iprint!(_stim, "{}\n", *r.SLEEP);

    *r.SLEEP = 0;
}
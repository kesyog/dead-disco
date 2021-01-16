#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use dead_disco::leds::DiscoLeds;
use hal::{interrupt, prelude::*, stm32 as pac};
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
use stm32f3xx_hal as hal;

// STM32F3DISCOVERY
// STLINK Virtual COM port: USART1 (PC4/PC5)
// LEDS:
//   N: LD3: PE9
//   NW: LD4: PE8
//   W: LD6: PE15
//   SW: LD8: PE14
//   S: LD10: PE13
//   SE: LD9: PE12
//   E: LD7: PE11
//   NE: LD5: PE10
// User button: PA0
// Compass: I2C
// Gyroscope: SPI

struct LedRoulette {
    leds: DiscoLeds,
    timer: hal::timer::Timer<pac::TIM2>,
}

static LEDS: Mutex<RefCell<Option<LedRoulette>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let _cp = cortex_m::peripheral::Peripherals::take().unwrap();
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM2);
    }

    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.sysclk(8.mhz()).freeze(&mut flash.acr);

    let gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let leds = DiscoLeds::new(gpioe);

    let mut timer = stm32f3xx_hal::timer::Timer::tim2(dp.TIM2, 20.hz(), clocks, &mut rcc.apb1);

    cortex_m::interrupt::free(|cs| {
        timer.listen(hal::timer::Event::Update);
        LEDS.borrow(cs).replace(Some(LedRoulette { leds, timer }));
    });

    loop {
        cortex_m::asm::wfi();
    }
}

#[interrupt]
fn TIM2() {
    static mut CURRENT_IDX: u8 = 0;
    // Toggle between two states
    static mut PING_PONG: bool = true;

    cortex_m::interrupt::free(|cs| {
        let mut state = LEDS.borrow(cs).borrow_mut();
        let state = state.as_mut().unwrap();
        if *PING_PONG {
            state.leds[*CURRENT_IDX].set_high().ok();
        } else {
            let prev_idx = if *CURRENT_IDX == 0 {
                7
            } else {
                *CURRENT_IDX - 1
            };
            state.leds[prev_idx].set_low().ok();
            *CURRENT_IDX = if *CURRENT_IDX < 7 {
                *CURRENT_IDX + 1
            } else {
                0
            };
        }
        *PING_PONG = !*PING_PONG;
        state.timer.clear_update_interrupt_flag();
    });
}

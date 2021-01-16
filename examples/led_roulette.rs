#![no_std]
#![no_main]

use core::iter::Iterator;
use cortex_m_rt::entry;
use dead_disco::leds::DiscoLeds;
use hal::{prelude::*, stm32 as pac};
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

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let clocks = rcc.cfgr.sysclk(16.mhz()).freeze(&mut flash.acr);

    let gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let mut leds = DiscoLeds::new(gpioe);
    let mut delay = hal::delay::Delay::new(cp.SYST, clocks);

    for current_idx in (0..8).cycle() {
        let prev_idx = if current_idx == 0 { 7 } else { current_idx - 1 };
        leds[current_idx].set_high().ok();
        delay.delay_ms(50_u16);
        leds[prev_idx].set_low().ok();
        delay.delay_ms(50_u16);
    }

    panic!();
}

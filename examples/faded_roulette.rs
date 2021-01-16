#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use dead_disco::leds::{DiscoLeds, GAMMA};
use hal::{interrupt, prelude::*, stm32 as pac};
use num_traits::float::FloatCore;
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
    let clocks = rcc
        .cfgr
        // Using the external oscillator
        // Set the frequency to that of the external oscillator
        .use_hse(8.mhz())
        // Set the frequency for the AHB bus,
        // which the root of every following clock peripheral
        .hclk(72.mhz())
        // The sysclk is equivalent to the core clock
        .sysclk(72.mhz())
        // The following are peripheral clocks, which are both
        // needed to configure specific peripherals.
        // Looking at the peripheral function parameters
        // should give more insight, which peripheral clock is needed.
        .pclk1(36.mhz())
        .pclk2(72.mhz())
        // Freeze / apply the configuration and setup all clocks
        .freeze(&mut flash.acr);

    let gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let leds = DiscoLeds::new(gpioe);

    let mut timer = stm32f3xx_hal::timer::Timer::tim2(dp.TIM2, 100.khz(), clocks, &mut rcc.apb1);

    cortex_m::interrupt::free(|cs| {
        timer.listen(hal::timer::Event::Update);
        LEDS.borrow(cs).replace(Some(LedRoulette { leds, timer }));
    });

    loop {
        cortex_m::asm::wfi();
    }
}

struct LedState {
    /// index into LED array
    index: u8,
    /// duty cycle on a 0-255 scale
    duty_cycle: f32,
}

impl LedState {
    fn step_duty(&mut self, step: f32) {
        self.duty_cycle += step;
        if self.duty_cycle > 255.0_f32 {
            self.duty_cycle = 255.0;
        }
    }

    fn duty(&self) -> f32 {
        self.duty_cycle
    }

    fn prev_led_idx(&self) -> u8 {
        if self.index > 0 {
            self.index - 1
        } else {
            7
        }
    }

    fn prev_led_duty(&self) -> f32 {
        255.0_f32 - self.duty_cycle
    }
}

/// Fade LED's on and off sequentially.
///
/// Use timer periods as a time base, tracked by COUNT
/// LED's are PWMed with a period of 256 timer periods so that we can set duty cycle with 8-bit
/// resolution.
/// Potential huge future improvement: most of the time this interrupt runs, nothing happens. We
/// should be able to use the output capture interrupts for duty cycle such that we only interrupt
/// when we have to.
///
/// At any given point in the cycle, one LED is fading in. Define this to be the "active" LED.
/// Another LED--the last LED to be active---is fading out.
#[interrupt]
fn TIM2() {
    static mut ACTIVE_LED: LedState = LedState {
        index: 0,
        duty_cycle: 0.0,
    };
    static mut COUNT: u32 = 0;

    // How long each LED will fade in (and out), in units of timer periods
    const FADE_IN_DURATION: u32 = 25000;

    *COUNT += 1;

    cortex_m::interrupt::free(|cs| {
        let mut state = LEDS.borrow(cs).borrow_mut();
        let state = state.as_mut().unwrap();

        let t = (*COUNT % 256) as u8;
        if t == 0 {
            const DUTY_STEP: f32 = 255.0 / (FADE_IN_DURATION as f32 / 256.0);
            ACTIVE_LED.step_duty(DUTY_STEP);

            if GAMMA[ACTIVE_LED.duty().round() as usize] != 0 {
                state.leds[ACTIVE_LED.index].set_high().ok();
            }
            if GAMMA[ACTIVE_LED.prev_led_duty().round() as usize] != 0 {
                state.leds[ACTIVE_LED.prev_led_idx()].set_high().ok();
            }
        }

        if t == GAMMA[ACTIVE_LED.duty().round() as usize] {
            state.leds[ACTIVE_LED.index].set_low().ok();
        }
        if t == GAMMA[ACTIVE_LED.prev_led_duty().round() as usize] {
            state.leds[ACTIVE_LED.prev_led_idx()].set_low().ok();
        }
        state.timer.clear_update_interrupt_flag();
    });

    // Finished fading in the active LED. Time to start fading it out and fading in the next LED
    if *COUNT == FADE_IN_DURATION {
        ACTIVE_LED.index += 1;
        if ACTIVE_LED.index == 8 {
            ACTIVE_LED.index = 0;
        }
        ACTIVE_LED.duty_cycle = 0.0;
        *COUNT = 0
    }
}

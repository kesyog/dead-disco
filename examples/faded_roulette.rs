#![no_std]
#![no_main]
/// LED roulette with fade on/off using the timer peripheral's output capture
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

struct LedRoulette<'a> {
    leds: DiscoLeds,
    timer: &'a mut pac::tim2::RegisterBlock,
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

    // Frequency set to give
    let mut tim2_hal = stm32f3xx_hal::timer::Timer::tim2(dp.TIM2, 500.hz(), clocks, &mut rcc.apb1);

    cortex_m::interrupt::free(|cs| {
        tim2_hal.listen(hal::timer::Event::Update);
        let tim2 = unsafe { &mut *(pac::TIM2::ptr() as *mut pac::tim2::RegisterBlock) };
        // Enable compare 1 and 2 interrupts
        tim2.dier
            .modify(|_, w| w.cc1ie().set_bit().cc2ie().set_bit());
        // Set initial duty cycle of previously-active LED to 100%
        tim2.ccr2.write(|w| w.ccr().bits(u32::MAX));
        // Enable buffering of new duty cycle values to avoid glitches
        tim2.ccmr1_output_mut()
            .modify(|_, w| w.oc1pe().set_bit().oc2pe().set_bit());
        LEDS.borrow(cs)
            .replace(Some(LedRoulette { leds, timer: tim2 }));
    });

    loop {
        cortex_m::asm::wfi();
    }
}

/// Container to help track LED brightness
///
/// At any given point in the cycle, one LED is fading in. Define this to be the "active" LED.
/// Another LED--the last LED to be active---is fading out.
struct ActiveLed {
    /// index into LED array of the active LED
    index: u8,
    /// duty cycle on a 0-255 scale
    /// TODO: pre-compute steps to avoid floating-point math
    duty_cycle: f32,
}

impl ActiveLed {
    /// Increase duty cycle by a given step. Duty cycle is on a scale of 0-255.
    fn step_duty(&mut self, step: f32) {
        self.duty_cycle += step;
        if self.duty_cycle > 255.0_f32 {
            self.duty_cycle = 255.0;
        }
    }

    /// Get gamma-corrected duty-cycle on a 0-255 scale
    fn duty(&self) -> u32 {
        u32::from(GAMMA[self.duty_cycle.round() as usize])
    }

    /// Get index of active LED
    fn index(&self) -> u8 {
        self.index
    }

    /// Get index of the previously-active LED
    fn prev_led_idx(&self) -> u8 {
        if self.index > 0 {
            self.index - 1
        } else {
            7
        }
    }

    /// Get the gamma-corrected duty-cycle of the previously-active LED
    fn prev_led_duty(&self) -> u32 {
        u32::from(GAMMA[(255.0_f32 - self.duty_cycle).round() as usize])
    }

    /// Rotate which LED is the active LED
    fn rotate_active_led(&mut self) {
        self.index += 1;
        if self.index == 8 {
            self.index = 0;
        }
    }

    /// Set the active LED's duty cycle to 0
    fn reset_duty(&mut self) {
        self.duty_cycle = 0.0;
    }
}

/// Fade LED's on and off sequentially.
///
/// LED brightness is controlled by PWM using TIM2. The output compares, 1 and 2, are set such their
/// interrupts trigger when the "LED on" portion of a duty cycle expires each period for the active
/// LED and previously-active LED, respectively.
#[interrupt]
fn TIM2() {
    static mut ACTIVE_LED: ActiveLed = ActiveLed {
        index: 0,
        duty_cycle: 0.0,
    };
    static mut COUNT: u32 = 0;

    // How long each LED will fade in (and out), in units of timer periods
    const FADE_IN_DURATION: u32 = 375;

    cortex_m::interrupt::free(|cs| {
        let mut state = LEDS.borrow(cs).borrow_mut();
        let state = state.as_mut().unwrap();

        let tim2 = &state.timer;
        let period = tim2.arr.read().arr().bits();

        // Period expired
        if tim2.sr.read().uif().bit_is_set() {
            tim2.sr.modify(|_, w| w.uif().clear());

            *COUNT += 1;
            // Finished fade-on cycle of the active LED. Start fading on the next LED, which will
            // also fade out the currently-active LED.
            if *COUNT == FADE_IN_DURATION {
                ACTIVE_LED.rotate_active_led();
                ACTIVE_LED.reset_duty();
                *COUNT = 0
            }

            // Update duty cycles
            if ACTIVE_LED.duty() != 0 {
                state.leds[ACTIVE_LED.index()].set_high().ok();
            }
            if ACTIVE_LED.prev_led_duty() != 0 {
                state.leds[ACTIVE_LED.prev_led_idx()].set_high().ok();
            }
            tim2.ccr1
                .write(|w| w.ccr().bits(ACTIVE_LED.duty() * period / 255));
            tim2.ccr2
                .write(|w| w.ccr().bits(ACTIVE_LED.prev_led_duty() * period / 255));

            // TODO: calculate this as a constant
            let duty_step: f32 = 255.0 / (FADE_IN_DURATION as f32);
            ACTIVE_LED.step_duty(duty_step);
        }

        // Compare 1 interrupt: duty cycle for active LED
        if tim2.sr.read().cc1if().bit_is_set() {
            tim2.sr.modify(|_, w| w.cc1if().clear());
            state.leds[ACTIVE_LED.index()].set_low().ok();
        }

        // Compare 2 interrupt: duty cycle for previously-active LED
        if tim2.sr.read().cc2if().bit_is_set() {
            tim2.sr.modify(|_, w| w.cc2if().clear());
            state.leds[ACTIVE_LED.prev_led_idx()].set_low().ok();
        }
    });
}

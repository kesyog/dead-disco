use core::ops::{Index, IndexMut};
use hal::gpio::{gpioe::PEx, Output, PushPull};
use stm32f3xx_hal as hal;

/// Re-implementing Leds from the F3 crate so that we can use the official HAL/PAC
pub struct DiscoLeds {
    leds: [hal::gpio::gpioe::PEx<Output<PushPull>>; 8],
}

// LEDS:
//   N: LD3: PE9
//   NW: LD4: PE8
//   W: LD6: PE15
//   SW: LD8: PE14
//   S: LD10: PE13
//   SE: LD9: PE12
//   E: LD7: PE11
//   NE: LD5: PE10
impl DiscoLeds {
    pub fn new(mut gpioe: hal::gpio::gpioe::Parts) -> Self {
        DiscoLeds {
            leds: [
                gpioe
                    .pe9
                    .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
                    .downgrade(),
                gpioe
                    .pe10
                    .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
                    .downgrade(),
                gpioe
                    .pe11
                    .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
                    .downgrade(),
                gpioe
                    .pe12
                    .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
                    .downgrade(),
                gpioe
                    .pe13
                    .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
                    .downgrade(),
                gpioe
                    .pe14
                    .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
                    .downgrade(),
                gpioe
                    .pe15
                    .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
                    .downgrade(),
                gpioe
                    .pe8
                    .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
                    .downgrade(),
            ],
        }
    }
}

impl Index<u8> for DiscoLeds {
    type Output = PEx<Output<PushPull>>;

    fn index(&self, index: u8) -> &Self::Output {
        &self.leds[usize::from(index)]
    }
}

impl IndexMut<u8> for DiscoLeds {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        &mut self.leds[usize::from(index)]
    }
}

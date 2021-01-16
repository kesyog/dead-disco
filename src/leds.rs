use core::ops::{Index, IndexMut};
use hal::gpio::{gpioe::PEx, Output, PushPull};
use stm32f3xx_hal as hal;

/// 8-bit Gamma correction table
/// See https://learn.adafruit.com/led-tricks-gamma-correction/the-quick-fix
pub const GAMMA: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5,
    5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 13, 13, 13, 14,
    14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 24, 24, 25, 25, 26, 27,
    27, 28, 29, 29, 30, 31, 32, 32, 33, 34, 35, 35, 36, 37, 38, 39, 39, 40, 41, 42, 43, 44, 45, 46,
    47, 48, 49, 50, 50, 51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 72,
    73, 74, 75, 77, 78, 79, 81, 82, 83, 85, 86, 87, 89, 90, 92, 93, 95, 96, 98, 99, 101, 102, 104,
    105, 107, 109, 110, 112, 114, 115, 117, 119, 120, 122, 124, 126, 127, 129, 131, 133, 135, 137,
    138, 140, 142, 144, 146, 148, 150, 152, 154, 156, 158, 160, 162, 164, 167, 169, 171, 173, 175,
    177, 180, 182, 184, 186, 189, 191, 193, 196, 198, 200, 203, 205, 208, 210, 213, 215, 218, 220,
    223, 225, 228, 231, 233, 236, 239, 241, 244, 247, 249, 252, 255,
];

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

impl Index<usize> for DiscoLeds {
    type Output = PEx<Output<PushPull>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.leds[index]
    }
}

impl IndexMut<usize> for DiscoLeds {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.leds[index]
    }
}

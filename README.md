# STM32F3 Discovery experiments

Following the [Rust embedded discovery book](https://docs.rust-embedded.org/discovery) with some
others at Fitbit and otherwise poking around now that I have a real STM32F3DISCOVERY board. Mostly
filling in the gaps I couldn't previously get to when doing a similar exercise with a less-featured
dev board ([sandbox-stm32f4-rust](https://github.com/kesyog/sandbox-stm32f4-rust)).

All exercises and experiments are in the examples folder and can be run with:

```sh
openocd -f interface/stlink-v2-1.cfg -f target/stm32f3x.cfg
cargo run --release --example <name_of_example>
```

## Useful links

* [docs folder](./docs): reference manuals from ST
* [stm32f3xx\_hal](https://docs.rs/stm32f3xx-hal/0.6.1/stm32f3xx_hal): stm32-rs HAL crate
* [stm32f3.stm32f303](https://docs.rs/stm32f3/0.12.1/stm32f3/stm32f303/index.html): stm32-rs
peripheral access crate
* [sandbox-stm32f4-rust](https://github.com/kesyog/sandbox-stm32f4-rust): similar examples on a
Nucleo board 
* [discovery](https://github.com/rust-embedded/discovery): the repo for the discovery tutorial

[japaric](https://github.com/japaric), the Rust embedded book author, wrote the support crates
below. They're used in the book, but don't play well with the official stm32-rs crates listed above,
and trying to use them was more trouble than it was worth, at least for the early examples.
* [f3](https://docs.rs/f3/0.6.1/f3/index.html): japaric's board support crate
* [stm32f30x\_hal](https://docs.rs/stm32f30x-hal/0.2.0/stm32f30x_hal/index.html): japaric's HAL
crate
* [stm32f30x](https://docs.rs/stm32f30x/0.8.0/stm32f30x/index.html): japaric's peripheral access
crate

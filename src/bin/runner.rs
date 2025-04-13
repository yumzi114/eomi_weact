#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use eomi_weact as _; 
#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true, 
    dispatchers = [EXTI0, EXTI1]
)]
mod app {
    use cortex_m::delay::Delay;
    use rtt_target::{debug_rprintln, debug_rtt_init_print};
    use stm32h7xx_hal::rcc::PllConfigStrategy;
    use stm32h7xx_hal::spi::{Mode, Phase, Polarity};
    use stm32h7xx_hal::{pwr::PwrExt, rcc::RccExt};
    use stm32h7xx_hal::{prelude::*, spi};
    use embedded_graphics::{
        pixelcolor::{raw::LittleEndian, Rgb565},
        prelude::*,
    };
    #[shared]
    struct Shared {
        
    }
    #[local]
    struct Local {
        led:stm32h7xx_hal::gpio::Pin<'E', 10, stm32h7xx_hal::gpio::Output>
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        debug_rtt_init_print!(); 
// 
        debug_rprintln!("init start"); 
        let dp = ctx.device;
        let mut cp =  ctx.core;

        let pwr = dp.PWR.constrain();
        
        // let pwrcfg = pwr.freeze();
        let vos = pwr.vos0(&dp.SYSCFG);
        let rcc = dp.RCC.constrain();
        let ccdr = rcc.sys_ck(300.MHz())
        .pll1_strategy(PllConfigStrategy::Iterative)
        .pll1_q_ck(550.MHz())
        .freeze(vos.freeze(), &dp.SYSCFG);
        // let mut delay = Delay::new(cp.SYST, ccdr.clocks.sys_ck().to_Hz());
        let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
        

        let mosi = gpioe.pe14.into_alternate::<5>();
        let miso = gpioe.pe13.into_alternate::<5>();
        let sck = gpioe.pe12.into_alternate::<5>();
        
        let cs = gpioe.pe11.into_push_pull_output();   // SPI 칩 선택 (CS)
        let mut led= gpioe.pe10.into_push_pull_output();
        // led.set_high();
        led.set_low();
        // let rst = gpioe.pe9.into_push_pull_output();
        // let spi = dp.SPI4.spi(
        //     pins, 
        //     spi::Config::new(spi::MODE_0)
        //     // Put 1 us idle time between every word sent. (the max is 15 spi peripheral ticks)
        //     .inter_word_delay(0.000001)
        //     // Specify that we use the hardware cs
        //     .hardware_cs(spi::HardwareCS {
        //         // See the docs of the HardwareCSMode to see what the different modes do
        //         mode: spi::HardwareCSMode::EndlessTransaction,
        //         // Put 1 us between the CS being asserted and the first clock
        //         assertion_delay: 0.000001,
        //         // Our CS should be high when not active and low when asserted
        //         polarity: spi::Polarity::IdleHigh,
        //     }), 
        //     3.MHz(), 
        //     ccdr.peripheral.SPI4,
        //     &ccdr.clocks,);
        let spi: spi::Spi<_, _, u8> = dp.SPI4.spi(
            (sck, miso, mosi),
            Mode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            1.MHz(),
            ccdr.peripheral.SPI4,
            &ccdr.clocks,
        );
        let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs);
        
        
        // let ccdr = rcc
        //     .sys_ck(400.MHz()).pll2_p_ck(400.MHz() / 5)
        //     .pll2_q_ck(400.MHz() / 2)
        //     .pll2_r_ck(400.MHz() / 2)
        //     // LTDC
        //     .pll3_p_ck(800.MHz() / 2)
        //     .pll3_q_ck(800.MHz() / 2)
        //     .pll3_r_ck(800.MHz() / 83)
        //     .freeze(pwrcfg, &dp.SYSCFG);
        // let pll3_r = ccdr.clocks.pll3_r_ck().expect("pll3 must run!");
        // let mut delay = cp.SYST.delay(ccdr.clocks);
        // // let ccdr = rcc.sys_ck(100.MHz()).freeze(pwrcfg, &dp.SYSCFG);
        // let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
        // cp.SCB.invalidate_icache();
        // cp.SCB.enable_icache();
        // //cp.SCB.enable_dcache(&mut cp.CPUID); // TODO invalidate dcache when writing to the display
        // cp.DWT.enable_cycle_counter();
        task1::spawn().ok();
        
        (
            Shared {
            },
            Local {
                led
            },
        )
    }
    #[idle(local = [led])]
    fn idle(cx:idle::Context,) -> ! {

        loop {
            // cx.local.led.set_high();
            // debug_rprintln!("TDLE"); 
            cortex_m::asm::wfi();
        }
    }

    #[task(priority = 1)]
    async fn task1(_cx: task1::Context) {
        debug_rprintln!("TASK!"); 
    }
    
}

#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use eomi_weact as _; // global logger + panicking-behavior + memory layout

// TODO(7) Configure the `rtic::app` macro
#[rtic::app(
    // TODO: Replace `some_hal::pac` with the path to the PAC
    device = stm32h7xx_hal::pac,
    peripherals = true, 
    dispatchers = [EXTI0, EXTI1]
)]
mod app {
    use rtt_target::{debug_rprintln, debug_rtt_init_print};

    // Shared resources go here
    #[shared]
    struct Shared {
        // TODO: Add resources
    }

    // Local resources go here
    #[local]
    struct Local {
        // TODO: Add resources
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        debug_rtt_init_print!(); 
        debug_rprintln!("init start"); 

        // TODO setup monotonic if used
        // let sysclk = { /* clock setup + returning sysclk as an u32 */ };
        // let token = rtic_monotonics::create_systick_token!();
        // rtic_monotonics::systick::Systick::new(cx.core.SYST, sysclk, token);


        task1::spawn().ok();

        (
            Shared {
                // Initialization of shared resources go here
            },
            Local {
                // Initialization of local resources go here
            },
        )
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {

        loop {
            debug_rprintln!("IDLE"); 
            continue;
        }
    }

    // TODO: Add tasks
    #[task(priority = 1)]
    async fn task1(_cx: task1::Context) {
        debug_rprintln!("TASK!"); 
        // loop{
            
        // }
    }
}

#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

#[link_section = ".axisram.buffers"]
static mut BUFFER: MaybeUninit<[u8; BUFFER_SIZE]> = MaybeUninit::uninit();


use core::mem::MaybeUninit;
const BUFFER_SIZE: usize = 100;

use eomi_weact as _; 
#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true, 
    dispatchers = [EXTI0, EXTI1]
)]

mod app {
    
    use core::mem::MaybeUninit;
    // use super::*;
    use cortex_m::delay::Delay;
    
    use display_interface_spi::SPIInterface;
    use embedded_graphics::mono_font::ascii::FONT_7X14_BOLD;
    use embedded_graphics::mono_font::iso_8859_1::FONT_9X18_BOLD;
    use embedded_graphics::mono_font::iso_8859_15::FONT_10X20;
    use embedded_graphics::pixelcolor::{Rgb565, Rgb666};
    use embedded_graphics::prelude::{Point, RgbColor};
    use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle};
    use embedded_graphics::{
        mono_font::MonoTextStyle,
        prelude::*,
        text::Text,
    };
    use embedded_hal_bus::spi::ExclusiveDevice;
    use ili9341::Ili9341;
    use mipidsi::models::ILI9486Rgb666;
    use mipidsi::{Builder, Display};
    use mipidsi::Orientation as Orientationa;
    // use embedded_hal_bus::spi::ExclusiveDevice;
    use rtt_target::{debug_rprintln, debug_rtt_init_print};
    use st7735_lcd::Orientation;
    use stm32h7xx_hal::dma::dma::{DmaConfig, StreamsTuple};
    use stm32h7xx_hal::dma::{MemoryToPeripheral, Transfer};
    use stm32h7xx_hal::gpio::{Edge, ExtiPin, Input, Output, Pin, Speed};
    use stm32h7xx_hal::pac::SPI1;
    use stm32h7xx_hal::rcc::PllConfigStrategy;
    use stm32h7xx_hal::spi::{Enabled, Mode, Phase, Polarity, Spi};
    use stm32h7xx_hal::timer::Timer;
    use stm32h7xx_hal::{pwr::PwrExt, rcc::RccExt};
    use stm32h7xx_hal::{prelude::*, spi};
    use embedded_graphics::draw_target::DrawTarget;
    // use mipidsi::models::ILI9486Rgb565;
    use profont::PROFONT_24_POINT;


    

    #[shared]
    struct Shared {
        delay:Delay,
        menu_num:usize
        // menu_list:[&'a str; 2]
    }
    #[local]
    struct Local {
        led:Pin<'E', 10, Output>,
        k1_bt:Pin<'C', 13, Input>,
        blue_led:Pin<'E', 3, Output>,
        up_bt:Pin<'A', 10, Input>,
        down_bt:Pin<'A', 11, Input>,
        sel_bt:Pin<'A', 12, Input>,
        display: Display<SPIInterface<Spi<SPI1, Enabled>, Pin<'A', 2, Output>, Pin<'A', 4, Output>>, ILI9486Rgb666, Pin<'A', 1, Output>>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        debug_rtt_init_print!(); 
// 
        debug_rprintln!("init start"); 
        let menu_num=1_usize;
        // let menu_list = ["FIRSTE MENU", "TWO MENU"];
        let mut dp = ctx.device;
        // !!!디버그 핀 GPIO 돌릴때
        // dp.DBGMCU.cr.modify(|_, w| w.dbgsleep_d1().clear_bit());
        let cp =  ctx.core;
        
        let pwr = dp.PWR.constrain();
        
        // let pwrcfg = pwr.freeze();
        let vos = pwr.vos0(&dp.SYSCFG).freeze();
        let rcc = dp.RCC.constrain();
        let ccdr = rcc.sys_ck(500.MHz())
        .pll1_strategy(PllConfigStrategy::Iterative)
        .pll1_q_ck(300.MHz())
        .freeze(vos, &dp.SYSCFG);

        let real_sys_ck = ccdr.clocks.sys_ck();
        debug_rprintln!("init start{:?}",real_sys_ck); 
        let mut delay: Delay = Delay::new(cp.SYST, ccdr.clocks.sys_ck().to_Hz());
        let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
        let mut k1_bt = gpioc.pc13.into_pull_down_input();
        k1_bt.make_interrupt_source(&mut dp.SYSCFG);
        k1_bt.trigger_on_edge(&mut dp.EXTI, Edge::Rising);
        k1_bt.enable_interrupt(&mut dp.EXTI);

        let mut up_bt = gpioa.pa10.into_pull_down_input();
        up_bt.make_interrupt_source(&mut dp.SYSCFG);
        up_bt.trigger_on_edge(&mut dp.EXTI, Edge::Rising);
        up_bt.enable_interrupt(&mut dp.EXTI);

        let mut down_bt = gpioa.pa11.into_pull_down_input();
        down_bt.make_interrupt_source(&mut dp.SYSCFG);
        down_bt.trigger_on_edge(&mut dp.EXTI, Edge::Rising);
        down_bt.enable_interrupt(&mut dp.EXTI);
        let mut sel_bt = gpioa.pa12.into_pull_down_input();
        sel_bt.make_interrupt_source(&mut dp.SYSCFG);
        sel_bt.trigger_on_edge(&mut dp.EXTI, Edge::Rising);
        sel_bt.enable_interrupt(&mut dp.EXTI);

        //board LCD pins
        let mosi = gpioe.pe14.into_alternate::<5>();
        let dc  = gpioe.pe13.into_push_pull_output();
        let sck = gpioe.pe12.into_alternate::<5>();
        let dummy_miso = stm32h7xx_hal::spi::NoMiso; 

        //ILI9488 pins
        let mut back_l= gpioa.pa3.into_push_pull_output();
        let s_cs = gpioa.pa4.into_push_pull_output();
        let s_dc  = gpioa.pa2.into_push_pull_output();
        let s_mosi = gpioa.pa7.into_alternate::<5>();
        let s_miso = gpioa.pa6.into_alternate::<5>();
        let s_sck = gpioa.pa5.into_alternate::<5>();
        let s_rst = gpioa.pa1.into_push_pull_output();
        let s_spi: spi::Spi<_, _, u8> = dp.SPI1.spi(
            (s_sck, s_miso, s_mosi),
            Mode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            20.MHz(),
            ccdr.peripheral.SPI1,
            &ccdr.clocks,
        );
        back_l.set_high();
        
        let spi_iface = SPIInterface::new(s_spi, s_dc,s_cs);
        let mut display = Builder::ili9486_rgb666(spi_iface)
        .init(&mut delay, Some(s_rst)).unwrap();
        // let s_style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb666::BLACK);
        display.clear(Rgb666::BLACK).unwrap();
        display.set_orientation(Orientationa::Landscape(false)).unwrap();
        display.set_orientation(Orientationa::Portrait(true)).unwrap();
        
        let mut led= gpioe.pe10.into_push_pull_output();
        led.set_low();
        let blue_led: Pin<'E', 3, stm32h7xx_hal::gpio::Output>= gpioe.pe3.into_push_pull_output();
        
        let rst = gpioe.pe9.into_push_pull_output();
        //SPI CONFIG 
        let spi: spi::Spi<_, _, u8> = dp.SPI4.spi(
            (sck, dummy_miso, mosi),
            Mode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            8.MHz(),
            ccdr.peripheral.SPI4,
            &ccdr.clocks,
        );
        let mut disp = st7735_lcd::ST7735::new(spi, dc, rst, true, false, 160, 80);
        disp.init(&mut delay).unwrap();
        
        disp.set_orientation(&Orientation::LandscapeSwapped).unwrap();
        disp.set_offset(1, 29);
        disp.clear(RgbColor::WHITE).unwrap();
        let style = MonoTextStyle::new(&FONT_7X14_BOLD, RgbColor::BLACK);

    // 텍스트 출력
        disp.set_offset(0, 30);
        Text::new("Hello TFT!", Point::new(6, 10), style)
        .draw(&mut disp)
        .unwrap();
        Text::new("TTTTTT", Point::new(6, 30), style)
            .draw(&mut disp)
            .unwrap();

        debug_rprintln!("init Done"); 
        (
            Shared {
                delay,
                menu_num
                
            },
            Local {
                led,
                k1_bt,
                blue_led,
                up_bt,
                down_bt,
                sel_bt,
                display
            },
        )
    }
    #[idle(local=[display],shared = [delay,menu_num])]
    // #[idle]
    fn idle(mut cx:idle::Context,) -> ! {
        let s_style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb666::WHITE);
        let sel_style = MonoTextStyle::new(&PROFONT_24_POINT, Rgb666::BLUE);
        let r_style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb666::BLACK) 
        .build();
        let menu_list = ["1 MENU", "2 MENU", "3 MENU"];
        let mut flag= 0_usize;
        loop {
      
            cx.shared.menu_num.lock(| num|{
                if *num !=flag{
                    Rectangle::new(Point::new(1, 10), Size::new(30, 130))
                        .into_styled(r_style)
                        .draw(cx.local.display)
                        .unwrap();
                    Rectangle::new(Point::new(20, 180), Size::new(200, 50))
                        .into_styled(r_style)
                        .draw(cx.local.display)
                        .unwrap();
                    Text::new("->", Point::new(1, (*num as i32 * 30)+15), sel_style)
                        .draw(cx.local.display)
                        .unwrap();
                    flag=*num;
                }
                
            });
            
            menu_list.iter().enumerate().for_each(|(len, str)| {
                let posion = len + 1;
                if flag==posion{
                    let description=match flag {
                        1=>"SOMEEEEE",
                        2=>"TWOOOOOOOO",
                        3=>"THREEEE",
                        _=>"UNKOWN"
                    };
                    Text::new(str, Point::new(40, (posion as i32 * 30)+15), sel_style)
                        .draw(cx.local.display)
                        .unwrap();
                    Line::new(Point::new(1, 150), Point::new(310, 150))
                        .into_styled(PrimitiveStyle::with_stroke(Rgb666::WHITE, 1))
                        .draw(cx.local.display).unwrap();
                    Text::new(description, Point::new(20, 200), s_style)
                        .draw(cx.local.display)
                        .unwrap();
                }else {
                    Text::new(str, Point::new(40, (posion as i32 * 30)+15), s_style)
                        .draw(cx.local.display)
                        .unwrap();
                }
                
            });
            
            // cx.local.display;
            // cortex_m::asm::wfi();
        }
    }

    #[task(binds = EXTI15_10, 
        local=[
            k1_bt,
            blue_led,
            up_bt,
            down_bt,
            sel_bt
        ],
        shared = [delay,menu_num], )]
    fn button_click(mut ctx: button_click::Context) {
        if ctx.local.k1_bt.check_interrupt() {
            ctx.local.k1_bt.clear_interrupt_pending_bit();
            ctx.local.blue_led.set_high();
            
            ctx.shared.delay.lock(|tim|tim.delay_ms(200));
            ctx.local.blue_led.set_low();
        }
        if ctx.local.up_bt.check_interrupt() {
            ctx.local.up_bt.clear_interrupt_pending_bit();
            ctx.local.blue_led.set_high();
            ctx.shared.menu_num.lock(| num|{
                if *num<3{
                    *num+=1_usize;
                }else{
                    *num=1_usize;
                }
            });
            ctx.shared.delay.lock(|tim|tim.delay_ms(200));
            ctx.local.blue_led.set_low();
        }
        if ctx.local.down_bt.check_interrupt() {
            ctx.local.down_bt.clear_interrupt_pending_bit();
            ctx.local.blue_led.set_high();
            ctx.shared.menu_num.lock(| num|{
                if *num>1{
                    *num-=1_usize;
                }else{
                    *num=3_usize;
                }
            });
            ctx.shared.delay.lock(|tim|tim.delay_ms(200));
            ctx.local.blue_led.set_low();
        }
        if ctx.local.sel_bt.check_interrupt() {
            ctx.local.sel_bt.clear_interrupt_pending_bit();
            ctx.local.blue_led.set_high();
            ctx.shared.delay.lock(|tim|tim.delay_ms(200));
            ctx.local.blue_led.set_low();
        }
    }
    
}

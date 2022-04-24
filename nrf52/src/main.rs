// stripped down from pellegrino/firmware/kernel/src/main.rs

#![no_main]
#![no_std]

//#[panic_handler]
//fn panic(_info: &PanicInfo) -> ! {
//    loop {
//        atomic::compiler_fence(Ordering::SeqCst);
//    }
//}

use defmt_rtt as _;
use panic_probe as _;

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
//#[defmt::panic_handler]
//fn panic() -> ! {
//    cortex_m::asm::udf()
//}

#[rtic::app(device = nrf52840_hal::pac, dispatchers = [SWI0_EGU0])]
mod app {
    // use core::sync::atomic::Ordering;
    use cortex_m::singleton;
    use defmt::unwrap;
    use systick_monotonic::*;

    //use groundhog::RollingTimer;
    //use groundhog_nrf52::GlobalRollingTimer;
    use nrf52840_hal::{
        clocks::{ExternalOscillator, Internal, LfOscStopped},
        // pac::TIMER0,
        pac::USBD,
        usbd::{UsbPeripheral, Usbd},
        Clocks,
    };
    use usb_device::{
        class_prelude::UsbBusAllocator,
        device::{UsbDeviceBuilder, UsbVidPid},
    };
    use usbd_serial::{SerialPort, USB_CLASS_CDC};

    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<100>; // 100 Hz / 10 ms granularity

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    fn enable_usb_interrupts(usbd: &USBD) {
        usbd.intenset.write(|w| {
            // rg -o "events_[a-z_0-9]+" ./usbd.rs | sort | uniq
            w.endepin0().set_bit();
            w.endepin1().set_bit();
            w.endepin2().set_bit();
            w.endepin3().set_bit();
            w.endepin4().set_bit();
            w.endepin5().set_bit();
            w.endepin6().set_bit();
            w.endepin7().set_bit();

            w.endepout0().set_bit();
            w.endepout1().set_bit();
            w.endepout2().set_bit();
            w.endepout3().set_bit();
            w.endepout4().set_bit();
            w.endepout5().set_bit();
            w.endepout6().set_bit();
            w.endepout7().set_bit();

            w.ep0datadone().set_bit();
            w.ep0setup().set_bit();
            w.sof().set_bit();
            w.usbevent().set_bit();
            w.usbreset().set_bit();
            w
        });
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let device = cx.device;

        // Setup clocks early in the process. We need this for USB later
        let clocks = Clocks::new(device.CLOCK);
        let clocks = clocks.enable_ext_hfosc();
        let clocks =
            unwrap!(singleton!(: Clocks<ExternalOscillator, Internal, LfOscStopped> = clocks));

        // Before we give away the USB peripheral, enable the relevant interrupts
        enable_usb_interrupts(&device.USBD);

        let (_usb_dev, _usb_serial) = {
            let usb_bus = Usbd::new(UsbPeripheral::new(device.USBD, clocks));
            let usb_bus =
                defmt::unwrap!(singleton!(:UsbBusAllocator<Usbd<UsbPeripheral>> = usb_bus));

            let usb_serial = SerialPort::new(usb_bus);
            let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
                .manufacturer("OVAR Labs")
                .product("Anachro Pellegrino")
                // TODO: Use some kind of unique ID. This will probably require another singleton,
                // as the storage must be static. Probably heapless::String -> singleton!()
                .serial_number("ajm001")
                .device_class(USB_CLASS_CDC)
                .max_packet_size_0(64) // (makes control transfers 8x faster)
                .build();

            (usb_dev, usb_serial)
        };
        let systick = cx.core.SYST;
        let mono = Systick::new(systick, 64_000_000);

        (Shared {}, Local {}, init::Monotonics(mono))
    }

    #[task]
    fn broebel(_: broebel::Context) {
        defmt::println!("broebel\n");
        // broebel::spawn_at(monotonics::now() + 1.secs()).unwrap();
        broebel::spawn_after(1.secs()).unwrap();
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        defmt::println!("idle\n");
        broebel::spawn_after(1.secs()).unwrap();
        loop {
            rtic::export::wfi()
        }
    }
}

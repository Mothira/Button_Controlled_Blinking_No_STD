#![no_std]
#![no_main]

use core::cell::{Cell, RefCell};
use critical_section::Mutex;
use esp_backtrace as _;
use esp_println::println;
use esp_hal::
{
    delay::Delay,
    prelude::*, 
    gpio::{Event, Input, Pull, Io, Level, Output}
};
// Create a Global Variable for a GPIO Peripheral to pass around between threads
static G_PIN: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));
// Create a GLobal Variable for a FLAG to pass around between threads
// Option is not used since the variable is being directly initialized
// Using Cell instead of RefCell because it's a bool so we can take a copy instead of a reference
static G_FLAG: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

#[handler]
fn gpio()
{
    // Start critical section
    critical_section::with(|cs| 
    {  
        // Obtain access to Global Input Peripheral
        // Clear Interrupt Pending Flag
        G_PIN.borrow_ref_mut(cs).as_mut().unwrap().clear_interrupt();

        // Assert G_FLAG indicating a press button happened
        G_FLAG.borrow(cs).set(true);
    });
}

#[entry]
fn main() -> ! 
{
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let delay = Delay::new();

    // Instantiate and Create Handle for IO
    let mut io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Interrupt Configuration
    io.set_interrupt_handler(gpio);
    let mut button = Input::new(io.pins.gpio0, Pull::Up);
    button.listen(Event::FallingEdge);
    critical_section::with(|cs| {  G_PIN.borrow_ref_mut(cs).replace(button) });

    // Create Output Pin
    let mut led_pin = Output::new(io.pins.gpio4, Level::High);

    // Initialize a delay
    let mut blinkdelay = 400_u32;

    // Start by setting the led to low
    led_pin.set_low();

    loop 
    {
        // This is where the button press is repeatedly checked
        critical_section::with(|cs| {
            if G_FLAG.borrow(cs).get()
            {
                // Clear Global flag
                G_FLAG.borrow(cs).set(false);
                // Change value of blink delay based on button press
                {
                    if blinkdelay <= 50_u32
                    {
                        blinkdelay = 400_u32;
                    }
                    else
                    {
                        blinkdelay -= 50_u32;
                    }
                }
                println!("Button pressed with a delay of: {}", blinkdelay);
            }
        });
        delay.delay_millis(blinkdelay);
        led_pin.set_high();
        delay.delay_millis(blinkdelay);
        led_pin.set_low();
    }
}

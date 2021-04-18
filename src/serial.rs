use core::time::Duration;

const BAUD:u32 = 230_400;
const PORT:&str = "/dev/serial/by-id/usb-FTDI_FT232R_USB_UART_00000000-if00-port0";

fn work() -> serialport::Result<()>
{
    let mut port = serialport::new(PORT, BAUD).open()?;
    port.set_timeout(Duration::from_millis(1000))?;
    loop {
	let mut buffer: [u8; 1024] = [0; 1024];
	match port.read(&mut buffer)
	{
	    Ok(bytes_read) => {
		if let Ok(s) = std::str::from_utf8(&buffer[0..bytes_read])
		{
		    print!("{}", s);
		}
	    }
	    Err(error) => {
		println!("error: {:?}", error);
	    }
	}
    }
}

fn main() {
    println!("Hello, world! This is Rusty Peanut!");
    work().expect("Work failed!");
}

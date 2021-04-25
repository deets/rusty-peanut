use std::thread;
use std::time::Duration;
use crossbeam::channel::{Receiver, unbounded};

pub struct SerialConnector
{
    pub receiver: Receiver<String>
}

impl SerialConnector
{
    pub fn new(port: &str, baud: u32) -> Result<SerialConnector, serialport::Error>
    {
	let mut port = serialport::new(port, baud).open()?;
	port.set_timeout(Duration::from_millis(1000))?;

	let (s, r) = unbounded();
	thread::spawn(move || {
	    loop {
		let mut buffer: [u8; 1024] = [0; 1024];
		match port.read(&mut buffer)
		{
		    Ok(bytes_read) => {
			if let Ok(debug_line) = std::str::from_utf8(&buffer[0..bytes_read])
			{
			    s.send(debug_line.to_string()).expect("serial crossbeam channel failed");
			}
		    }
		    Err(error) => {
			println!("error: {:?}", error);
		    }
		}
	    }
	});
	Ok(SerialConnector{receiver: r})
    }
}

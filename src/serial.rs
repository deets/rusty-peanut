use std::thread;
use std::time::Duration;
use crossbeam::channel::{Receiver, unbounded};

pub struct SerialConnector
{
    pub receiver: Receiver<String>
}

struct LineProtocol
{
    bytes: Vec<u8>
}

impl LineProtocol
{
    fn new() -> LineProtocol
    {
	LineProtocol{ bytes: vec![] }
    }

    fn feed<F>(&mut self, buffer: &[u8], mut func: F) where F: FnMut(&str)
    {
	for c in buffer {
	    self.bytes.push(*c);
	    let l = self.bytes.len();
	    let ends_with_crlf = unsafe {
		l >= 2 && *self.bytes.get_unchecked(l - 2) == 13 as u8 && *self.bytes.get_unchecked(l - 1) == 10 as u8
	    };
	    if ends_with_crlf {
		if let Ok(s) = std::str::from_utf8(&self.bytes[0..self.bytes.len() - 2])
		{
		    func(s);
		}
		self.bytes.clear();
	    }
	}
    }
}


impl SerialConnector
{
    pub fn new(port: &str, baud: u32) -> Result<SerialConnector, serialport::Error>
    {
	let mut port = serialport::new(port, baud).open()?;
	port.set_timeout(Duration::from_millis(1000))?;
	let mut lp = LineProtocol::new();
	let (s, r) = unbounded();
	thread::spawn(move || {
	    loop {
		let mut buffer: [u8; 1024] = [0; 1024];
		match port.read(&mut buffer)
		{
		    Ok(bytes_read) => {
			lp.feed(&buffer[0..bytes_read], |line: &str| {
			    s.send(line.to_string()).expect("serial crossbeam channel failed");
			});
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_env_log::test;

    #[test]
    fn feed_bytes_but_no_crlf() {
	let mut lp = LineProtocol::new();
	let mut called = false;
	lp.feed(b"Hallo", |_x: &str| { called = true; });
	assert!(called == false);
    }

    #[test]
    fn feed_bytes_with_crlf() {
	let mut lp = LineProtocol::new();
	let mut line:String = "".to_string();
	lp.feed(b"Hallo\r\n", |x: &str| { line = x.to_string() });
	assert!(line == "Hallo");
    }

}

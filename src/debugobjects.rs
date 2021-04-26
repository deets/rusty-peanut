use nannou::prelude::*;
use std::collections::hash_map::HashMap;
use std::vec::Vec;
use std::collections::VecDeque;
use log::{debug, warn};
use thiserror::Error;

type Rect = nannou::geom::rect::Rect;
type Color = Rgb<u8>;


#[derive(Error, Debug)]
pub enum DebugObjectError
{
    #[error("No name given for the DebugObject")]
    NoNameGiven,
    #[error("Invalid format {0}")]
    InvalidFormat(String),
    #[error("Unknown error")]
    Unknown,
    #[error("IndexError")]
    IndexError,
    #[error("ParseNumberError")]
    ParseNumberError,
}

impl From<std::num::ParseFloatError> for DebugObjectError {
    fn from(_error: std::num::ParseFloatError) -> Self {
	DebugObjectError::ParseNumberError
    }
}

impl From<std::num::ParseIntError> for DebugObjectError {
    fn from(_error: std::num::ParseIntError) -> Self {
	DebugObjectError::ParseNumberError
    }
}


fn strip_single_quotes(input: &str) -> &str
{
    let v: Vec<&str> = input.split("'").collect();
    unsafe {
	v.get_unchecked(v.len() / 2)
    }
}

pub struct DebugLine
{
    pub keyword: String,
    pub tokens: Vec<String>
}

impl DebugLine
{
    pub fn from_str(line: &str) -> std::result::Result<DebugLine, DebugObjectError>
    {
	let tokens:Vec<String> = line.split_whitespace().map(|s| { s.to_string() }).filter(|part| { part.len() > 0 }).collect();
	if tokens.len() > 0{
	    let mut keyword = tokens[0].clone();
	    if keyword.starts_with("`") {
		keyword = keyword[1..].to_string();
		return Ok(DebugLine{keyword: keyword, tokens: tokens[1..].to_vec()});
	    }
	}
	Err(DebugObjectError::InvalidFormat(line.to_string()))
    }
}

pub trait DebugProcessor
{
    fn name(&self) -> String;
    fn draw(&self, draw: &nannou::draw::Draw);
    fn feed(&mut self, tokens: Vec<String>);
}

struct ScopeConfig
{
    name: String,
    pos: Point2,
    size: Point2,
    samples: usize,
    rate: usize,
    color: Color,
}

impl ScopeConfig
{
    fn from_tokens(tokens: &Vec<String>) -> Result<ScopeConfig, DebugObjectError>
    {
	let name = tokens.get(0).ok_or(DebugObjectError::NoNameGiven)?;
	let pos = pt2(0.0, 0.0);
	let mut size = pt2(255.0, 256.0);
	let mut samples: usize = 256;
	let rate: usize = 1;
	let color = BLACK;
	let mut index: usize = 1;
	while index < tokens.len() {
	    let command = tokens.get(index).ok_or(DebugObjectError::IndexError)?;
	    debug!("ScopeConfig: attempting to decode {} at index {}", &command, index);
	    if command == "SIZE" {
		let width = tokens.get(index + 1).ok_or(DebugObjectError::IndexError)?.parse::<f32>()?;
		let height = tokens.get(index + 2).ok_or(DebugObjectError::IndexError)?.parse::<f32>()?;
		size = pt2(width, height);
		debug!("decoded SIZE: {:?}", size);
		index += 3
	    } else if command == "SAMPLES" {
		samples = tokens.get(index + 1).ok_or(DebugObjectError::IndexError)?.parse::<usize>()?;
                index += 2;
	    } else {
		warn!("Not implemented");
		break;
	    }
	}
	Ok(ScopeConfig{ name: strip_single_quotes(name).to_string(), pos, size, samples, rate, color })
    }
}

struct ScopeSignal
{
    name: String,
    min: f32,
    max: f32,
    y_size: Option<f32>,
    y_base: Option<f32>,
    //{legend
    color: Color,
    pub values: VecDeque<f32>,
}

pub struct Scope
{

    name: String,
    samples: usize, // Number of retained samples
    rect: Rect,

    signals: Vec<ScopeSignal>
}

impl Scope {

    pub fn new(tokens: &Vec<String>) -> Scope
    {
	assert!(tokens.len() >= 1);

	let mut values = VecDeque::new();
	values.push_back(0.0);
	values.push_back(0.0);

	let size: usize = 256;
	let height = 256.0;
	let width = 255.0;

	let mut res = Scope{
	    name: tokens[0].clone(),
	    samples: size,
	    rect: Rect::from_x_y_w_h(0.0, 0.0, width, height),
	    signals: vec![],
	};
	res.signals.push(
	    ScopeSignal
	    {
		name: "Test".to_string(),
		min: -100.0,
		max: 100.0,
		y_size: None,
		y_base: None,
		color: GREEN,
		values: VecDeque::from(vec![0.0, 0.0])
	    }
	);
	res
    }

    pub fn feed_floats(&mut self, values: Vec<f32>)
    {
	if values.len() != self.signals.len() {
	    warn!("Scope<{}>::feed values and signals length differ", self.name);
	}
	let samples = self.samples;
	self.signals.iter_mut().zip(values)
	    .for_each(|(signal, value)| {
		signal.values.push_back(value.clamp(signal.min, signal.max));
		while signal.values.len() >= samples {
		    signal.values.pop_front();
		}
	    });
    }

    fn setup_signal(&mut self, tokens: &Vec<String>) -> Result<(), String>
    {
	Err("not implemented".to_string())
    }
}

impl DebugProcessor for Scope {

    fn name(&self) -> String {
	self.name.clone()
    }

    fn draw(&self, draw: &nannou::draw::Draw)
    {
	let xy = self.rect.xy();
	let wh = self.rect.wh();
	let step = wh.x / (self.samples as f32 - 1.0);
	draw.rect().xy(xy).wh(wh).color(YELLOW);
	self.signals.iter().for_each(|signal| {
	    // Create an iterator yielding triangles for drawing a sine wave.
	    let tris = signal.values.iter().zip(signal.values.iter().skip(1)).enumerate()
		.flat_map(|(i, (left, right))| {
		    let l_x = step * i as f32;
		    let r_x = step * (i + 1) as f32;
		    let a = pt2(l_x, map_range(*left, signal.min, signal.max, 0.0, wh.y));
		    let b = pt2(r_x, map_range(*right, signal.min, signal.max, 0.0, wh.y));
		    let c = pt2(r_x, map_range(0.0, signal.min, signal.max, 0.0, wh.y));
		    let d = pt2(l_x, map_range(0.0, signal.min, signal.max, 0.0, wh.y));
		    geom::Quad([a, b, c, d]).triangles_iter()
		})
		.map(|tri| {
		    // Color the vertices based on their amplitude.
		    tri.map_vertices(|v| {
			let y_fract = map_range(v.y.abs(), 0.0, wh.y, 0.0, 1.0);
			(v, signal.color)
		    })
		});
	    // Draw the mesh!
	    draw.xy(-wh / 2.0).mesh().tris_colored(tris);
	});
    }

    fn feed(&mut self, tokens: Vec<String>)
    {
	let mut err = Ok(());
	let mut floats = vec![];
	for token in &tokens {
	    match token.parse::<f32>() {
		Ok(number) => {
		    floats.push(number);
		}
		Err(e) => {
		    err = Err(e);
		    break;
		}
	    }
	}
	match err {
	    Ok(_) => {
		self.feed_floats(floats);
	    }
	    _ => {
		if self.setup_signal(&tokens).is_err() {
		    warn!("couldn't setup signal with {:?}", &tokens);
		}
	    }
	}
    }
}

pub enum DebugObject
{
    Scope(Scope)
}

impl DebugProcessor for DebugObject
{
    fn name(&self) -> std::string::String {
	match self {
	    DebugObject::Scope(scope) => scope.name()
	}
    }

    fn draw(&self, draw: &nannou::draw::Draw)
    {
	match self {
	    DebugObject::Scope(scope) => { scope.draw(draw); }
	}
    }

    fn feed(&mut self, tokens: Vec<String>)
    {
	match self {
	    DebugObject::Scope(scope) => { scope.feed(tokens); }
	}
    }
}

pub struct DebugObjects
{
    objects: HashMap<String, DebugObject>
}

impl DebugObjects
{
    pub fn new() -> DebugObjects
    {
	DebugObjects{objects: HashMap::new()}
    }
}

impl DebugObjects
{
    pub fn feed(&mut self, line: &str)
    {
	let line = DebugLine::from_str(line).expect("error parsing line");
	match self.objects.get_mut(&line.keyword) {
	    Some(debug_object) => {
		debug!("found DebugObject `{}, feeding to it", debug_object.name());
		debug_object.feed(line.tokens);
	    }
	    None => {
		debug!("no DebugObject for keyword  {} - trying to create one", line.keyword);
		match self.create(&line.keyword, &line.tokens)
		{
		    Some(new_object) => {
			self.objects.insert(new_object.name(), new_object);
		    },
		    _ => { warn!("No factory found for {}", line.keyword); }
		}
	    }
	}
    }

    fn create(&self, keyword: &str, tokens: &Vec<String>) -> Option<DebugObject>
    {
	// We need at least one additional token afetr the
	// name, which will become the identifier.
	if tokens.len() >= 1 {
	    if keyword == "SCOPE" {
		debug!("created Scope object named {}", tokens[0]);
		return Some(DebugObject::Scope(Scope::new(tokens)))
	    }
	}
	None
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use test_env_log::test;

    const SCOPE_DECLARATION:&str = "`SCOPE MyScope SIZE 254 84 SAMPLES 128\r\n\
`MyScope 'Sawtooth' 0 63 64 10 %1111\r\n\
`MyScope 31\r\n\
`MyScope 32\r\n\
`MyScope 33\r\n\
`MyScope 34\r\n\
`MyScope 35\r\n\
`MyScope 36\r\n\
";

    #[test]
    fn instantiate_scope_through_debug_objects() {
	let mut debug_objects = DebugObjects::new();
	debug_objects.feed("`SCOPE MyScope SIZE 254 84 SAMPLES 128");
	debug_objects.feed("`MyScope 'Sawtooth' 0 63 64 10 %1111");
    }

    #[test]
    fn instantiate_scope_configure_and_feed() {
	let mut debug_objects = DebugObjects::new();
	debug_objects.feed("`SCOPE MyScope SIZE 254 84 SAMPLES 128");
	for line in String::from(SCOPE_DECLARATION).split_terminator("\r\n")
	{
	    debug_objects.feed(line);
	}
    }

    fn to_tokens(tokens: &[&str]) -> Vec<String>
    {
	tokens.iter().map(|s| { s.to_string() }).collect()
    }

    #[test]
    fn test_configuration_commandline() {
	let tokens = to_tokens(&["MyScope", "SIZE", "254", "84", "SAMPLES", "128"]);
	let scope_config = ScopeConfig::from_tokens(&tokens).expect("invalid configuration");
	assert_eq!(scope_config.name, "MyScope");
	assert_eq!(scope_config.size, pt2(254.0, 84.0));
	assert_eq!(scope_config.samples, 128);
    }
}

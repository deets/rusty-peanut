use nannou::prelude::*;
use std::collections::hash_map::HashMap;
use std::vec::Vec;
use std::collections::VecDeque;
use log::{debug, warn};
use thiserror::Error;
use phf::phf_map;

type Rect = nannou::geom::rect::Rect;
type Color = Rgb<u8>;
type Point2 = nannou::geom::Point2<f32>;

static COLOR_MAP: phf::Map<&'static str, Color> = phf_map! {
    "BLACK" => BLACK,
    "WHITE" => WHITE,
    "ORANGE" => ORANGE,
    "BLUE" => BLUE,
    "GREEN" => GREEN,
    "CYAN" => CYAN,
    "RED" => RED,
    "MAGENTA" => MAGENTA,
    "YELLOW" => YELLOW,
};

struct Style
{
    font_size: u32,
    // the offset from the topleft corner we
    // draw the signal name to
    signal_name_offset: Point2,
    // Padding between two subsequent signal names
    signal_name_padding: f32,
}

impl Style
{
    fn new() -> Style
    {
	Style{
	    font_size: 15,
	    signal_name_offset: pt2(0.0, 6.0),
	    signal_name_padding: 4.0,
	}
    }
}

#[derive(Error, Debug)]
pub enum DebugObjectError
{
    #[error("No name given for the DebugObject")]
    NoNameGiven,
    #[error("Invalid format {0}")]
    InvalidFormat(String),
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

#[derive(Debug)]
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

#[derive(Debug)]
struct ScopeSignalConfig
{
    name: String,
    min: f32,
    max: f32,
    y_size: f32,
    y_base: f32,
    color: Color,
}

impl ScopeSignalConfig
{
    fn from_tokens(tokens: &Vec<String>) -> Result<ScopeSignalConfig, DebugObjectError>
    {
	let name = tokens.get(0).ok_or(DebugObjectError::NoNameGiven)?;
	let min = tokens.get(1).ok_or(DebugObjectError::IndexError)?.parse::<f32>()?;
	let max = tokens.get(2).ok_or(DebugObjectError::IndexError)?.parse::<f32>()?;
	let y_size = tokens.get(3).ok_or(DebugObjectError::IndexError)?.parse::<f32>()?;
	let y_base = tokens.get(4).ok_or(DebugObjectError::IndexError)?.parse::<f32>()?;
	let mut color = YELLOW;
	if let Some(legend_or_color) = tokens.get(5)
	{
	    if legend_or_color.starts_with("%") {
		if let Some(color_name) = tokens.get(6) {
		    color = *COLOR_MAP.get::<str>(&color_name).unwrap_or(&YELLOW);
		}
	    }
	}

	Ok(ScopeSignalConfig{
	    name: strip_single_quotes(name).to_string(),
	    min,
	    max,
	    y_size,
	    y_base,
	    color,
	})
    }
}

struct ScopeSignal
{
    name: String,
    min: f32,
    max: f32,
    y_size: f32,
    y_base: f32,
    //{legend
    color: Color,
    pub values: VecDeque<f32>,
}

pub struct Scope
{
    name: String,
    samples: usize, // Number of retained samples
    rect: Rect,
    background: Color,
    grid: Color,
    signals: Vec<ScopeSignal>
}

impl Scope {

    pub fn new(tokens: &Vec<String>) -> Result<Scope, DebugObjectError>
    {
	let config = ScopeConfig::from_tokens(tokens)?;

	let res = Scope{
	    name: config.name,
	    samples: config.samples,
	    rect: Rect::from_x_y_w_h(config.pos.x, config.pos.y, config.size.x, config.size.y),
	    background: BLACK,
	    grid: GREY,
	    signals: vec![],
	};
	Ok(res)
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

    pub fn setup_signal(&mut self, tokens: &Vec<String>) -> Result<(), DebugObjectError>
    {
	println!("setup_signal: {:?}", tokens);
	let sc = ScopeSignalConfig::from_tokens(tokens)?;
	println!("setup_signal: {:?}", sc);
	self.signals.push(
	    ScopeSignal
	    {
	       name: sc.name,
	       min: sc.min,
	       max: sc.max,
	       y_size: sc.y_size,
	       y_base: sc.y_base,
	       color: sc.color,
	       values: VecDeque::from(vec![0.0, 0.0])
	    });
	Ok(())
    }
}

impl DebugProcessor for Scope {

    fn name(&self) -> String {
	self.name.clone()
    }

    fn draw(&self, draw: &nannou::draw::Draw)
    {
	let style = Style::new();

	let xy = self.rect.xy();
	let wh = self.rect.wh();

	let draw = draw.y(-wh.y);

	let mut cursor = pt2(0.0, wh.y) + style.signal_name_offset;

	fn draw_signal_name(draw: &nannou::draw::Draw, signal: &ScopeSignal, cursor: Point2, style: &Style) -> Point2
	{
	    // the rectangle is for wrapping, so we make it really big to avoid that wrapping
	    let text = text(&signal.name).font_size(style.font_size).build(Rect::from_w_h(1000.0, 1000.0));
	    let bounding_rect = text.bounding_rect();
	    draw.xy(cursor + bounding_rect.wh() / 2.0).path().fill().color(signal.color).events(text.path_events());
	    cursor + pt2(bounding_rect.w() + style.signal_name_padding, 0.0)
	}

	let step = wh.x / (self.samples as f32 - 1.0);

	draw.rect().xy(xy + wh / 2.0).wh(wh).color(self.background);
	draw.line().weight(1.0).color(self.grid).start(xy).end(xy + pt2(wh.x, 0.0));
	draw.line().weight(1.0).color(self.grid).start(xy).end(xy + pt2(0.0, wh.y));
	draw.line().weight(1.0).color(self.grid).start(xy + pt2(wh.x, 0.0)).end(xy + wh);
	draw.line().weight(1.0).color(self.grid).start(xy + pt2(0.0, wh.y)).end(xy + wh);
	self.signals.iter().for_each(|signal| {
	    // Lower/Upper Boundary
	    for v in &[signal.min, signal.max] {
		let v = map_range(*v, signal.min, signal.max, 0.0, -signal.y_size) + wh.y - signal.y_base;
		draw.line().weight(1.0).color(self.grid).start(pt2(0.0, v)).end(pt2(wh.x, 0.0) + pt2(0.0, v));
	    }
	    cursor = draw_signal_name(&draw, signal, cursor, &style);

	    // Draw the actual waveform
	    let vertices = signal.values.iter().enumerate()
		.map(|(i, value)| {
		    let v = map_range(*value, signal.min, signal.max, 0.0, signal.y_size) - signal.y_size - signal.y_base + wh.y;
		    (pt2(i as f32 * step, v), signal.color)
		});
	    draw.polyline()
		.weight(1.0)
		.points_colored(vertices);
	});
    }

    fn feed(&mut self, tokens: Vec<String>)
    {
	let mut err = Ok(());
	let mut floats = vec![];
	for token in &tokens {
	    let mut token = token.clone();
	    if token.ends_with(",") {
		token.remove(token.len() - 1);
	    }
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
	if let Ok(line) = DebugLine::from_str(line) {
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
    }

    pub fn draw(&self, draw: &nannou::draw::Draw)
    {
	for (_, debug_object) in &self.objects {
	    debug_object.draw(draw);
	}
    }

    fn create(&self, keyword: &str, tokens: &Vec<String>) -> Option<DebugObject>
    {
	// We need at least one additional token afetr the
	// name, which will become the identifier.
	if tokens.len() >= 1 {
	    if keyword == "SCOPE" {
		debug!("created Scope object named {}", tokens[0]);
		if let Some(scope) = Scope::new(tokens).ok()
		{
		    return Some(DebugObject::Scope(scope))
		}
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

    fn to_tokens(tokens: &[&str]) -> Vec<String>
    {
	tokens.iter().map(|s| { s.to_string() }).collect()
    }

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

    #[test]
    fn test_configuration_commandline() {
	let tokens = to_tokens(&["MyScope", "SIZE", "254", "84", "SAMPLES", "128"]);
	let scope_config = ScopeConfig::from_tokens(&tokens).expect("invalid configuration");
	assert_eq!(scope_config.name, "MyScope");
	assert_eq!(scope_config.size, pt2(254.0, 84.0));
	assert_eq!(scope_config.samples, 128);
    }

    #[test]
    fn test_configuration_signal() {
	let tokens = to_tokens(&["'Sawtooth'", "0", "63", "64", "10", "%1111", "CYAN"]);
	let signal_config = ScopeSignalConfig::from_tokens(&tokens).expect("invalid configuration");
	assert_eq!(signal_config.name, "Sawtooth");
	assert_eq!(signal_config.min, 0.0);
	assert_eq!(signal_config.max, 63.0);
	assert_eq!(signal_config.y_size, 64.0);
	assert_eq!(signal_config.y_base, 10.0);
	assert_eq!(signal_config.color, CYAN);
    }

}

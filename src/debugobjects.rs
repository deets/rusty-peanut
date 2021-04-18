use nannou::prelude::*;
use std::collections::hash_map::HashMap;
use std::vec::Vec;
use std::collections::VecDeque;
use log::{debug, warn};

type Rect = nannou::geom::rect::Rect;
type Color = Rgb<u8>;

pub trait DebugProcessor
{
    fn name(&self) -> String;
    fn draw(&self, draw: &nannou::draw::Draw);
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

    pub fn feed(&mut self, values: Vec<f32>)
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

struct DebugLine
{
    keyword: String,
    tokens: Vec<String>
}

fn parse_line(line: &str) -> std::result::Result<DebugLine, String>
{
    let tokens:Vec<String> = line.split_whitespace().map(|s| { s.to_string() }).filter(|part| { part.len() > 0 }).collect();
    if tokens.len() > 0{
	let mut keyword = tokens[0].clone();
	if keyword.starts_with("`") {
	    keyword = keyword[1..].to_string();
	    return Ok(DebugLine{keyword: keyword, tokens: tokens[1..].to_vec()});
	}
    }
    Err(format!("Can't parse line '{}'", line))
}

impl DebugObjects
{
    pub fn feed(&mut self, line: &str)
    {
	debug!("feeding line {}", line);
	let line = parse_line(line).expect("error parsing line");
	if self.objects.contains_key(&line.keyword) {
	    debug!("found DebugObject, feeding to it");
	} else {
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

    fn create(&self, keyword: &str, tokens: &Vec<String>) -> Option<DebugObject>
    {
	// We need at least one additional token afetr the
	// name, which will become the identifier.
	if tokens.len() >= 1 {
	    if keyword == "SCOPE" {
		debug!("Createda Scope object named {}", tokens[0]);
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
    }

    #[test]
    fn instantiate_scope_configure_and_feed() {
	for line in String::from(SCOPE_DECLARATION).split_terminator("\r\n") {
	    debug_objects.feed(&line);
	}
    }
}

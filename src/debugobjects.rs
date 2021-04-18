use std::collections::hash_map::HashMap;
use std::vec::Vec;
use std::collections::VecDeque;
use log::{debug, warn};

pub trait DebugProcessor
{
    fn name(&self) -> String;
}

pub struct Scope
{
    name: String,
    pub values: VecDeque<f32>,
    length: usize

}

impl Scope {

    pub fn new(tokens: &Vec<String>) -> Scope
    {
	assert!(tokens.len() >= 1);
	let mut values = VecDeque::new();
	values.push_back(0.0);
	values.push_back(0.0);
	Scope{ name: tokens[0].clone(), values, length: 128 }
    }

    pub fn feed(&mut self, value: f32)
    {
	self.values.push_back(value);
	while self.values.len() >= self.length {
	    self.values.pop_front();
	}
    }
}

impl DebugProcessor for Scope {
    fn name(&self) -> String {
	self.name.clone()
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

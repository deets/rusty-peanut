extern crate nom;
use nom::combinator::opt;
use nom::sequence::pair;
use nom::character::complete::multispace0;
use nom::branch::alt;
use nom::sequence::{tuple, preceded, terminated, delimited};
use nom::character::complete::{
    one_of,
    alphanumeric1,
    char,
    multispace1
};
use nom::multi::{
    many_m_n,
    many0,
    many1
};
use nom::combinator::recognize;
use nom::{
    named,
    alt,
    tag,
    preceded,
    bytes::complete::{tag},
    sequence::{separated_pair},
    IResult,
};
use phf::phf_map;
use nannou::prelude::*;

type Color = Rgb<u8>;

mod ast {
    use super::*;


    #[derive(Debug, PartialEq)]
    pub struct Legend
    {
	pub max: bool,
	pub min: bool,
	pub max_line: bool,
	pub min_line: bool
    }

    #[derive(Debug, PartialEq)]
    pub enum DebugInstructionAtom
    {
	// Keywords
	SCOPE,
	// Name
	Identifier{value: String},
	// `Name
	Symbol{value: String},
	// 'String'
	String{value: String},
	// SCOPE Parameters
	Title(String),
	Pos(i64, i64),
	Size(i64, i64),
	Samples(i64),
	Rate(i64),
	DotSize(i64),
	LineSize(i64),
	TextSize(i64),
	Color{ background: Color, grid: Option<Color> },
	// TODO: packed data
    }

    #[derive(Debug, PartialEq)]
    pub enum DebugInstruction
    {
	SCOPE{ name: String, configurations: Vec<DebugInstructionAtom> },
	SignalDefinition{
	    name: String,
	    min: Option<i64>,
	    max: Option<i64>,
	    y_size: Option<i64>,
	    y_base: Option<i64>,
	    legend: Option<Legend>,
	    color: Option<Color>
	}
    }
}

static COLOR_MAP: phf::Map<&[u8], Color> = phf_map! {
    b"BLACK" => BLACK,
    b"WHITE" => WHITE,
    b"ORANGE" => ORANGE,
    b"BLUE" => BLUE,
    b"GREEN" => GREEN,
    b"CYAN" => CYAN,
    b"RED" => RED,
    b"MAGENTA" => MAGENTA,
    b"YELLOW" => YELLOW,
};

fn named_color_parser(input: &[u8]) -> IResult<&[u8], Color> {
    named!( color_name, alt!(
    tag!("BLACK") |
    tag!("WHITE") |
    tag!("ORANGE") |
    tag!("BLUE") |
    tag!("GREEN") |
    tag!("CYAN") |
    tag!("RED") |
    tag!("MAGENTA") |
    tag!("YELLOW")
    ));
    let (rest, color) = color_name(input)?;
    Ok((rest, *COLOR_MAP.get(color).unwrap()))
}

// Symbols
named!(scope_symbol, preceded!(tag!("`"), tag!("SCOPE")));

// Keywords
named!(title_keyword, tag!("TITLE"));
named!(pos_keyword, tag!("POS"));
named!(size_keyword, tag!("SIZE"));
named!(samples_keyword, tag!("SAMPLES"));
named!(rate_keyword, tag!("RATE"));
named!(dotsize_keyword, tag!("DOTSIZE"));
named!(linesize_keyword, tag!("LINESIZE"));
named!(textsize_keyword, tag!("TEXTSIZE"));
named!(color_keyword, tag!("COLOR"));

fn string_from_atom(identifier: &ast::DebugInstructionAtom) -> String
{
    match identifier {
	ast::DebugInstructionAtom::Identifier{ value: identifier } => { return identifier.clone(); },
	ast::DebugInstructionAtom::String{ value: string } => { return string.clone(); },
	_ => { panic!("Grave parsing error"); }
    };
}

fn identifier_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstructionAtom> {
    let (rest, value) =
	recognize(many1(
	    alt((
		alphanumeric1,
		tag("_")))
	))(input)?;
    let value = std::str::from_utf8(value).expect("parser error").to_string();
    Ok((rest, ast::DebugInstructionAtom::Identifier{ value }))
}

fn symbol_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstructionAtom> {
    let (rest, value) =
	preceded(
	    tag("`"),
	    identifier_parser
	)(input)?;
    Ok((rest, ast::DebugInstructionAtom::Symbol{ value: string_from_atom(&value) }))
}

fn string_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstructionAtom> {
    let (rest, value) =
	delimited(
	    tag("'"),
	    recognize(many0(
		alt((
		    alphanumeric1,
		    tag(" "),
		    tag("_")))
	    )),
	    tag("'")
	)(input)?;
    let value = std::str::from_utf8(value).expect("parser error").to_string();
    Ok((rest, ast::DebugInstructionAtom::String{ value }))
}

fn decimal(input: &[u8]) -> IResult<&[u8], i64> {
    let (rest, number_literal) = recognize(
	many1(
	    terminated(one_of("0123456789"), many0(char('_')))
	)
    )(input)?;
    let number = std::str::from_utf8(number_literal).expect("parser error").parse::<i64>().expect("parser error");
    Ok((rest, number))
}

fn gray_color_parser(input: &[u8]) -> IResult<&[u8], Color> {
    let (rest, (_name, level)) = separated_pair(
	alt((tag("GRAY"), tag("GREY"))),
	multispace1,
	decimal)(input)?;
    let level = (5 + level * 25) as u8;
    Ok((rest, Color::new(level, level, level)))
}

fn color_value_parser(input: &[u8]) -> IResult<&[u8], Color> {
    alt((gray_color_parser, named_color_parser))(input)
}

fn size_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstructionAtom> {
    let (rest, (_, (x, y))) = separated_pair(
	size_keyword,
	multispace1,
	separated_pair(
	    decimal,
	    multispace1,
	    decimal
	)
    )(input)?;
    Ok((rest, ast::DebugInstructionAtom::Size(x, y)))
}

fn pos_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstructionAtom> {
    let (rest, (_, (x, y))) = separated_pair(
	pos_keyword,
	multispace1,
	separated_pair(
	    decimal,
	    multispace1,
	    decimal
	)
    )(input)?;
    Ok((rest, ast::DebugInstructionAtom::Pos(x, y)))
}

fn samples_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstructionAtom> {
    let (rest, (_, samples)) = separated_pair(
	samples_keyword,
	multispace1,
	decimal,
    )(input)?;
    Ok((rest, ast::DebugInstructionAtom::Samples(samples)))
}

fn rate_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstructionAtom> {
    let (rest, (_, rate)) = separated_pair(
	rate_keyword,
	multispace1,
	decimal,
    )(input)?;
    Ok((rest, ast::DebugInstructionAtom::Rate(rate)))
}

fn color_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstructionAtom> {
    let (rest, colors) = preceded(
	color_keyword,
	many_m_n(1, 2, preceded(multispace1, color_value_parser)),
    )(input)?;
    let background = colors[0];
    let mut grid = None;
    if colors.len() == 2 {
	grid = Some(colors[1]);
    }
    Ok((rest, ast::DebugInstructionAtom::Color{ background, grid}))
}

fn title_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstructionAtom> {
    let (rest, (_, title)) = separated_pair(
	title_keyword,
	multispace1,
	string_parser,
    )(input)?;

    Ok((rest, ast::DebugInstructionAtom::Title(string_from_atom(&title))))
}

fn dotsize_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstructionAtom> {
    let (rest, (_, dotsize)) = separated_pair(
	dotsize_keyword,
	multispace1,
	decimal,
    )(input)?;
    Ok((rest, ast::DebugInstructionAtom::DotSize(dotsize)))
}

fn linesize_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstructionAtom> {
    let (rest, (_, linesize)) = separated_pair(
	linesize_keyword,
	multispace1,
	decimal,
    )(input)?;
    Ok((rest, ast::DebugInstructionAtom::LineSize(linesize)))
}

fn textsize_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstructionAtom> {
    let (rest, (_, textsize)) = separated_pair(
	textsize_keyword,
	multispace1,
	decimal,
    )(input)?;
    Ok((rest, ast::DebugInstructionAtom::TextSize(textsize)))
}

fn legend_parser(input: &[u8]) -> IResult<&[u8], ast::Legend> {
    let (rest, (max, min, max_line, min_line)) = preceded(
	tag("%"),
	tuple((
	    one_of("01"),
	    one_of("01"),
	    one_of("01"),
	    one_of("01"),
	))
    )(input)?;

    let max = if max == '1' { true } else { false };
    let min = if min == '1' { true } else { false };
    let max_line = if max_line == '1' { true } else { false };
    let min_line = if min_line == '1' { true } else { false };

    Ok((rest, ast::Legend{
	max: max,
	min: min,
	max_line: max_line,
	min_line: min_line
    }))
}

fn scope_definition_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
    let preamble = preceded(
	scope_symbol,
	preceded(
	    multispace1,
	    identifier_parser));
    let configurations = many0(
	preceded(multispace1,
		 alt((
		     title_parser, pos_parser, size_parser,
		     samples_parser, rate_parser, dotsize_parser,
		     linesize_parser, textsize_parser, color_parser))));
    let (rest, (name, configurations)) = separated_pair(
	preamble,
	multispace1,
	configurations)(input)?;
    Ok((rest, ast::DebugInstruction::SCOPE{ name: string_from_atom(&name), configurations }))
}

// The following parsers all imply a keyword with the
// SCOPE name in the beginning. E.g.
// `MyScope 1, 2, 3, 4
fn scope_signal_data_parser(input: &[u8]) -> IResult<&[u8], Vec<i64>> {
    let (rest, (first, mut tail)) = pair(
	decimal,
	many0(
	    preceded(
		preceded(tag(","), multispace0),
		decimal
	    )))(input)?;
    tail.insert(0, first);
    Ok((rest, tail))
}

// `MyScope 'Sawtooth' 0 63 64 10 %1111
fn legend_and_color_parser(input: &[u8]) -> IResult<&[u8], (Option<ast::Legend>, Color)>
{
    let (rest, (legend, color)) = pair(
	opt(terminated(legend_parser, multispace1)),
	color_value_parser
    )(input)?;
    Ok((rest, (legend, color)))
}

fn scope_signal_declaration_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction>
{
    let (rest, (name, arguments)) = pair(
	string_parser,
	opt(
	    pair(
		preceded(multispace1, decimal), // min
		opt(
		    pair(
			preceded(multispace1, decimal), // max
			opt(
			    pair(
				preceded(multispace1, decimal), // y_size
				opt(
				    pair(
					preceded(multispace1, decimal), // y_base
					opt(
					    preceded(multispace1, legend_and_color_parser),
					)
				    )
				)
			    )
			)
		    )
		)
	    )
	)
	)(input)?;

    let name = string_from_atom(&name);
    let mut min = None;
    let mut max = None;
    let mut y_size = None;
    let mut y_base = None;
    let mut color = None;
    let mut legend = None;

    if let Some((min_value, arguments)) = arguments {
	min = Some(min_value);
	if let Some((max_value, arguments)) = arguments {
	    max = Some(max_value);
	    if let Some((y_size_value, arguments)) = arguments {
		y_size = Some(y_size_value);
		if let Some((y_base_value, arguments)) = arguments {
		    y_base = Some(y_base_value);
		    if let Some((legend_value, color_value)) = arguments {
			color = Some(color_value);
			legend = legend_value;
		    }
		}
	    }
	}
    };

    let result = ast::DebugInstruction::SignalDefinition{
	name,
	min: min,
	max: max,
	y_size: y_size,
	y_base: y_base,
	legend: legend,
	color: color,
    };
    Ok((rest, result))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_color_value() {
	let (_rest, result) = color_value_parser(b"YELLOW").unwrap();
	assert_eq!(result, YELLOW);
	let (_rest, result) = color_value_parser(b"GRAY 1").unwrap();
	assert_eq!(result, Color::new(30, 30, 30));
	let (_rest, result) = color_value_parser(b"GRAY 10").unwrap();
	assert_eq!(result, Color::new(255, 255, 255));
    }

    #[test]
    fn parse_scope_configurations() {
	let (_rest, result) = title_parser(b"TITLE  'FooBarBaz'").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::Title("FooBarBaz".to_string()));
	let (_rest, result) = pos_parser(b"POS   100   200").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::Pos(100, 200));
	let (_rest, result) = size_parser(b"SIZE   100   200").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::Size(100, 200));
	let (_rest, result) = samples_parser(b"SAMPLES  128").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::Samples(128));
	let (_rest, result) = rate_parser(b"RATE  128").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::Rate(128));
	let (_rest, result) = dotsize_parser(b"DOTSIZE  12").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::DotSize(12));
	let (_rest, result) = linesize_parser(b"LINESIZE  12").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::LineSize(12));
	let (_rest, result) = textsize_parser(b"TEXTSIZE  12").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::TextSize(12));

	let (_rest, result) = color_parser(b"COLOR  YELLOW").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::Color{ background: YELLOW, grid: None });
	let (_rest, result) = color_parser(b"COLOR  YELLOW   GREEN").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::Color{ background: YELLOW, grid: Some(GREEN) });
    }

    #[test]
    fn parse_signal_legend() {
	let (_rest, result) = legend_parser(b"%1010").unwrap();
	assert_eq!(result, ast::Legend{
	    max: true, min: false, max_line: true, min_line: false}
	);
    }

    #[test]
    fn parse_symbols() {
	let (_rest, result) = symbol_parser(b"`SpaceSignal").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::Symbol{ value: "SpaceSignal".to_string() });
	let (rest, _result) = scope_symbol(b"`SCOPE").unwrap();
	assert_eq!(rest, b"");
    }

    #[test]
    fn parse_string() {
	let (_rest, result) = string_parser(b"'String'").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::String{ value: "String".to_string() });
	let (_rest, result) = string_parser(b"'String with Space'").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::String{ value: "String with Space".to_string() });
	let (_rest, result) = string_parser(b"'String_with_underscores'").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::String{ value: "String_with_underscores".to_string() });
    }

    #[test]
    fn parse_identifier() {
	let (_rest, result) = identifier_parser(b"Identifier").unwrap();
	assert_eq!(result, ast::DebugInstructionAtom::Identifier{ value: "Identifier".to_string() });
    }

    #[test]
    fn parse_scope_declaration() {
	let (_rest, result) = scope_definition_parser(b"`SCOPE MyScope SIZE 254 84 SAMPLES 128").unwrap();
	match result {
	    ast::DebugInstruction::SCOPE{ name, configurations } => {
		assert!(name == "MyScope".to_string());
	    },
	    _ => { assert!(false); }
	}
    }

    #[test]
    fn parse_scope_signal_data() {
	let (_rest, result) = scope_signal_data_parser(b"1, 2,  3, 4,5").unwrap();
	assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn parse_scope_signal_definition() {
	let (_rest, (legend, color)) = legend_and_color_parser(b"%1111 YELLOW").unwrap();
	assert_eq!(color, YELLOW);
	assert_eq!(legend, Some(ast::Legend{
	    max: true, min: true, max_line: true, min_line: true}
	));

	let (_rest, (legend, color)) = legend_and_color_parser(b"YELLOW").unwrap();
	assert_eq!(color, YELLOW);
	assert_eq!(legend, None);

	let (_rest, result) = scope_signal_declaration_parser(b"'Sawtooth'").unwrap();
	match result {
	    ast::DebugInstruction::SignalDefinition{
		name,
		min: _,
		max: _,
		y_base: _,
		y_size: _,
		legend: _,
		color: _
	    } => {
		assert_eq!(name, "Sawtooth".to_string());
	    },
	    _ => { assert!(false); }
	}

	let (_rest, result) = scope_signal_declaration_parser(b"'Sawtooth' 10 20 30 40 YELLOW").unwrap();
	match result {
	    ast::DebugInstruction::SignalDefinition{
		name,
		min,
		max,
		y_size,
		y_base,
		legend: _,
		color
	    } => {
		assert_eq!(name, "Sawtooth".to_string());
		assert_eq!(min, Some(10));
		assert_eq!(max, Some(20));
		assert_eq!(y_size, Some(30));
		assert_eq!(y_base, Some(40));
		assert_eq!(color, Some(YELLOW));
	    },
	    _ => { assert!(false); }
	}

	let (_rest, result) = scope_signal_declaration_parser(b"'Sawtooth' 10 20 30 40 %1111 YELLOW").unwrap();
	match result {
	    ast::DebugInstruction::SignalDefinition{
		name,
		min,
		max,
		y_size,
		y_base,
		legend,
		color
	    } => {
		assert_eq!(name, "Sawtooth".to_string());
		assert_eq!(min, Some(10));
		assert_eq!(max, Some(20));
		assert_eq!(y_size, Some(30));
		assert_eq!(y_base, Some(40));
		assert_eq!(color, Some(YELLOW));
		assert_eq!(legend, Some(ast::Legend{
		    max: true, min: true, max_line: true, min_line: true}
		));
	    },
	    _ => { assert!(false); }
	}
    }

}

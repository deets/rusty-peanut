extern crate nom;
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
    pub enum DebugInstruction
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
	Pos(i32, i32),
	Size(i32, i32),
	Samples(i32),
	Rate(i32),
	DotSize(i32),
	LineSize(i32),
	TextSize(i32),
	Color{ background: Color, grid: Option<Color> },
	// TODO: packed data
	// SCOPE Signal Configurations
	Legend{ max: bool, min: bool, max_line: bool, min_line: bool}
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

// Keywords
named!(scope_keyword, tag!("SCOPE"));
named!(title_keyword, tag!("TITLE"));
named!(pos_keyword, tag!("POS"));
named!(size_keyword, tag!("SIZE"));
named!(samples_keyword, tag!("SAMPLES"));
named!(rate_keyword, tag!("RATE"));
named!(dotsize_keyword, tag!("DOTSIZE"));
named!(linesize_keyword, tag!("LINESIZE"));
named!(textsize_keyword, tag!("TEXTSIZE"));
named!(color_keyword, tag!("COLOR"));

fn string_from_atom(identifier: &ast::DebugInstruction) -> String
{
    match identifier {
	ast::DebugInstruction::Identifier{ value: identifier } => { return identifier.clone(); },
	ast::DebugInstruction::String{ value: string } => { return string.clone(); },
	_ => { panic!("Grave parsing error"); }
    };
}

fn scope_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
    let (rest, _) = scope_keyword(input)?;
    Ok((rest, ast::DebugInstruction::SCOPE))
}

fn identifier_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
    let (rest, value) =
	recognize(many1(
	    alt((
		alphanumeric1,
		tag("_")))
	))(input)?;
    let value = std::str::from_utf8(value).expect("parser error").to_string();
    Ok((rest, ast::DebugInstruction::Identifier{ value }))
}

fn symbol_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
    let (rest, value) =
	preceded(
	    tag("`"),
	    identifier_parser
	)(input)?;
    Ok((rest, ast::DebugInstruction::Symbol{ value: string_from_atom(&value) }))
}

fn string_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
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
    Ok((rest, ast::DebugInstruction::String{ value }))
}

fn decimal(input: &[u8]) -> IResult<&[u8], i32> {
    let (rest, number_literal) = recognize(
	many1(
	    terminated(one_of("0123456789"), many0(char('_')))
	)
    )(input)?;
    let number = std::str::from_utf8(number_literal).expect("parser error").parse::<i32>().expect("parser error");
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

fn size_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
    let (rest, (_, (x, y))) = separated_pair(
	size_keyword,
	multispace1,
	separated_pair(
	    decimal,
	    multispace1,
	    decimal
	)
    )(input)?;
    Ok((rest, ast::DebugInstruction::Size(x, y)))
}

fn pos_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
    let (rest, (_, (x, y))) = separated_pair(
	pos_keyword,
	multispace1,
	separated_pair(
	    decimal,
	    multispace1,
	    decimal
	)
    )(input)?;
    Ok((rest, ast::DebugInstruction::Pos(x, y)))
}

fn samples_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
    let (rest, (_, samples)) = separated_pair(
	samples_keyword,
	multispace1,
	decimal,
    )(input)?;
    Ok((rest, ast::DebugInstruction::Samples(samples)))
}

fn rate_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
    let (rest, (_, rate)) = separated_pair(
	rate_keyword,
	multispace1,
	decimal,
    )(input)?;
    Ok((rest, ast::DebugInstruction::Rate(rate)))
}

fn color_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
    let (rest, colors) = preceded(
	color_keyword,
	many_m_n(1, 2, preceded(multispace1, color_value_parser)),
    )(input)?;
    let background = colors[0];
    let mut grid = None;
    if colors.len() == 2 {
	grid = Some(colors[1]);
    }
    Ok((rest, ast::DebugInstruction::Color{ background, grid}))
}

fn title_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
    let (rest, (_, title)) = separated_pair(
	title_keyword,
	multispace1,
	string_parser,
    )(input)?;

    Ok((rest, ast::DebugInstruction::Title(string_from_atom(&title))))
}

fn dotsize_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
    let (rest, (_, dotsize)) = separated_pair(
	dotsize_keyword,
	multispace1,
	decimal,
    )(input)?;
    Ok((rest, ast::DebugInstruction::DotSize(dotsize)))
}

fn linesize_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
    let (rest, (_, linesize)) = separated_pair(
	linesize_keyword,
	multispace1,
	decimal,
    )(input)?;
    Ok((rest, ast::DebugInstruction::LineSize(linesize)))
}

fn textsize_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
    let (rest, (_, textsize)) = separated_pair(
	textsize_keyword,
	multispace1,
	decimal,
    )(input)?;
    Ok((rest, ast::DebugInstruction::TextSize(textsize)))
}

fn legend_parser(input: &[u8]) -> IResult<&[u8], ast::DebugInstruction> {
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

    Ok((rest, ast::DebugInstruction::Legend{
	max: max,
	min: min,
	max_line: max_line,
	min_line: min_line
    }))
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
	assert_eq!(result, ast::DebugInstruction::Title("FooBarBaz".to_string()));
	let (_rest, result) = pos_parser(b"POS   100   200").unwrap();
	assert_eq!(result, ast::DebugInstruction::Pos(100, 200));
	let (_rest, result) = size_parser(b"SIZE   100   200").unwrap();
	assert_eq!(result, ast::DebugInstruction::Size(100, 200));
	let (_rest, result) = samples_parser(b"SAMPLES  128").unwrap();
	assert_eq!(result, ast::DebugInstruction::Samples(128));
	let (_rest, result) = rate_parser(b"RATE  128").unwrap();
	assert_eq!(result, ast::DebugInstruction::Rate(128));
	let (_rest, result) = dotsize_parser(b"DOTSIZE  12").unwrap();
	assert_eq!(result, ast::DebugInstruction::DotSize(12));
	let (_rest, result) = linesize_parser(b"LINESIZE  12").unwrap();
	assert_eq!(result, ast::DebugInstruction::LineSize(12));
	let (_rest, result) = textsize_parser(b"TEXTSIZE  12").unwrap();
	assert_eq!(result, ast::DebugInstruction::TextSize(12));

	let (_rest, result) = color_parser(b"COLOR  YELLOW").unwrap();
	assert_eq!(result, ast::DebugInstruction::Color{ background: YELLOW, grid: None });
	let (_rest, result) = color_parser(b"COLOR  YELLOW   GREEN").unwrap();
	assert_eq!(result, ast::DebugInstruction::Color{ background: YELLOW, grid: Some(GREEN) });
    }

    #[test]
    fn parse_signal_legend() {
	let (_rest, result) = legend_parser(b"%1010").unwrap();
	assert_eq!(result, ast::DebugInstruction::Legend{
	    max: true, min: false, max_line: true, min_line: false}
	);
    }

    #[test]
    fn parse_keywords() {
	let (_rest, result) = scope_parser(b"SCOPE").unwrap();
	assert_eq!(result, ast::DebugInstruction::SCOPE);
    }

    #[test]
    fn parse_symbol() {
	let (_rest, result) = symbol_parser(b"`SpaceSignal").unwrap();
	assert_eq!(result, ast::DebugInstruction::Symbol{ value: "SpaceSignal".to_string() });
    }

    #[test]
    fn parse_string() {
	let (_rest, result) = string_parser(b"'String'").unwrap();
	assert_eq!(result, ast::DebugInstruction::String{ value: "String".to_string() });
	let (_rest, result) = string_parser(b"'String with Space'").unwrap();
	assert_eq!(result, ast::DebugInstruction::String{ value: "String with Space".to_string() });
	let (_rest, result) = string_parser(b"'String_with_underscores'").unwrap();
	assert_eq!(result, ast::DebugInstruction::String{ value: "String_with_underscores".to_string() });
    }

    #[test]
    fn parse_identifier() {
	let (_rest, result) = identifier_parser(b"Identifier").unwrap();
	assert_eq!(result, ast::DebugInstruction::Identifier{ value: "Identifier".to_string() });
    }
    // #[test]
    // fn parse_scope_declaration() {
    // 	"`SCOPE MyScope SIZE 254 84 SAMPLES 128"
    // }

}

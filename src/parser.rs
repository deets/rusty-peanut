extern crate nom;
use nom::branch::alt;
use nom::character::complete::char;
use nom::multi::many0;
use nom::character::complete::one_of;
use nom::sequence::terminated;
use nom::multi::many1;
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
    #[derive(Debug, PartialEq)]
    pub enum ScopeConfig
    {
	Size(i32, i32),
	Samples(i32),
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
	tag(" "),
	decimal)(input)?;
    let level = (5 + level * 25) as u8;
    Ok((rest, Color::new(level, level, level)))
}

fn color_parser(input: &[u8]) -> IResult<&[u8], Color> {
    alt((gray_color_parser, named_color_parser))(input)
}

fn size_parser(input: &[u8]) -> IResult<&[u8], ast::ScopeConfig> {
    let (rest, (_, (x, y))) = separated_pair(
	tag("SIZE"),
	tag(" "),
	separated_pair(
	    decimal,
	    tag(" "),
	    decimal
	)
    )(input)?;
    Ok((rest, ast::ScopeConfig::Size(x, y)))
}

fn samples_parser(input: &[u8]) -> IResult<&[u8], ast::ScopeConfig> {
    let (rest, (_, samples)) = separated_pair(
	tag("SAMPLES"),
	tag(" "),
	decimal,
    )(input)?;
    Ok((rest, ast::ScopeConfig::Samples(samples)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_color() {
	let (_rest, result) = color_parser(b"YELLOW").unwrap();
	assert_eq!(result, YELLOW);
	let (_rest, result) = color_parser(b"GRAY 1").unwrap();
	assert_eq!(result, Color::new(30, 30, 30));
	let (_rest, result) = color_parser(b"GRAY 10").unwrap();
	assert_eq!(result, Color::new(255, 255, 255));
    }

    #[test]
    fn parse_size() {
	let (_rest, result) = size_parser(b"SIZE 100 200").unwrap();
	assert_eq!(result, ast::ScopeConfig::Size(100, 200));
    }

    #[test]
    fn parse_samples() {
	let (_rest, result) = samples_parser(b"SAMPLES 128").unwrap();
	assert_eq!(result, ast::ScopeConfig::Samples(128));
    }

}

extern crate nom;
use nom::branch::alt;
use nom::character::complete::char;
use nom::multi::many0;
use nom::character::complete::one_of;
use nom::sequence::terminated;
use nom::multi::many1;
use nom::combinator::recognize;
use nom::character::is_digit;
use nom::{
    named,
    alt,
    tag,
    take_while1,
    bytes::complete::{tag},
    sequence::{separated_pair},
    IResult,
};
use phf::phf_map;
use nannou::prelude::*;

type Color = Rgb<u8>;

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

fn decimal(input: &[u8]) -> IResult<&[u8], &[u8]> {
  recognize(
    many1(
      terminated(one_of("0123456789"), many0(char('_')))
    )
  )(input)
}

fn gray_color_parser(input: &[u8]) -> IResult<&[u8], Color> {
    named!(gray, alt!(
	tag!("GRAY") |
	tag!("GREY")));

    let (rest, (_name, level)) = separated_pair(
	gray,
	tag(" "),
	decimal)(input)?;
    let level = std::str::from_utf8(level).expect("parser error").parse::<i32>().expect("parser error");
    let level = (5 + level * 25) as u8;
    Ok((rest, Color::new(level, level, level)))
}

fn color_parser(input: &[u8]) -> IResult<&[u8], Color> {
    alt((gray_color_parser, named_color_parser))(input)
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

}

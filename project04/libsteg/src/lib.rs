use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
pub enum PPMError {
    BadHeader(String),
    BadFile(String),
    // IOError(io::Error),
}

#[derive(Debug)]
pub struct PPMHeader {
    pub magic_number: [u8; 2],
    pub width: u32,
    pub height: u32,
    pub max_color_value: u32,
}

impl PPMHeader {
    pub fn parse_from_file(f: &mut File) -> Result<PPMHeader, PPMError> {
        let header = PPMHeader {
            // magic_number: [80, 54],
            magic_number: PPMHeader::parse_magic_number(f)?,
            width: PPMHeader::string_to_u32(
                PPMHeader::bytes_to_ascii_string(PPMHeader::parse_width_from_file(f)?)?
                    .trim()
                    .to_string(),
            )?,
            height: PPMHeader::string_to_u32(
                PPMHeader::bytes_to_ascii_string(PPMHeader::parse_height_from_file(f)?)?
                    .trim()
                    .to_string(),
            )?,
            max_color_value: PPMHeader::string_to_u32(
                PPMHeader::bytes_to_ascii_string(parse_maximum_color_value(f)?)?
                    .trim()
                    .to_string(),
            )?,
        };

        Ok(header)
    }

    fn parse_width_from_file(f: &mut File) -> Result<Vec<u8>, PPMError> {
        let mut ret = vec![0u8; 0];

        ret.extend(parse_one_white_space(f)?);

        ret.extend(parse_dimension(f)?);

        Ok(ret)
    }

    fn parse_height_from_file(f: &mut File) -> Result<Vec<u8>, PPMError> {
        // eprintln!("Parsing height");
        Ok(parse_dimension(f)?)
    }

    fn bytes_to_ascii_string(bytes: Vec<u8>) -> Result<String, PPMError> {
        String::from_utf8(bytes)
            .map_err(|_| PPMError::BadHeader("Bad String in header".to_string()))
    }

    pub fn string_to_u32(s: String) -> Result<u32, PPMError> {
        s.parse::<u32>()
            .map_err(|_| PPMError::BadHeader("Invalid number in header".to_string()))
    }

    fn parse_magic_number(f: &mut File) -> Result<[u8; 2], PPMError> {
        let mut magic_number_bytes = [0u8; 2];

        match f.read(&mut magic_number_bytes) {
            Ok(2) => {
                // check to see if the magic number is correct!
                // if char::from(magic_number_bytes[0]) == 'P' && char::from
                let b1 = char::from(magic_number_bytes[0]);
                let b2 = char::from(magic_number_bytes[1]);

                match (b1, b2) {
                    ('P', '6') => Ok(magic_number_bytes),
                    _ => Err(PPMError::BadHeader(format!(
                        "Bad Magic Number: {}{}",
                        b1, b2
                    ))),
                }
            }
            Ok(n) => Err(PPMError::BadHeader(format!(
                "Could not read two bytes for magic number parsing! Read {} bytes!",
                n
            ))),
            Err(err) => Err(PPMError::BadHeader(err.to_string())),
        }
    }
}

#[derive(Debug)]
pub struct PPM {
    pub header: PPMHeader,
    pub pixels: Vec<u8>,
}

impl PPM {
    /// Creates a new PPM struct from the specified path.
    pub fn new(path: String) -> Result<PPM, PPMError> {
        let mut f = match File::open(path) {
            Ok(f) => f,
            Err(err) => return Err(PPMError::BadFile(err.to_string())),
        };

        let header = PPMHeader::parse_from_file(&mut f)?;

        let mut pixels: Vec<u8> = Vec::new();

        match f.read_to_end(&mut pixels) {
            Ok(_) => Ok(PPM { header, pixels }),
            Err(err) => Err(PPMError::BadFile(err.to_string())),
        }
    }
}

fn parse_dimension(f: &mut File) -> Result<Vec<u8>, PPMError> {
    // eprintln!("Parsing dimension");

    let mut ret = vec![0u8; 0];

    let mut b = [0u8; 1];

    // now we keep reading until we hit something that is not white space

    let mut digit_start_found = false;

    loop {
        match f.read(&mut b) {
            Ok(1) if !digit_start_found => {
                // probably need to put extra error checking
                // stuff here, although we should make it to
                // the EOF while parsing dimension Error
                // if we never find a digit.
                if is_white_space(b[0]) {
                    ret.extend(&b);
                } else if is_digit(b[0]) {
                    digit_start_found = true;
                    ret.extend(&b);
                }
            }
            Ok(1) => {
                // either we read a digit, in which case
                // we are good and keep looking for more digits
                // or we read a white space character, in which case
                // we have reached the end of our width field
                // or we read something else
                // in which case we have an error
                if is_digit(b[0]) {
                    ret.extend(&b);
                } else if is_white_space(b[0]) {
                    ret.extend(&b);
                    return Ok(ret);
                } else {
                    return Err(PPMError::BadHeader(
                        "Unexpected character when parsing dimensino in header".to_string(),
                    ));
                }
            }
            Ok(_) => {
                return Err(PPMError::BadHeader(
                    "EOF while parsing dimension".to_string(),
                ));
            }
            Err(error) => {
                return Err(PPMError::BadHeader(error.to_string()));
            }
        }
    }
}

fn parse_one_white_space(f: &mut File) -> Result<Vec<u8>, PPMError> {
    let mut ret = vec![0u8; 0];

    let mut b = [0u8; 1];

    match f.read(&mut b) {
        Ok(1) => {
            // we got one byte, if it is white space, we can stick
            // it into our result
            if is_white_space(b[0]) {
                ret.extend(&b);
                Ok(ret)
            } else {
                Err(PPMError::BadHeader(format!(
                    "Expected white space, got: {}",
                    b[0]
                )))
            }
        }
        Ok(_) => Err(PPMError::BadHeader(
            "Reached end EOF while looking for a single white space character!".to_string(),
        )),
        Err(err) => Err(PPMError::BadHeader(err.to_string())),
    }
}

fn is_white_space(b: u8) -> bool {
    match char::from(b) {
        '\n' | ' ' | '\t' | '\r' => true,
        _ => false,
    }
}

fn is_digit(b: u8) -> bool {
    (b >= 48) && (b <= 57)
}

fn parse_maximum_color_value(f: &mut File) -> Result<Vec<u8>, PPMError> {
    let mut digit_start_found = false;

    let mut digit_start_index = 0;

    let mut ret = vec![0u8; 0];

    let mut b = [0u8; 1];

    loop {
        // this feels done poorly...
        match f.read(&mut b) {
            Ok(1) if !digit_start_found => {
                if is_white_space(b[0]) {
                    ret.extend(&b);
                    digit_start_index += 1;
                } else if is_digit(b[0]) {
                    ret.extend(&b);
                    digit_start_found = true;
                } else {
                    return Err(PPMError::BadHeader(
                        "Found a non digit when parsing max color value!".to_string(),
                    ));
                }
            }
            Ok(1) => {
                if is_digit(b[0]) {
                    ret.extend(&b);
                } else if is_white_space(b[0]) {
                    ret.extend(&b);
                    // ok, we've reached the end of searching for our digits.
                    // let's now let's make sure that it's legit
                    // eprintln!("ret.len() = {}", ret.len());
                    // eprintln!("ret: {:?}", ret);
                    let digits = &ret[digit_start_index..ret.len() - 1];
                    // eprintln!("digits.len() = {}", digits.len());
                    // eprintln!("digits: {:?}", digits);
                    match digits.len() {
                        1..=2 => {
                            // good
                            return Ok(ret);
                        }
                        3 if (digits[0] <= 50 && digits[1] <= 53 && digits[2] <= 53) => {
                            // eprintln!("ret.len() = {}", ret.len());
                            // eprintln!("ret: {:?}", ret);
                            return Ok(ret);
                        }
                        _ => {
                            return Err(PPMError::BadHeader(
                                "Max color value bigger than 255!".to_string(),
                            ))
                        }
                    }
                } else {
                    return Err(PPMError::BadHeader(
                        "Found a non digit when parsing max color value!".to_string(),
                    ));
                }
            }
            Ok(_) => {
                return Err(PPMError::BadHeader(
                    "Reached end EOF while parsing max color value".to_string(),
                ));
            }
            Err(err) => {
                return Err(PPMError::BadHeader(err.to_string()));
            }
        }
    }
}

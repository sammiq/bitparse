use lexopt::ValueExt;

struct Args {
    input: String,
    width: Option<u32>,
    unpack: Vec<u32>,
}

fn parse_args() -> Result<Args, lexopt::Error> {
    let mut input = None;
    let mut width = None;
    let mut unpack = Vec::new();

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            lexopt::Arg::Value(val) => {
                if input.is_none() {
                    input = Some(val.string()?);
                } else {
                    return Err(lexopt::Error::UnexpectedArgument(val));
                }
            }
            lexopt::Arg::Short('w') | lexopt::Arg::Long("width") => {
                let val = parser.value()?;

                match val.to_str().ok_or("error parsing width")? {
                    "b" => width = Some(8),
                    "w" => width = Some(16),
                    "d" => width = Some(32),
                    "q" => width = Some(64),
                    _ => return Err(lexopt::Error::UnexpectedArgument(val)),
                }
            }
            lexopt::Arg::Short('u') | lexopt::Arg::Long("unpack") => {
                let val = parser.value()?;
                let val2 = val.clone();
                let val_str = val.string()?;
                for num_str in val_str.split(',') {
                    let num: u32 = num_str.parse().map_err(|_| lexopt::Error::UnexpectedArgument(val2.clone()))?;
                    unpack.push(num);
                }
                unpack.sort();
            }
            lexopt::Arg::Long("help") => {
                usage();
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Args {
        input: input.ok_or("missing input argument")?,
        width,
        unpack,
    })
}

fn usage() {
    println!("Usage: bitparse [-w|--width=b|w|d|q] [-u|--unpack=offset[,offset]] <value>");
    println!("<value> can be in decimal, or prefixed with 0x (hex), 0o (octal), or 0b (binary).");
    println!("Options:");
    println!("  -w, --width=[b|w|d|q]\t\t Force set bit width");
    println!("  -u, --unpack=offset[,offset]\t Unpack fields at specified bit offsets");
    std::process::exit(0);
}


fn adjust_width(width: usize) -> u32 {
    match width {
        1..=8 => 8,
        9..=16 => 16,
        17..=32 => 32,
        33..=64 => 64,
        _ => 64,
    }
}

fn main() {
    let mut args = parse_args().unwrap_or_else(|e| {
        eprintln!("Error parsing arguments: {}\n", e);
        usage();
        std::process::exit(1);
    });

    let value = if args.input.starts_with("0x") {
        let input = args.input.trim_start_matches("0x");
        if args.width.is_none() {
            args.width = Some(adjust_width(input.len().div_ceil(2) * 8));
        }
        u64::from_str_radix(input, 16).expect("Failed to parse hex input")
    } else if args.input.starts_with("0o") {
        u64::from_str_radix(args.input.trim_start_matches("0o"), 8).expect("Failed to parse octal input")
    } else if args.input.starts_with("0b") {
        let input = args.input.trim_start_matches("0b");
        if args.width.is_none() {
            args.width = Some(adjust_width(input.len().div_ceil(8) * 8));
        }
        u64::from_str_radix(input, 2).expect("Failed to parse binary input")
    } else {
        args.input.parse::<u64>().expect("Failed to parse decimal input")
    };

    let width = args.width.unwrap_or({
        if value > u32::MAX as u64 {
            64
        } else if value > u16::MAX as u64 {
            32
        } else if value > u8::MAX as u64 {
            16
        } else {
            8
        }
    });

    println!("Unsigned decimal: {}", value);
    match width {
        64 => {
            println!("Signed decimal: {}", value as i64);
            println!("Hexadecimal: 0x{:016X}", value);
            println!("Octal: 0o{:024o}", value);
            println!("Binary: 0b{:064b}", value);
            //grab the bits and try to print as a double
            let bits = value;
            let double = f64::from_bits(bits);
            println!("Double-precision float: {}", double);
        }
        32 => {
            println!("Signed decimal: {}", value as i32);
            println!("Hexadecimal: 0x{:08X}", value);
            println!("Octal: 0o{:012o}", value);
            println!("Binary: 0b{:032b}", value);
            //grab the bits and try to print as a float
            let bits = value as u32;
            let float = f32::from_bits(bits);
            println!("Single-precision float: {}", float);
        }
        16 => {
            println!("Signed decimal: {}", value as i16);
            println!("Hexadecimal: 0x{:04X}", value);
            println!("Octal: 0o{:06o}", value);
            println!("Binary: 0b{:016b}", value);
        }
        8 => {
            println!("Signed decimal: {}", value as i8);
            println!("Hexadecimal: 0x{:02X}", value);
            println!("Octal: 0o{:03o}", value);
            println!("Binary: 0b{:08b}", value);
        }
        _ => unreachable!(),
    }
    println!("Bits:");
    for i in (0..width).rev() {
        let bit = if ((value >> i) & 1) != 0 { '1' } else { '0' };
        print!("{} ", bit);
        if i % 8 == 0 && i != 0 {
            print!("| ");
        }
    }
    println!();
    for i in (0..width).rev().step_by(8) {
        print!("    {:>2} - {:<2}       ", i, i - 7);
    }
    println!();
    if !args.unpack.is_empty() {
        println!("Unpacked fields:");
        for i in 0..args.unpack.len() - 1 {
            let this_offset = args.unpack[i];
            if this_offset >= width {
                break;
            }
            let next_offset = args.unpack.get(i + 1).copied().unwrap_or(width);
            let width = next_offset - this_offset;
            let field_value = (value >> this_offset) & ((1u64 << width) - 1);
            println!(
                "  Bits {:>2} to {:>2}: {} (0x{:02X}) (0b{:0width$b})",
                this_offset,
                next_offset - 1,
                field_value,
                field_value,
                field_value,
                width = width as usize
            );
        }
    }
}

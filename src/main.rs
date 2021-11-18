use std::error::Error;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
struct CanvasPixel {
    artist: u32,
    x: u8,
    y: u8,
    color: Color,
}

impl CanvasPixel {
    fn new(artist: u32, x: u8, y: u8, red: u8, green: u8, blue: u8) -> CanvasPixel {
        CanvasPixel {
            artist,
            x,
            y,
            color: Color::new(red, green, blue),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.r, self.g, self.b)
    }
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
}

// TODO
// allow specifying file name
// allow default size to analyze
fn main() -> Result<(), Box<dyn Error>> {
    let logfile = match File::open("../a5-sam-gab-swag/canvas.log") {
        Ok(file) => {
            println!("Successfully found log file {}.", "canvas.log");
            file
        }
        Err(data) => {
            println!("File {} not found.", "canvas.log");
            return Err(data)?;
        }
    };

    let log = BufReader::new(logfile);

    let log_lines = log
        .lines()
        .map(|l| l.expect("Error reading line.\n"))
        .collect::<Vec<_>>();

    let canvas = read_log_to_canvas(log_lines)?;

    // begin analysis
    // first, verify that all artists have a unique color
    println!("Verifying that all artists use unique colors...");

    let mut color_set: HashMap<Color, u32> = HashMap::new();
    for pixel in &canvas {
        if let Some(artist) = color_set.get(&pixel.color) {
            if *artist == pixel.artist {
                continue
            } else {
                eprintln!("Artist {} uses color {}, which is also used by artist {}",
                    pixel.artist, pixel.color, color_set.get(&pixel.color).unwrap());
            }
        } else {
            color_set.insert(pixel.color, pixel.artist);
        }
    }
    println!("All artists use unique colors!");

    // next, verify that no artists paint over one another
    println!("Verifying that no artists paint over each other...");

    let mut posns_map: HashMap<u32, HashSet<(u8, u8)>> = HashMap::new();
    for pixel in &canvas {
        match posns_map.get_mut(&pixel.artist) {
            Some(set) => {
                let res = set.insert((pixel.x, pixel.y));
                if !res {
                    eprintln!("Artist {} already painted at position {:#?}!", 
                        pixel.artist, (pixel.x, pixel.y));
                }
            }
            None => {
                let mut posn_set = HashSet::new();
                posn_set.insert((pixel.x, pixel.y));
                posns_map.insert(pixel.artist, posn_set);
            }
        }
    }

    println!("All positions aggregated, cross-checking artist positions...");
    for (artist, posns) in posns_map.iter() {
        for (other_artist, other_posns) in posns_map.iter() {
            if artist == other_artist {
                continue
            } else {
                let intersect: Vec<&(u8, u8)> = posns.intersection(other_posns).collect();
                if intersect.len() > 0 {
                    eprintln!("=============================");
                    eprintln!("Artist {} overlaps with artist {} at the following points:", artist, other_artist);
                    for point in intersect {
                        eprintln!("{:#?}", point);
                    }
                    eprintln!("All errors for artist {} complete.", artist);
                    eprintln!("=============================");
                }
            }
        }
    }
    println!("Finshed verifying artist positions.");

    // finally, double check for artists receiving the same random value. this can be done
    // by analyzing their points, to see if two sets of points are isomorphic
    println!("Checking for duplicated artist patterns...");



    Ok(())
}

// format of lines:
// artist_tid, x, y, r, g, b\n
fn read_log_to_canvas(lines: Vec<String>) -> Result<Vec<CanvasPixel>, Box<dyn Error>> {
    let mut pixels: Vec<CanvasPixel> = Vec::new();
    for (lnum, line) in lines.iter().enumerate() {
        let parts: Vec<&str> = line.split(' ').map(|s| s.trim_end_matches(',')).collect();
        if parts.len() != 6 {
            Err("Line is formatted improperly".to_string())?
        }
        println!("{:?}", parts);
        let artist_tid = try_parse::<u32>(parts[0], "artist", lnum)?;
        let x_pos = try_parse::<u8>(parts[1], "x", lnum)?;
        let y_pos = try_parse::<u8>(parts[2], "y", lnum)?;
        let red = try_parse::<u8>(parts[3], "red", lnum)?;
        let green = try_parse::<u8>(parts[4], "green", lnum)?;
        let blue = try_parse::<u8>(parts[5], "blue", lnum)?;

        pixels.push(CanvasPixel::new(artist_tid, x_pos, y_pos, red, green, blue));
    }

    println!("{} pixels were painted", pixels.len());
    Ok(pixels)
}

fn try_parse<T: FromStr>(s: &str, name: &str, line_num: usize) -> Result<T, <T as FromStr>::Err> {
    match s.parse::<T>() {
        Ok(val) => Ok(val),
        Err(e) => {
            eprintln!("Failed to parse {} on line {}; formatted incorrectly", name, line_num);
            Err(e)
        }
    }
}

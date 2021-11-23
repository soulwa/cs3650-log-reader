use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

type Canvas = Vec<CanvasPixel>;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
struct CanvasPixel {
    artist: u32,
    coord: Point,
    color: Color,
}

impl CanvasPixel {
    fn new(artist: u32, x: i16, y: i16, red: u8, green: u8, blue: u8) -> CanvasPixel {
        CanvasPixel {
            artist,
            coord: Point::new(x, y),
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

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
struct Point {
    x: i16,
    y: i16,
}

impl Point {
    fn new(x: i16, y: i16) -> Point {
        Point { x, y }
    }
}

// similarly here, ordering by rightmost point
impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// trick here: we just care about *some* relative ordering, so ordering by rightmost point.
impl Ord for Point {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.x.cmp(&other.x)
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

    // read the file data into out data structure
    let log = BufReader::new(logfile);

    let log_lines = log
        .lines()
        .map(|l| l.expect("Error reading line.\n"))
        .collect::<Vec<_>>();

    let canvas = read_log_to_canvas(log_lines)?;

    // begin analysis
    // initialize all the useful data structures for analysis beforehand
    let mut posns_map: HashMap<u32, HashSet<Point>> = HashMap::new();
    println!("Initializing artist and color data...");
    for pixel in &canvas {
        match posns_map.get_mut(&pixel.artist) {
            Some(set) => {
                let res = set.insert(pixel.coord);
                if !res {
                    eprintln!(
                        "Artist {} already painted at position {:#?}!",
                        pixel.artist, pixel.coord
                    );
                }
            }
            None => {
                let mut posn_set = HashSet::new();
                posn_set.insert(pixel.coord);
                posns_map.insert(pixel.artist, posn_set);
            }
        }
    }

    // verify that a sufficient number of artists exist
    // possible but highly unlikely this fails due to starvation, not a lack of generation
    print_err_msg(check_enough_artists(
        &posns_map.keys().collect::<Vec<_>>()[..],
        54,
    ));

    // check that all artists draw at least one pixel
    print_err_msg(check_all_artists_draw(&posns_map, 1));

    // verify that all artists have a unique color
    print_err_msg(check_colors_unique(&canvas));

    // verify that no artists paint over one another
    print_err_msg(check_no_overlapping(&posns_map));

    // verify that there are no islands in the log file
    print_err_msg(check_no_islands(&posns_map));

    // double check for artists receiving the same random value. this can be done
    // by analyzing their points, to see if two sets of points are isomorphic
    // this is impossible if each thread has its own rng, but some patterns may not
    // show up even if they have the same rng because of competing for pixels within the pattern
    print_err_msg(check_no_repeating_patterns(posns_map));

    println!("Finished analyzing the log.");

    Ok(())
}

fn check_enough_artists(artists: &[&u32], num_artists: usize) -> Result<(), String> {
    if artists.len() != num_artists {
        Err(format!(
            "Expected {} artists, but found {}; incorrect number of artists painted!",
            num_artists,
            artists.len()
        ))
    } else {
        println!("Found {} artists!", num_artists);
        Ok(())
    }
}

fn check_all_artists_draw(
    posns_map: &HashMap<u32, HashSet<Point>>,
    num_pixels: usize,
) -> Result<(), String> {
    println!(
        "Verifying that all artists draw at least {} pixels...",
        num_pixels
    );
    let mut draw_error = false;
    for (artist, points) in posns_map.iter() {
        if points.len() < num_pixels {
            draw_error = true;
            eprintln!(
                "Artist {} drew {} pixels; should draw at least {} pixels.",
                artist,
                points.len(),
                num_pixels
            );
        }
    }
    if draw_error {
        Err("Artists did not draw enough pixels: could be starved, but make sure that all artists have a chance to draw!".to_string())
    } else {
        println!("All artists draw at least {} pixels!", num_pixels);
        Ok(())
    }
}

fn check_colors_unique(canvas: &Canvas) -> Result<(), String> {
    println!("Verifying that all artists use unique colors...");
    let mut color_error = false;
    let mut color_set: HashMap<Color, u32> = HashMap::new();
    for pixel in canvas {
        if let Some(artist) = color_set.get(&pixel.color) {
            if *artist == pixel.artist {
                continue;
            } else {
                color_error = true;
                eprintln!(
                    "Artist {} uses color {}, which is also used by artist {}",
                    pixel.artist,
                    pixel.color,
                    color_set.get(&pixel.color).unwrap()
                );
            }
        } else {
            color_set.insert(pixel.color, pixel.artist);
        }
    }
    if color_error {
        return Err(
            "Ensure that each artist must have a unique color, when you generate artists!"
                .to_string(),
        );
    } else {
        println!("All artists use unique colors!");
    }

    Ok(())
}

fn check_no_overlapping(posns_map: &HashMap<u32, HashSet<Point>>) -> Result<(), String> {
    println!("Verifying that no artists paint over one another...");
    let mut posn_error = false;
    for (artist, posns) in posns_map.iter() {
        for (other_artist, other_posns) in posns_map.iter() {
            if artist == other_artist {
                continue;
            } else {
                let intersect: Vec<&Point> = posns.intersection(other_posns).collect();
                if intersect.len() > 0 {
                    posn_error = true;
                    eprintln!(
                        "Artist {} overlaps with artist {} at the following points:",
                        artist, other_artist
                    );
                    for point in intersect {
                        eprintln!("{:#?}", point);
                    }
                    eprintln!("All errors for artist {} complete.", artist);
                }
            }
        }
    }
    if posn_error {
        return Err("Make sure that artists do not paint to the same position- you may need to lock the position or ensure artists skip the position if it is locked.".to_string());
    } else {
        println!("All artists paint on separate pixels!");
    }

    Ok(())
}

fn check_no_islands(posns_map: &HashMap<u32, HashSet<Point>>) -> Result<(), String> {
    println!("Verifying that all pixels are connected to pixels of the same color...");
    unimplemented!()
}

fn check_no_repeating_patterns(posns_map: HashMap<u32, HashSet<Point>>) -> Result<(), String> {
    println!("Checking for duplicated artist patterns...");
    let normalized: Vec<HashSet<Point>> = posns_map
        .into_values()
        .map(|set| normalize_points(set).expect("Failed to normalize set: "))
        .collect();

    let mut duplicates: HashSet<(usize, usize)> = HashSet::new();

    for (ii, set) in normalized.iter().enumerate() {
        for (jj, other_set) in normalized.iter().enumerate() {
            if ii == jj {
                continue;
            } else if set.is_subset(other_set) && set.is_superset(other_set) {
                duplicates.insert((min(ii, jj), max(ii, jj)));
                eprintln!(
                    "Duplicate pattern found! So far, found {} duplicates",
                    duplicates.len()
                );
            }
        }
    }

    if duplicates.is_empty() {
        Ok(())
    } else {
        Err(format!("Found {} duplicate patterns", duplicates.len()))
    }
}

fn normalize_points(points: HashSet<Point>) -> Result<HashSet<Point>, String> {
    // find the rightmost point for the relative "origin"
    let rightmost = match points.iter().max() {
        Some(point) => point,
        None => {
            return Err("Error finding rightmost point in the set of points.".to_string());
        }
    };

    Ok(points
        .iter()
        .map(|p| Point::new(p.x - rightmost.x, p.y - rightmost.y))
        .collect::<HashSet<Point>>())
}

// format of lines:
// artist_tid, x, y, r, g, b\n
fn read_log_to_canvas(lines: Vec<String>) -> Result<Canvas, Box<dyn Error>> {
    let mut pixels: Vec<CanvasPixel> = Vec::new();
    for (lnum, line) in lines.iter().enumerate() {
        let parts: Vec<&str> = line.split(' ').map(|s| s.trim_end_matches(',')).collect();
        if parts.len() != 6 {
            Err("Line is formatted improperly".to_string())?
        }
        let artist_tid = try_parse::<u32>(parts[0], "artist", lnum)?;
        let x_pos = try_parse::<i16>(parts[1], "x", lnum)?;
        let y_pos = try_parse::<i16>(parts[2], "y", lnum)?;
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
            eprintln!(
                "Failed to parse {} on line {}; formatted incorrectly",
                name, line_num
            );
            Err(e)
        }
    }
}

fn print_err_msg<T, E: fmt::Display>(res: Result<T, E>) {
    if let Err(msg) = res {
        eprintln!("{}", msg);
    }
}

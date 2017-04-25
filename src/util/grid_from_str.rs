use point::Point;

/// Creates an object of the specified type from a grid string using constructor
/// and callback closures. Typically used for quickly producing 2D grids/worlds.
/// The grid string may look something like this:
/// ```
/// .....
/// .#.#.
/// ..@..
/// .#.#.
/// .....
/// ```
pub fn make_grid_from_str<M, F, T>(text: &str, mut constructor: M, mut callback: F) -> T
    where M: FnMut(Point) -> T,
          F: FnMut(&Point, char, &mut T) {
    let mut x = 0;
    let mut y = 0;

    let lines = text.split('\n').filter(|l| l.len() > 0).collect::<Vec<_>>();
    let height = lines.len();
    assert!(height > 0);
    let width = lines[0].len();
    assert!(width > 0);
    assert!(lines.iter().all(|line| line.chars().count() == width));
    let mut thing = constructor(Point::new(width as i32, height as i32));

    for line in lines {
        for ch_at_point in line.chars() {
            let grid_pos = Point { x: x as i32, y: y as i32 };
            callback(&grid_pos, ch_at_point, &mut thing);

            x += 1;
        }
        y += 1;
        x = 0;
    }

    thing
}

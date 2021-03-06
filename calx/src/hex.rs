use std::slice;
use std::ops::{Add, Sub};
use std::f32::consts::PI;
use std::cmp::max;
use rand::{Rand, Rng};
use num::{Integer, Float};
use geom::V2;

/// Hex grid geometry for vectors.
pub trait HexGeom {
    /// Hex distance represented by a vector.
    fn hex_dist(&self) -> i32;
}

impl HexGeom for V2<i32> {
    fn hex_dist(&self) -> i32 {
        let xd = self.0;
        let yd = self.1;
        if xd.signum() == yd.signum() {
            max(xd.abs(), yd.abs())
        } else {
            xd.abs() + yd.abs()
        }
    }
}

/// Hex grid directions.
#[derive(Copy, Eq, PartialEq, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum Dir6 {
    North = 0,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
}

impl Dir6 {
    /// Convert a vector into the closest hex direction.
    ///
    /// ```notrust
    ///        *0*       *1*
    ///           \ 14 15 | 00 01
    ///           13\     |      02
    ///               \   |
    ///         12      \ |        03
    ///     *5* ----------O-X------- *2*
    ///         11        Y \      04
    ///                   |   \
    ///           10      |     \05
    ///             09 08 | 07 06 \
    ///                  *4*       *3*
    ///
    /// The hexadecants (00 to 15) and the hex
    /// directions (*0* to *5*) around the origin.
    /// ```
    ///
    /// Vectors that are in a space between two hex direction vectors are
    /// rounded to a hexadecant, then assigned the hex direction whose vector
    /// is nearest to that hexadecant.
    pub fn from_v2(v: V2<i32>) -> Dir6 {
        let hexadecant = {
            let width = PI / 8.0;
            let mut radian = (v.0 as f32).atan2(-v.1 as f32);
            if radian < 0.0 { radian += 2.0 * PI }
            (radian / width).floor() as i32
        };

        Dir6::from_int(match hexadecant {
            13 | 14 => 0,
            15 | 0 | 1 => 1,
            2 | 3 | 4 => 2,
            5 | 6 => 3,
            7 | 8 | 9 => 4,
            10 | 11 | 12 => 5,
            _ => panic!("Bad hexadecant")
        })
    }

    /// Convert an integer to a hex dir using modular arithmetic.
    pub fn from_int(i: i32) -> Dir6 { DIRS[i.mod_floor(&6) as usize] }

    /// Convert a hex dir into the corresponding unit vector.
    pub fn to_v2(&self) -> V2<i32> {
        [V2(-1, -1),
         V2( 0, -1),
         V2( 1,  0),
         V2( 1,  1),
         V2( 0,  1),
         V2(-1,  0)][*self as usize]
    }

    /// Iterate through the six hex dirs in the standard order.
    pub fn iter() -> slice::Iter<'static, Dir6> { DIRS.iter() }
}

impl Add<i32> for Dir6 {
    type Output = Dir6;
    fn add(self, other: i32) -> Dir6 { Dir6::from_int(self as i32 + other) }
}

impl Sub<i32> for Dir6 {
    type Output = Dir6;
    fn sub(self, other: i32) -> Dir6 { Dir6::from_int(self as i32 - other) }
}

impl Rand for Dir6 {
    fn rand<R: Rng>(rng: &mut R) -> Dir6 { Dir6::from_int(rng.gen_range(0, 6)) }
}

static DIRS: [Dir6; 6] = [
    Dir6::North,
    Dir6::NorthEast,
    Dir6::SouthEast,
    Dir6::South,
    Dir6::SouthWest,
    Dir6::NorthWest];

/// Field of view iterator for a hexagonal map.
///
/// Takes a function that
/// indicates opaque cells and yields the visible locations from around the
/// origin up to the given radius.
pub struct HexFov<F> {
    /// Predicate for whether a given point will block the field of view.
    is_opaque: F,
    range: u32,
    stack: Vec<Sector>,
    fake_isometric_hack: bool,
    /// Extra values generated by special cases.
    side_channel: Vec<V2<i32>>,
}

impl<F> HexFov<F>
    where F: Fn(V2<i32>) -> bool
{
    pub fn new(is_opaque: F, range: u32) -> HexFov<F> {
        // The origin position V2(0, 0) is a special case for the traversal
        // algorithm, but it's also always present, so instead of adding ugly
        // branches to the actual iterator, we just chain it in right here.
        let init_group = is_opaque(Dir6::from_int(0).to_v2());
        HexFov {
            is_opaque: is_opaque,
            range: range,
            stack: vec![Sector {
                begin: PolarPoint::new(0.0, 1),
                pt: PolarPoint::new(0.0, 1),
                end: PolarPoint::new(6.0, 1),
                group_opaque: init_group,
            }],
            fake_isometric_hack: false,
            // The FOV algorithm will not generate the origin point, so we use
            // the side channel to explicitly add it in the beginning.
            side_channel: vec![V2(0, 0)],
        }
    }

    /// Make wall tiles in acute corners visible when running the algorithm so
    /// that the complete wall rectangle of fake-isometric rooms will appear
    /// in the FOV.
    pub fn fake_isometric(mut self) -> HexFov<F> {
        self.fake_isometric_hack = true;
        self
    }
}

impl<F> Iterator for HexFov<F>
    where F: Fn(V2<i32>) -> bool
{
    type Item = V2<i32>;
    fn next(&mut self) -> Option<V2<i32>> {
        if let Some(ret) = self.side_channel.pop() {
            return Some(ret);
        }

        if let Some(mut current) = self.stack.pop() {
            if current.pt.is_below(current.end) {
                let pos = current.pt.to_v2();
                let current_opaque = (self.is_opaque)(pos);

                // Terrain opacity changed, branch out.
                if current_opaque != current.group_opaque {
                    // Add the rest of this sector with the new opacity.
                    self.stack.push(Sector {
                        begin: current.pt,
                        pt: current.pt,
                        end: current.end,
                        group_opaque: current_opaque,
                    });

                    // If this was a visible sector and we're below range, branch
                    // out further.
                    if !current.group_opaque && current.begin.radius < self.range {
                        self.stack.push(Sector {
                            begin: current.begin.further(),
                            pt: current.begin.further(),
                            end: current.pt.further(),
                            group_opaque: (self.is_opaque)(current.begin.further().to_v2()),
                        });
                    }
                    return self.next();
                }

                // Hack for making acute corner tiles of fake-isometric rooms
                // visible.

                // XXX: Side cells should only be visible with wallform tiles,
                // but the FOV API can't currently distinguish between
                // wallform and blockform FOV blockers.
                if self.fake_isometric_hack {
                    if let Some(side_pt) = current.pt.side_point() {
                        // Only do this if both the front tiles and the target
                        // tile are opaque.
                        let next = current.pt.next();
                        if next.is_below(current.end)
                            && current.group_opaque
                            && (self.is_opaque)(next.to_v2())
                            && (self.is_opaque)(side_pt)
                            && current.begin.radius < self.range {
                                self.side_channel.push(side_pt);
                        }
                    }
                }

                // Proceed along the current sector.
                current.pt = current.pt.next();
                self.stack.push(current);
                return Some(pos);
            } else {
                // Hit the end of the sector.

                // If this was a visible sector and we're below range, branch
                // out further.
                if !current.group_opaque && current.begin.radius < self.range {
                    self.stack.push(Sector {
                        begin: current.begin.further(),
                        pt: current.begin.further(),
                        end: current.end.further(),
                        group_opaque: (self.is_opaque)(current.begin.further().to_v2()),
                    });
                }

                self.next()
            }
        } else {
            None
        }
    }
}

struct Sector {
    /// Start point of current sector.
    begin: PolarPoint,
    /// Point currently being processed.
    pt: PolarPoint,
    /// End point of current sector.
    end: PolarPoint,
    /// Currently iterating through a sequence of opaque cells.
    group_opaque: bool,
}

/// Points on a hex circle expressed in polar coordinates.
#[derive(Copy, Clone, PartialEq)]
struct PolarPoint {
    pos: f32,
    radius: u32
}

impl PolarPoint {
    pub fn new(pos: f32, radius: u32) -> PolarPoint { PolarPoint { pos: pos, radius: radius } }
    /// Index of the discrete hex cell along the circle that corresponds to this point.
    fn winding_index(self) -> i32 { (self.pos + 0.5).floor() as i32 }

    pub fn is_below(self, other: PolarPoint) -> bool { self.winding_index() < other.end_index() }
    fn end_index(self) -> i32 { (self.pos + 0.5).ceil() as i32 }

    pub fn to_v2(self) -> V2<i32> {
        if self.radius == 0 { return V2(0, 0); }
        let index = self.winding_index();
        let sector = index.mod_floor(&(self.radius as i32 * 6)) / self.radius as i32;
        let offset = index.mod_floor(&(self.radius as i32));
        let rod = Dir6::from_int(sector).to_v2() * (self.radius as i32);
        let tangent = Dir6::from_int((sector + 2) % 6).to_v2() * offset;
        rod + tangent
    }

    /// If this point and the next point are adjacent vertically (along the xy
    /// axis), return a tuple of the point outside of the circle between the
    /// two points.
    ///
    /// This is a helper function for the FOV special case where acute corners
    /// of fake isometric rooms are marked visible even though strict hex FOV
    /// logic would keep them unseen.
    pub fn side_point(self) -> Option<V2<i32>> {
        let next = self.next();
        let V2(x1, y1) = self.to_v2();
        let V2(x2, y2) = next.to_v2();

        if x2 == x1 + 1 && y2 == y1 + 1 {
            // Going down the right rim.
            Some(V2(x1 + 1, y1))
        } else if x2 == x1 - 1 && y2 == y1 - 1 {
            // Going up the left rim.
            Some(V2(x1 - 1, y1))
        } else {
            None
        }
    }

    /// The point corresponding to this one on the hex circle with radius +1.
    pub fn further(self) -> PolarPoint {
        PolarPoint::new(
            self.pos * (self.radius + 1) as f32 / self.radius as f32,
            self.radius + 1)
    }

    /// The point next to this one along the hex circle.
    pub fn next(self) -> PolarPoint {
        PolarPoint::new((self.pos + 0.5).floor() + 0.5, self.radius)
    }
}

#[cfg(test)]
mod test {
    use geom::V2;
    // XXX: Why doesn't super::* work here?
    use super::{Dir6};
    use super::Dir6::*;

    #[test]
    fn test_dir6() {
        assert_eq!(North, Dir6::from_int(0));
        assert_eq!(NorthWest, Dir6::from_int(-1));
        assert_eq!(NorthWest, Dir6::from_int(5));
        assert_eq!(NorthEast, Dir6::from_int(1));

        assert_eq!(NorthEast, Dir6::from_v2(V2(20, -21)));
        assert_eq!(SouthEast, Dir6::from_v2(V2(20, -10)));
        assert_eq!(North, Dir6::from_v2(V2(-10, -10)));
        assert_eq!(South, Dir6::from_v2(V2(1, 1)));

        for i in 0..6 {
            let d = Dir6::from_int(i);
            let v = d.to_v2();
            let v1 = Dir6::from_int(i - 1).to_v2();
            let v2 = Dir6::from_int(i + 1).to_v2();

            // Test static iter
            assert_eq!(Some(d), Dir6::iter().nth(i as usize).map(|&x| x));

            // Test vector mapping.
            assert_eq!(d, Dir6::from_v2(v));

            // Test opposite dir vector mapping.
            assert_eq!(Dir6::from_int(i + 3), Dir6::from_v2(-v));

            // Test approximation of longer vectors.
            assert_eq!(d, Dir6::from_v2(v * 3));
            assert_eq!(d, Dir6::from_v2(v * 3 + v1));
            assert_eq!(d, Dir6::from_v2(v * 3 + v2));
        }
    }
}

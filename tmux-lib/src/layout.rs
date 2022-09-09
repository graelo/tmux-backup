//! Parse the window layout string.
//!
//! Tmux reports the layout for a window, it can also use it to apply an existing layout to a
//! window.
//!
//! A window layout has this format:
//!
//! ```text
//! "41e9,279x71,0,0[279x40,0,0,71,279x30,0,41{147x30,0,41,72,131x30,148,41,73}]"
//! ```
//!
//! The parser in this module returns the corresponding [`WindowLayout`].

use nom::{
    branch::alt,
    character::complete::{char, digit1, hex_digit1},
    combinator::{all_consuming, map_res},
    multi::separated_list1,
    sequence::{delimited, tuple},
    IResult,
};

use crate::{error::map_add_intent, Result};

/// Represent a parsed window layout.
#[derive(Debug, PartialEq, Eq)]
pub struct WindowLayout {
    /// 4-char hex id, such as `9f58`.
    id: u16,
    /// Container.
    container: Container,
}

impl WindowLayout {
    /// Return a flat list of pane ids.
    pub fn pane_ids(&self) -> Vec<u16> {
        let mut acc: Vec<u16> = vec![];
        acc.reserve(1);
        self.walk(&mut acc);
        acc
    }

    /// Walk the structure, searching for pane ids.
    fn walk(&self, acc: &mut Vec<u16>) {
        self.container.walk(acc);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Container {
    /// Dimensions of the container.
    dimensions: Dimensions,
    /// Offset of the top left corner of the container.
    coordinates: Coordinates,
    /// Either a pane, or a horizontal or vertical split.
    element: Element,
}

impl Container {
    /// Walk the structure, searching for pane ids.
    fn walk(&self, acc: &mut Vec<u16>) {
        self.element.walk(acc);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Dimensions {
    /// Width (of the window or pane).
    width: u16,
    /// Height (of the window or pane).
    height: u16,
}

#[derive(Debug, PartialEq, Eq)]
struct Coordinates {
    /// Horizontal offset of the top left corner (of the window or pane).
    x: u16,
    /// Vertical offset of the top left corner (of the window or pane).
    y: u16,
}

/// Element in a container.
#[derive(Debug, PartialEq, Eq)]
enum Element {
    /// A pane.
    Pane { pane_id: u16 },
    /// A horizontal split.
    Horizontal(Split),
    /// A vertical split.
    Vertical(Split),
}

impl Element {
    /// Walk the structure, searching for pane ids.
    fn walk(&self, acc: &mut Vec<u16>) {
        match self {
            Self::Pane { pane_id } => acc.push(*pane_id),
            Self::Horizontal(split) | Self::Vertical(split) => {
                split.walk(acc);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Split {
    /// Embedded containers.
    elements: Vec<Container>,
}

impl Split {
    /// Walk the structure, searching for pane ids.
    fn walk(&self, acc: &mut Vec<u16>) {
        for element in &self.elements {
            element.walk(acc);
        }
    }
}

/// Parse the Tmux layout string description and return the pane-ids.
pub fn parse_window_layout(input: &str) -> Result<WindowLayout> {
    let desc = "window-layout";
    let intent = "window-layout";
    let (_, win_layout) =
        all_consuming(window_layout)(input).map_err(|e| map_add_intent(desc, intent, e))?;

    Ok(win_layout)
}

pub(crate) fn window_layout(input: &str) -> IResult<&str, WindowLayout> {
    let (input, (id, _, container)) = tuple((layout_id, char(','), container))(input)?;
    Ok((input, WindowLayout { id, container }))
}

fn from_hex(input: &str) -> std::result::Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(input, 16)
}

fn layout_id(input: &str) -> IResult<&str, u16> {
    map_res(hex_digit1, from_hex)(input)
}

fn parse_u16(input: &str) -> IResult<&str, u16> {
    map_res(digit1, str::parse)(input)
}

fn dimensions(input: &str) -> IResult<&str, Dimensions> {
    let (input, (width, _, height)) = tuple((parse_u16, char('x'), parse_u16))(input)?;
    Ok((input, Dimensions { width, height }))
}

fn coordinates(input: &str) -> IResult<&str, Coordinates> {
    let (input, (x, _, y)) = tuple((parse_u16, char(','), parse_u16))(input)?;
    Ok((input, Coordinates { x, y }))
}

fn single_pane(input: &str) -> IResult<&str, Element> {
    let (input, (_, pane_id)) = tuple((char(','), parse_u16))(input)?;
    Ok((input, Element::Pane { pane_id }))
}

fn horiz_split(input: &str) -> IResult<&str, Element> {
    let (input, elements) =
        delimited(char('{'), separated_list1(char(','), container), char('}'))(input)?;
    Ok((input, Element::Horizontal(Split { elements })))
}

fn vert_split(input: &str) -> IResult<&str, Element> {
    let (input, elements) =
        delimited(char('['), separated_list1(char(','), container), char(']'))(input)?;
    Ok((input, Element::Vertical(Split { elements })))
}

fn element(input: &str) -> IResult<&str, Element> {
    alt((single_pane, horiz_split, vert_split))(input)
}

fn container(input: &str) -> IResult<&str, Container> {
    let (input, (dimensions, _, coordinates, element)) =
        tuple((dimensions, char(','), coordinates, element))(input)?;
    Ok((
        input,
        Container {
            dimensions,
            coordinates,
            element,
        },
    ))
}

#[cfg(test)]
mod tests {

    use super::{
        coordinates, dimensions, layout_id, single_pane, vert_split, window_layout, Container,
        Coordinates, Dimensions, Element, Split, WindowLayout,
    };

    #[test]
    fn test_parse_layout_id() {
        let input = "9f58";

        let actual = layout_id(input);
        let expected = Ok(("", 40792_u16));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_dimensions() {
        let input = "237x0";

        let actual = dimensions(input);
        let expected = Ok((
            "",
            Dimensions {
                width: 237,
                height: 0,
            },
        ));
        assert_eq!(actual, expected);

        let input = "7x13";

        let actual = dimensions(input);
        let expected = Ok((
            "",
            Dimensions {
                width: 7,
                height: 13,
            },
        ));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_parse_coordinates() {
        let input = "120,0";

        let actual = coordinates(input);
        let expected = Ok(("", Coordinates { x: 120, y: 0 }));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_single_pane() {
        let input = ",46";

        let actual = single_pane(input);
        let expected = Ok(("", Element::Pane { pane_id: 46 }));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_vertical_split() {
        let input = "[279x47,0,0,82,279x23,0,48,83]";

        let actual = vert_split(input);
        let expected = Ok((
            "",
            Element::Vertical(Split {
                elements: vec![
                    Container {
                        dimensions: Dimensions {
                            width: 279,
                            height: 47,
                        },
                        coordinates: Coordinates { x: 0, y: 0 },
                        element: Element::Pane { pane_id: 82 },
                    },
                    Container {
                        dimensions: Dimensions {
                            width: 279,
                            height: 23,
                        },
                        coordinates: Coordinates { x: 0, y: 48 },
                        element: Element::Pane { pane_id: 83 },
                    },
                ],
            }),
        ));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_layout() {
        let input = "41e9,279x71,0,0[279x40,0,0,71,279x30,0,41{147x30,0,41,72,131x30,148,41,73}]";

        let actual = window_layout(input);
        let expected = Ok((
            "",
            WindowLayout {
                id: 0x41e9,
                container: Container {
                    dimensions: Dimensions {
                        width: 279,
                        height: 71,
                    },
                    coordinates: Coordinates { x: 0, y: 0 },
                    element: Element::Vertical(Split {
                        elements: vec![
                            Container {
                                dimensions: Dimensions {
                                    width: 279,
                                    height: 40,
                                },
                                coordinates: Coordinates { x: 0, y: 0 },
                                element: Element::Pane { pane_id: 71 },
                            },
                            Container {
                                dimensions: Dimensions {
                                    width: 279,
                                    height: 30,
                                },
                                coordinates: Coordinates { x: 0, y: 41 },
                                element: Element::Horizontal(Split {
                                    elements: vec![
                                        Container {
                                            dimensions: Dimensions {
                                                width: 147,
                                                height: 30,
                                            },
                                            coordinates: Coordinates { x: 0, y: 41 },
                                            element: Element::Pane { pane_id: 72 },
                                        },
                                        Container {
                                            dimensions: Dimensions {
                                                width: 131,
                                                height: 30,
                                            },
                                            coordinates: Coordinates { x: 148, y: 41 },
                                            element: Element::Pane { pane_id: 73 },
                                        },
                                    ],
                                }),
                            },
                        ],
                    }),
                },
            },
        ));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_pane_ids() {
        let input = "41e9,279x71,0,0[279x40,0,0,71,279x30,0,41{147x30,0,41,72,131x30,148,41,73}]";
        let (_, layout) = window_layout(input).unwrap();

        let actual = layout.pane_ids();
        let expected = vec![71, 72, 73];
        assert_eq!(actual, expected);
    }
}

use std::fmt::Debug;
use nom::{
    bytes::complete::{tag, take_while, take_while_m_n},
    multi::{many0, many1},
    sequence::tuple,
    combinator::map,
    branch::alt,
    character::complete::digit0,
    IResult,
};

#[derive(Debug, Clone)]
struct Parsed {
    molecules: Vec<Molecule>,
}

#[derive(Debug, Clone)]
enum Molecule {
    Group(Vec<Molecule>, u32),
    Single(Element, u32),
}

// lazy
type Element = String;


fn parse_element(input: &str) -> IResult<&str, Element> {
    let (input, (first_letter, rest)) =
        tuple((
            take_while_m_n(1, 1, char::is_uppercase),
            take_while(char::is_lowercase),
        ))(input)?;

    let element = format!("{}{}", first_letter, rest);

    Ok((input, element))
}

fn parse_amount(input: &str) -> IResult<&str, u32> {
    map(
        digit0,
        |amount| match amount {
            "" => 1,
            _  => amount.parse::<u32>().unwrap(),
        }
    )(input)
}

fn parse_molecule(input: &str) -> IResult<&str, Molecule> {
    alt((
        map(
            tuple((
                tag("("),
                many1(parse_molecule),
                tag(")"),
                parse_amount
            )),
            |(_, molec, _, amt)| Molecule::Group(molec, amt),
        ),
        map(
            tuple((
                parse_element,
                parse_amount
            )),
            |(elem, amt)| Molecule::Single(elem, amt)
        ),
    ))(input)
}


fn parse(input: &str) -> IResult<&str, Parsed> {
    map(
        many0(parse_molecule),
        |molecules| Parsed { molecules },
    )(input)
}


fn main() {
    println!("AST Style:");
    println!("{:?}", parse("NH2(NO3)2"));
    println!("{:?}", parse("Al(H2O)12(OH)4"));
    println!("{:?}", parse("(OH)4"));
    println!("{:?}", parse("OH"));
    println!();
    println!("Flat Style:");
    println!("{:?}", parse("NH2(NO3)2").flatten());
    println!("{:?}", parse("Al(H2O)12(OH)4").flatten());
    println!("{:?}", parse("(OH)4").flatten());
    println!("{:?}", parse("OH").flatten());
}


// Garbage below here...


trait Flatten {
    fn flatten(&self) -> Vec<(Element, u32)>;
}

impl<E> Flatten for Result<(&str, Parsed), E>
where
    E: Debug
{
    fn flatten(&self) -> Vec<(Element, u32)> {
        self.as_ref()
            .map(|(_, p)| p)
            .unwrap()
            .flatten()
    }
}

impl Flatten for Parsed {
    fn flatten(&self) -> Vec<(Element, u32)> {
        // stuff everything into one big molecule, then flatten
        Molecule::Group(self.molecules.clone(), 1)
            .flatten()
    }
}

impl Flatten for Molecule {
    fn flatten(&self) -> Vec<(Element, u32)> {
        let mut flat = vec![];

        match self {
            // already flat
            Molecule::Single(elem, amt) => {
                flat.accumulate(elem.to_string(), *amt);
            },
            Molecule::Group(molecules, grp_amt) => {
                for molecule in molecules {
                    let elems = molecule.flatten();

                    for (elem, elem_amt) in elems {
                        flat.accumulate(elem.to_string(), elem_amt * *grp_amt);
                    }
                }
            }
        }

        flat
    }
}

trait Accumulate<T, N> {
    fn accumulate(&mut self, item: T, amount: N);
}

impl<T> Accumulate<T, u32> for Vec<(T, u32)>
where
    T: PartialEq
{
    fn accumulate(&mut self, item: T, amount: u32) {
        if let Some(elem) = self.iter_mut().find(|(elem, _)| *elem == item) {
            elem.1 += amount;
        } else {
            self.push((item, amount));
        }
    }
}


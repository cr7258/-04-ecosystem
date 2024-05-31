use std::string::ToString;
use strum::{Display, EnumCount, EnumIs, EnumIter, IntoEnumIterator, VariantNames};

#[allow(unused)]
#[derive(Display, EnumIs, EnumIter, VariantNames, Debug)]
enum Color {
    #[strum(serialize = "redred")]
    Red,
    Green {
        range: usize,
    },
    Blue(usize),
    Yellow,
    #[strum(to_string = "purple with {sat} saturation")]
    Purple {
        sat: usize,
    },
}

#[derive(Debug, EnumCount, EnumIter)]
enum Week {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

fn main() {
    // Display
    // uses the serialize string for Display
    let red = Color::Red;
    assert_eq!(String::from("redred"), format!("{}", red));
    // by default the variants Name
    let yellow = Color::Yellow;
    assert_eq!(String::from("Yellow"), yellow.to_string());
    // or for string formatting
    println!(
        "blue: {}, green: {}",
        Color::Blue(10),
        Color::Green { range: 42 }
    );
    // you can also use named fields in message
    let purple = Color::Purple { sat: 10 };
    assert_eq!(
        String::from("purple with 10 saturation"),
        purple.to_string()
    );

    // EnumCount
    assert_eq!(7, Week::COUNT);
    assert_eq!(Week::iter().count(), Week::COUNT);

    // EnumIs
    assert!(Color::Red.is_red());
    assert!(Color::Green { range: 42 }.is_green());

    // VariantNames
    println!("{:?}", Color::VARIANTS);

    // EnumIter
    for color in Color::iter() {
        println!("My favorite color is {:?}", color);
    }
}

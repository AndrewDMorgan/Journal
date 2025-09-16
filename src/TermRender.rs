// snake case is just bad
#![allow(dead_code, non_snake_case)]

use std::io::Write;


// static color/mod pairs for default ascii/ansi codes
// colorCode (if any), mods, background (bool)   when called if background then add that color as background col
//      if no background found, provide no such parameter
// /033[ is the base with the ending post-fix being
// start;color;mod;mod;mod...suffix   how do I do different colored mods? Do I add another attachment? <- correct
// https://notes.burke.libbey.me/ansi-escape-codes/
// https://stackoverflow.com/questions/4842424/list-of-ansi-color-escape-sequences
// /033[... doesn't work; use /x1b[...
pub static CLEAR: &str = "\x1b[0m";
pub static SHOW_CURSOR: &str = "\x1b[?25h";
pub static HIDE_CURSOR: &str = "\x1b[?25l";

// * color, modifiers, is_background
pub static EMPTY_MODIFIER_REFERENCE: &[&str] = &[];  // making a default static type is annoying

pub static BLACK:      (Option <&str>, &[&str], bool) = (Some("30"), &[], false);
pub static RED:        (Option <&str>, &[&str], bool) = (Some("31"), &[], false);
pub static GREEN:      (Option <&str>, &[&str], bool) = (Some("32"), &[], false);
pub static YELLOW:     (Option <&str>, &[&str], bool) = (Some("33"), &[], false);
pub static BLUE:       (Option <&str>, &[&str], bool) = (Some("34"), &[], false);
pub static MAGENTA:    (Option <&str>, &[&str], bool) = (Some("35"), &[], false);
pub static CYAN:       (Option <&str>, &[&str], bool) = (Some("36"), &[], false);
pub static WHITE:      (Option <&str>, &[&str], bool) = (Some("37"), &[], false);
pub static DEFAULT:    (Option <&str>, &[&str], bool) = (Some("39"), &[], false);

pub static BRIGHT_BLACK:   (Option <&str>, &[&str], bool) = (Some("90"), &[], false );
pub static BRIGHT_RED:     (Option <&str>, &[&str], bool) = (Some("91"), &[], false );
pub static BRIGHT_GREEN:   (Option <&str>, &[&str], bool) = (Some("92"), &[], false );
pub static BRIGHT_YELLOW:  (Option <&str>, &[&str], bool) = (Some("93"), &[], false );
pub static BRIGHT_BLUE:    (Option <&str>, &[&str], bool) = (Some("94"), &[], false );
pub static BRIGHT_MAGENTA: (Option <&str>, &[&str], bool) = (Some("95"), &[], false );
pub static BRIGHT_CYAN:    (Option <&str>, &[&str], bool) = (Some("96"), &[], false );
pub static BRIGHT_WHITE:   (Option <&str>, &[&str], bool) = (Some("97"), &[], false );
pub static BRIGHT_DEFAULT: (Option <&str>, &[&str], bool) = (Some("99"), &[], false );

pub static ON_BLACK:   (Option <&str>, &[&str], bool) = (Some("100"), &[], true );
pub static ON_RED:     (Option <&str>, &[&str], bool) = (Some("101"), &[], true );
pub static ON_GREEN:   (Option <&str>, &[&str], bool) = (Some("102"), &[], true );
pub static ON_YELLOW:  (Option <&str>, &[&str], bool) = (Some("103"), &[], true );
pub static ON_BLUE:    (Option <&str>, &[&str], bool) = (Some("104"), &[], true );
pub static ON_MAGENTA: (Option <&str>, &[&str], bool) = (Some("105"), &[], true );
pub static ON_CYAN:    (Option <&str>, &[&str], bool) = (Some("106"), &[], true );
pub static ON_WHITE:   (Option <&str>, &[&str], bool) = (Some("107"), &[], true );
pub static ON_DEFAULT: (Option <&str>, &[&str], bool) = (Some("109"), &[], true );

pub static ON_BRIGHT_BLACK:   (Option <&str>, &[&str], bool) = (Some("40"), &[], true );
pub static ON_BRIGHT_RED:     (Option <&str>, &[&str], bool) = (Some("41"), &[], true );
pub static ON_BRIGHT_GREEN:   (Option <&str>, &[&str], bool) = (Some("42"), &[], true );
pub static ON_BRIGHT_YELLOW:  (Option <&str>, &[&str], bool) = (Some("43"), &[], true );
pub static ON_BRIGHT_BLUE:    (Option <&str>, &[&str], bool) = (Some("44"), &[], true );
pub static ON_BRIGHT_MAGENTA: (Option <&str>, &[&str], bool) = (Some("45"), &[], true );
pub static ON_BRIGHT_CYAN:    (Option <&str>, &[&str], bool) = (Some("46"), &[], true );
pub static ON_BRIGHT_WHITE:   (Option <&str>, &[&str], bool) = (Some("47"), &[], true );
pub static ON_BRIGHT_DEFAULT: (Option <&str>, &[&str], bool) = (Some("49"), &[], true );

pub static BOLD:      (Option <&str>, &[&str], bool) = (None    , &["1"], false);
pub static DIM:       (Option <&str>, &[&str], bool) = (None    , &["2"], false);
pub static ITALIC:    (Option <&str>, &[&str], bool) = (None    , &["3"], false);
pub static UNDERLINE: (Option <&str>, &[&str], bool) = (None    , &["4"], false);
pub static BLINK:     (Option <&str>, &[&str], bool) = (None    , &["5"], false);
pub static REVERSE:   (Option <&str>, &[&str], bool) = (None    , &["7"], false);
pub static HIDE:      (Option <&str>, &[&str], bool) = (None    , &["8"], false);


// manages the global state for light/dark modes (handles basic colors switching around)
// no support for RGB/custom color codes, only the default variants
#[derive(Clone, PartialEq, Eq, Debug, Default, Hash, Copy)]
pub enum ColorMode {
    #[default] Dark,
    Light,
}

impl ColorMode {
    pub fn ToLight () {
        unsafe {COLOR_MODE = ColorMode::Light};
    }

    pub fn ToDark () {
        unsafe {COLOR_MODE = ColorMode::Dark};
    }
}

// hopefully this will let full usage of colors while not worrying too much about light/dark mode
// -- (basic but limited automatic support; not everything will look perfect by default)
static mut COLOR_MODE: ColorMode = ColorMode::Dark;


#[derive(Clone, Debug, Eq, PartialEq, Default, Hash, Copy)]
// Different base ascii text modifiers (static constants)
pub enum ColorType {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    #[default] Default,

    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    BrightDefault,

    OnBlack,
    OnRed,
    OnGreen,
    OnYellow,
    OnBlue,
    OnMagenta,
    OnCyan,
    OnWhite,
    OnDefault,

    OnBrightBlack,
    OnBrightRed,
    OnBrightGreen,
    OnBrightYellow,
    OnBrightBlue,
    OnBrightMagenta,
    OnBrightCyan,
    OnBrightWhite,
    OnBrightDefault,

    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Reverse,
    Hide,

    OnRGB (u8, u8, u8),
    Rgb(u8, u8, u8),
    OnANSI (u8),
    Ansi(u8),
}

// Stores a unique color type.
// The unique types are either a fully static color
// or a partially dynamic type.
// This allows for a passing of different types,
// circumventing lifetime issues while preserving statics.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum UniqueColor {
    Static  ((Option <&'static str>, &'static [&'static str], bool)),
    Dynamic ((Option <   String   >, &'static [&'static str], bool)),
}

impl UniqueColor {
    // Converts a static slice into a vector of type String
    fn IntoStringVec (&self, attributes: &'static [&'static str]) -> Vec<String> {
        let mut mods = vec![];
        for modifier in attributes {
            mods.push(modifier.to_string())
        }
        mods
    }

    // Converts the unique color into a standardized tuple form.
    // In other words, converts the dynamic and static versions
    // into a single unified version for easier handling
    pub fn UnwrapIntoTuple (&self) -> (Option <String>, Vec <String>, bool) {
        match self {
            UniqueColor::Static(s) => {
                (s.0.map(|t| t.to_owned()), self.IntoStringVec(s.1), s.2)
            },
            UniqueColor::Dynamic(s) => {
                (s.0.clone(), self.IntoStringVec(s.1), s.2)
            },
        }
    }
}

impl ColorType {
    // Converts the color type into a unique color (static or dynamic)
    pub fn GetColor (&self) -> UniqueColor {
        if unsafe { COLOR_MODE } == ColorMode::Dark {
            self.GetDarkColor()
        } else {
            self.GetLightColor()
        }
    }

    fn GetLightColor (&self) -> UniqueColor {
        match self {
            ColorType::Black =>   { UniqueColor::Static(BRIGHT_WHITE) },
            ColorType::Red =>     { UniqueColor::Static(RED) },
            ColorType::Green =>   { UniqueColor::Static(GREEN) },
            ColorType::Yellow =>  { UniqueColor::Static(YELLOW) },
            ColorType::Blue =>    { UniqueColor::Static(BLUE) },
            ColorType::Magenta => { UniqueColor::Static(MAGENTA) },
            ColorType::Cyan =>    { UniqueColor::Static(CYAN) },
            ColorType::White =>   { UniqueColor::Static(BRIGHT_BLACK) },
            ColorType::Default => { UniqueColor::Static(BRIGHT_DEFAULT) },

            ColorType::BrightBlack =>   { UniqueColor::Static(WHITE) },
            ColorType::BrightRed =>     { UniqueColor::Static(RED) },
            ColorType::BrightGreen =>   { UniqueColor::Static(GREEN) },
            ColorType::BrightYellow =>  { UniqueColor::Static(YELLOW) },
            ColorType::BrightBlue =>    { UniqueColor::Static(BLUE) },
            ColorType::BrightMagenta => { UniqueColor::Static(MAGENTA) },
            ColorType::BrightCyan =>    { UniqueColor::Static(CYAN) },
            ColorType::BrightWhite =>   { UniqueColor::Static(BLACK) },
            ColorType::BrightDefault => { UniqueColor::Static(DEFAULT) },

            ColorType::OnBlack => { UniqueColor::Static(ON_WHITE) },
            ColorType::OnRed => { UniqueColor::Static(ON_RED) },
            ColorType::OnGreen => { UniqueColor::Static(ON_GREEN) },
            ColorType::OnYellow => { UniqueColor::Static(ON_YELLOW) },
            ColorType::OnBlue => { UniqueColor::Static(ON_BLUE) },
            ColorType::OnMagenta => { UniqueColor::Static(ON_MAGENTA) },
            ColorType::OnCyan => { UniqueColor::Static(ON_CYAN) },
            ColorType::OnWhite => { UniqueColor::Static(ON_BRIGHT_BLACK) },
            ColorType::OnDefault => { UniqueColor::Static(ON_BRIGHT_DEFAULT) },

            ColorType::OnBrightBlack => { UniqueColor::Static(ON_BRIGHT_WHITE) },
            ColorType::OnBrightRed => { UniqueColor::Static(ON_RED) },
            ColorType::OnBrightGreen => { UniqueColor::Static(ON_GREEN) },
            ColorType::OnBrightYellow => { UniqueColor::Static(ON_YELLOW) },
            ColorType::OnBrightBlue => { UniqueColor::Static(ON_BLUE) },
            ColorType::OnBrightMagenta => { UniqueColor::Static(ON_MAGENTA) },
            ColorType::OnBrightCyan => { UniqueColor::Static(ON_CYAN) },
            ColorType::OnBrightWhite => { UniqueColor::Static(ON_BLACK) },
            ColorType::OnBrightDefault => { UniqueColor::Static(ON_DEFAULT) },

            // 24-bit? I think so but make sure it works
            ColorType::Rgb(r, g, b) => {
                let (mut rn, mut gn, mut bn) = (*r, *g, *b);
                if rn > 128 {  rn -= 128;  }
                if gn > 128 {  gn -= 128;  }
                if bn > 128 {  bn -= 128;  }
                UniqueColor::Dynamic((Some(format!("38;2;{};{};{}", rn, gn, bn)), EMPTY_MODIFIER_REFERENCE, false))
            },
            // background 24-bit? Make sure that's right
            ColorType::OnRGB (r, g, b) => {
                let (mut rn, mut gn, mut bn) = (*r, *g, *b);
                if rn > 128 {  rn -= 128;  }
                if gn > 128 {  gn -= 128;  }
                if bn > 128 {  bn -= 128;  }
                UniqueColor::Dynamic((Some(format!("48;2;{};{};{}", rn, gn, bn)), EMPTY_MODIFIER_REFERENCE, true))
            },
            ColorType::Ansi(index) => {
                UniqueColor::Dynamic((Some(format!("38;5;{}", index)), EMPTY_MODIFIER_REFERENCE, false))
            },
            ColorType::OnANSI (index) => {
                UniqueColor::Dynamic((Some(format!("48;5;{}", index)), EMPTY_MODIFIER_REFERENCE, true))
            },

            ColorType::Bold => { UniqueColor::Static(BOLD) },
            ColorType::Dim => { UniqueColor::Static(DIM) },
            ColorType::Italic => { UniqueColor::Static(ITALIC) },
            ColorType::Underline => { UniqueColor::Static(UNDERLINE) },
            ColorType::Blink => { UniqueColor::Static(BLINK) },
            ColorType::Reverse => { UniqueColor::Static(REVERSE) },
            ColorType::Hide => { UniqueColor::Static(HIDE) },
        }
    }

    fn GetDarkColor (&self) -> UniqueColor {
        match self {
            ColorType::Black => { UniqueColor::Static(BLACK) },
            ColorType::Red => { UniqueColor::Static(RED) },
            ColorType::Green => { UniqueColor::Static(GREEN) },
            ColorType::Yellow => { UniqueColor::Static(YELLOW) },
            ColorType::Blue => { UniqueColor::Static(BLUE) },
            ColorType::Magenta => { UniqueColor::Static(MAGENTA) },
            ColorType::Cyan => { UniqueColor::Static(CYAN) },
            ColorType::White => { UniqueColor::Static(WHITE) },
            ColorType::Default => { UniqueColor::Static(DEFAULT) },

            ColorType::BrightBlack => { UniqueColor::Static(BRIGHT_BLACK) },
            ColorType::BrightRed => { UniqueColor::Static(BRIGHT_RED) },
            ColorType::BrightGreen => { UniqueColor::Static(BRIGHT_GREEN) },
            ColorType::BrightYellow => { UniqueColor::Static(BRIGHT_YELLOW) },
            ColorType::BrightBlue => { UniqueColor::Static(BRIGHT_BLUE) },
            ColorType::BrightMagenta => { UniqueColor::Static(BRIGHT_MAGENTA) },
            ColorType::BrightCyan => { UniqueColor::Static(BRIGHT_CYAN) },
            ColorType::BrightWhite => { UniqueColor::Static(BRIGHT_WHITE) },
            ColorType::BrightDefault => { UniqueColor::Static(BRIGHT_DEFAULT) },

            ColorType::OnBlack => { UniqueColor::Static(ON_BRIGHT_BLACK) },
            ColorType::OnRed => { UniqueColor::Static(ON_BRIGHT_RED) },
            ColorType::OnGreen => { UniqueColor::Static(ON_BRIGHT_GREEN) },
            ColorType::OnYellow => { UniqueColor::Static(ON_BRIGHT_YELLOW) },
            ColorType::OnBlue => { UniqueColor::Static(ON_BRIGHT_BLUE) },
            ColorType::OnMagenta => { UniqueColor::Static(ON_BRIGHT_MAGENTA) },
            ColorType::OnCyan => { UniqueColor::Static(ON_BRIGHT_CYAN) },
            ColorType::OnWhite => { UniqueColor::Static(ON_BRIGHT_WHITE) },
            ColorType::OnDefault => { UniqueColor::Static(ON_DEFAULT) },

            ColorType::OnBrightBlack => { UniqueColor::Static(ON_BLACK) },
            ColorType::OnBrightRed => { UniqueColor::Static(ON_RED) },
            ColorType::OnBrightGreen => { UniqueColor::Static(ON_GREEN) },
            ColorType::OnBrightYellow => { UniqueColor::Static(ON_YELLOW) },
            ColorType::OnBrightBlue => { UniqueColor::Static(ON_BLUE) },
            ColorType::OnBrightMagenta => { UniqueColor::Static(ON_MAGENTA) },
            ColorType::OnBrightCyan => { UniqueColor::Static(ON_CYAN) },
            ColorType::OnBrightWhite => { UniqueColor::Static(ON_WHITE) },
            ColorType::OnBrightDefault => { UniqueColor::Static(ON_BRIGHT_DEFAULT) },

            // 24-bit? I think so but make sure it works
            ColorType::Rgb(r, g, b) => {
                UniqueColor::Dynamic((Some(format!("38;2;{};{};{}", r, g, b)), EMPTY_MODIFIER_REFERENCE, false))
            },
            // background 24-bit? Make sure that's right
            ColorType::OnRGB (r, g, b) => {
                UniqueColor::Dynamic((Some(format!("48;2;{};{};{}", r, g, b)), EMPTY_MODIFIER_REFERENCE, true))
            },
            ColorType::Ansi(index) => {
                UniqueColor::Dynamic((Some(format!("38;5;{}", index)), EMPTY_MODIFIER_REFERENCE, false))
            },
            ColorType::OnANSI (index) => {
                UniqueColor::Dynamic((Some(format!("48;5;{}", index)), EMPTY_MODIFIER_REFERENCE, true))
            },

            ColorType::Bold => { UniqueColor::Static(BOLD) },
            ColorType::Dim => { UniqueColor::Static(DIM) },
            ColorType::Italic => { UniqueColor::Static(ITALIC) },
            ColorType::Underline => { UniqueColor::Static(UNDERLINE) },
            ColorType::Blink => { UniqueColor::Static(BLINK) },
            ColorType::Reverse => { UniqueColor::Static(REVERSE) },
            ColorType::Hide => { UniqueColor::Static(HIDE) },
        }
    }
}

// Color setters for standard primitives

/// Converts the given instance into the type Colored based on a provided
/// set of modifiers (in the form of ColorType).
pub trait Colorize {
    // adds a set of modifiers/colors
    fn Colorizes (&self, colors: Vec <ColorType>) -> Colored;

    // adds a single modifier/color
    fn Colorize (&self, colors: ColorType) -> Colored;
}

impl Colorize for &str {
    fn Colorizes (&self, colors: Vec <ColorType>) -> Colored {
        Colored::GetFromColorTypesStr(self, colors)
    }

    fn Colorize (&self, color: ColorType) -> Colored {
        Colored::GetFromColorTypesStr(self, vec![color])
    }
}

impl Colorize for String {
    fn Colorizes (&self, colors: Vec <ColorType>) -> Colored {
        Colored::GetFromColorTypesStr(self.as_str(), colors)
    }

    fn Colorize (&self, color: ColorType) -> Colored {
        Colored::GetFromColorTypesStr(self.as_str(), vec![color])
    }
}


// A colored string
// It stores all of its modifiers like colors/underlying/other
//#[derive(Clone)]
#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct Colored {
    text: String,
    mods: Vec <String>,
    color: Option <String>,
    bgColor: Option <String>,
}

impl Colorize for Colored {
    fn Colorizes (&self, colors: Vec <ColorType>) -> Colored {
        let mut mods = vec![];
        for modifier in colors {
            mods.push(modifier);
        }
        Colored::GetFromColorTypes(self, mods)
    }

    fn Colorize (&self, color: ColorType) -> Colored {
        Colored::GetFromColorTypes(self, vec![color])
    }
}

impl Colored {
    pub fn new (text: String) -> Colored {
        Colored {
            text,
            mods: vec![],
            color: None,
            bgColor: None,
        }
    }

    /// returns the left and right halves as unique Colored instances. Keeps the original instance untouched.
    pub fn Split (&self, midPoint: usize) -> (Colored, Colored) {
        (
            Colored {
                text: self.text[..midPoint].to_string(),
                mods: self.mods.clone(),
                color: self.color.clone(),
                bgColor: self.bgColor.clone(),
            },
            Colored {
                text: self.text[midPoint..].to_string(),
                mods: self.mods.clone(),
                color: self.color.clone(),
                bgColor: self.bgColor.clone(),
            }
        )
    }

    pub fn IsUncolored (&self) -> bool {
        self.mods.is_empty() && self.color.is_none() && self.bgColor.is_none()
    }

    pub fn Contains (&self, color: &ColorType) -> bool {
        let col = color.GetColor().UnwrapIntoTuple();
        if col.0 == self.bgColor && col.2 {  return true;  }
        if let Some(selfColor) = &self.color {
            if let Some(otherCol) = col.0 {
                if selfColor.contains(&otherCol) {  return true;  }
            }
        }
        for modifier in col.1 {
            if self.mods.contains(&modifier) {  return true;  }
        } false
    }

    pub fn ChangeText (&mut self, text: String) {
        self.text = text;
    }

    // Adds a color type
    pub fn AddColor (&mut self, color: ColorType) {
        self.AddUnique(color.GetColor());
    }

    // Adds a unique color
    pub fn AddUnique (&mut self, uniqueColor: UniqueColor) {
        let (color, mods, background) = uniqueColor.UnwrapIntoTuple();
        if background {  self.bgColor = color;  }
        else if let Some(col) = color {
            // making sure to not overwrite the existing color if this is None
            self.color = Some(col);
        }
        for modifier in mods {
            self.mods.push(modifier);
        }
    }

    // Takes a set of color types and returns a filled out Colored instance
    pub fn GetFromColorTypes (colored: &Colored, colors: Vec <ColorType>) -> Colored {
        let mut colored = Colored {
            text: colored.text.clone(),
            mods: colored.mods.clone(),
            color: colored.color.clone(),
            bgColor: colored.bgColor.clone(),
        };
        for color in colors {
            colored.AddColor(color);
        } colored
    }

    // Takes a set of color types and returns a filled out Colored instance
    pub fn GetFromColorTypesStr (text: &str, colors: Vec <ColorType>) -> Colored {
        let mut colored = Colored::new(text.to_owned());
        for color in colors {
            colored.AddColor(color);
        } colored
    }

    // Takes a set of unique colors and generates a filled out instance
    pub fn GetFromUniqueColors (text: String, uniqueColors: Vec <UniqueColor>) -> Colored {
        let mut colored = Colored::new(text);
        for color in uniqueColors {
            colored.AddUnique(color);
        } colored
    }

    pub fn GetText (&self, lastColor: &mut String) -> (String, usize) {
        let mut text = String::new();

        let col = match &self.color {
            Some(colr) => colr,
            _ => &String::new()
        };

        let bgCol = match &self.bgColor {
            Some(colr) => colr,
            _ => &String::new()
        };

        let color = match
            (self.bgColor.is_some(), self.color.is_some(), !self.mods.is_empty())
        {
            (true, true, true) => format!("\x1b[0;{};{};{}m", col, bgCol, self.mods.join(";")),
            (true, true, false) => format!("\x1b[0;{};{}m", col, bgCol),
            (false, true, true) => format!("\x1b[0;{};{}m", col, self.mods.join(";")),
            (false, true, false) => format!("\x1b[0;{}m", col),
            (true, false, true) => format!("\x1b[0;{};{}m", bgCol, self.mods.join(";")),
            (true, false, false) => format!("\x1b[0;{}m", bgCol),
            (false, false, _) => String::from("\x1b[0m"),
        };

        if color != *lastColor {
            //text.push_str(CLEAR);
            text.push_str(&color);
            *lastColor = color;
        }

        text.push_str(&self.text);
        (text, self.text.chars().count())
    }

    pub fn GetSize (&self) -> usize {
        self.text.chars().count()
    }
}

// A colored span of text (fancy string)
#[derive(Clone, Debug, Default, Hash, Eq, PartialEq)]
pub struct Span {
    line: Vec <Colored>,
}

impl Span {
    pub fn FromTokens (tokens: Vec <Colored>) -> Self {
        Span {
            line: tokens,
        }
    }

    pub fn Size (&self) -> usize {
        let mut size = 0;
        for colored in &self.line {
            size += colored.GetSize();
        }
        size
    }

    pub fn Join (&self) -> (String, usize) {
        //let mut lastColored = vec![];
        let mut lastColored = String::new();
        let mut total = String::new();
        let mut totalSize = 0;
        for colored in &self.line {
            let (text, size) = colored.GetText(&mut lastColored);
            total.push_str(&text);
            totalSize += size;
        }
        (total, totalSize)
    }
}


// Similar to a paragraph in Ratatui
// Windows are a block or section within the terminal space
// Multiple windows can be rendered at once
// Each window can contain its own text or logic
// This allows a separation/abstraction for individual sections
// This also allows for a cached window to be reused if temporarily closed
#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct Window {
    pub position: (u16, u16),
    pub depth: u16,
    pub size: (u16, u16),
    updated: Vec <bool>,
    wasUpdated: bool,

    // (Span, cached render, num visible chars)
    lines: Vec <(Span, String, usize)>,

    bordered: bool,
    title: (Span, usize),
    color: Colored,
    pub hidden: bool,
}

type RenderClosure = Vec <(Box <dyn FnOnce () -> String + Send>, u16, u16, u16)>;

impl Window {
    pub fn new (position: (u16, u16), depth: u16, size: (u16, u16)) -> Self {
        Window {
            position,
            depth,
            size,
            updated: vec![false; size.1 as usize],
            wasUpdated: false,
            lines: vec![],
            bordered: false,
            title: (Span::default(), 0),
            color: Colored::new(String::new()),  // format!("\x1b[38;2;{};{};{}m", 125, 125, 0),//String::new(),
            hidden: false,
        }
    }

    pub fn Hide (&mut self) -> bool {
        if self.hidden {  return false;  }
        self.hidden = true;
        self.UpdateAll();
        true
    }

    pub fn Show(&mut self) -> bool {
        if !self.hidden {  return false;  }
        self.hidden = false;
        self.UpdateAll();
        true
    }

    pub fn Move (&mut self, newPosition: (u16, u16)) {
        if newPosition == self.position {  return;  }
        self.position = newPosition;
        self.UpdateAll();
    }

    pub fn Colorizes (&mut self, colors: Vec <ColorType>) {
        for color in colors {
            self.color.AddColor(color);
        } self.UpdateAll();
    }

    pub fn Colorize (&mut self, color: ColorType) {
        self.color.AddColor(color);
        self.UpdateAll();
    }

    pub fn TryColorize (&mut self, color: ColorType) -> bool {
        if self.color.Contains(&color) {  return false;  }
        self.color.AddColor(color);
        self.UpdateAll();
        true
    }

    pub fn ClearColors (&mut self) -> bool {
        if self.color.IsUncolored() { return false; }
        self.color = Colored::new(String::new());
        self.UpdateAll();
        true
    }

    // Adds a border around the window/block
    pub fn Bordered (&mut self) {
        self.bordered = true;
    }

    // Sets/updates the title of the window/block
    pub fn Titled (&mut self, title: String) {
        self.title = (
            Span::FromTokens(
            vec![title.Colorizes(vec![])]),
            title.chars().count()
        );
        self.wasUpdated = false;
        self.updated[0] = false;
        //self.color.ChangeText(title);
    }

    pub fn HasTitle (&self) -> bool {
        self.title.1 != 0
    }

    pub fn TitledColored (&mut self, title: Span) {
        let size = title.Size();
        self.title = (title, size);
        self.wasUpdated = false;
        self.updated[0] = false;
    }

    pub fn Resize (&mut self, changed: (u16, u16)) -> bool {
        if self.size == changed {  return false;  }
        self.size = (
            std::cmp::max(changed.0, 0),
            std::cmp::max(changed.1, 0)
        );
        self.updated = vec![false; self.size.1 as usize];
        self.UpdateAll();
        true
    }
    
    // Clamps a string to a maximum length of visible UTF-8 characters while preserving escape codes
    fn ClampStringVisibleUTF_8 (text: &str, maxLength: usize) -> String {
        let mut accumulative: String = String::new();

        let mut visible = 0;
        let mut inEscape = false;
        for chr in text.chars() {
            if chr == '\x1b' {
                inEscape = true;
            } else if inEscape {
                if chr == 'm' {
                    inEscape = false;
                }
            } else {
                visible += 1;
                if visible > maxLength {  break;  }
            }
            accumulative.push(chr);
        }

        accumulative
    }

    pub fn RenderWindowSlice (color: (String, usize),
                              bordered: bool,
                              renderText: (String, usize),
                              size: (u16, u16)
    ) -> String {
        let mut text = String::new();

        //let line = &self.lines[index - 1];//self.lines[0..self.size.1 as usize - borderSize][0];
        let borderSize = match bordered {
            true => 2, false => 0
        };
        let lineText = Window::ClampStringVisibleUTF_8(
            &renderText.0, size.0 as usize - borderSize
        );
        let lineSize = std::cmp::min(renderText.1, size.0 as usize - borderSize);

        // handling the side borders
        if bordered {
            text.push_str(&color.0);
            text.push('│');
            text.push_str(CLEAR);
            text.push_str(&lineText);
            text.push_str(CLEAR);
            let padding = (size.0 as usize - 2) - lineSize;
            text.push_str(&" ".repeat(padding));
            text.push_str(&color.0);
            text.push('│');
            text.push_str(CLEAR);
        } else {
            text.push_str(&lineText);
            text.push_str(CLEAR);  // making sure the following are blank
            let padding = (size.0 as usize) - lineSize;
            text.push_str(&" ".repeat(padding));
        } text
    }

    fn HandleHiddenClosure (&mut self, mut renderClosures: RenderClosure) -> RenderClosure {
        self.wasUpdated = true;
        for i in 0..self.updated.len() {
            if self.updated[i] {  continue;  }
            self.updated[i] = true;
            let width = self.size.0;
            renderClosures.push((Box::new(move || {
                " ".repeat(width as usize)
            }), self.position.0, self.position.1 + i as u16, 0));  // the depth is 0, right?
        }
        renderClosures
    }

    pub fn GetRenderClosure (&mut self) -> RenderClosure {
        if self.wasUpdated {  return vec![];  }  // no re-rendering is needed

        let mut renderClosures: RenderClosure = vec![];
        if self.hidden {
            return self.HandleHiddenClosure(renderClosures);
        }

        // these will need to be sorted by row, and the cursor movement is handled externally (the u16 pair)
        let borderColor = self.color.GetText(&mut String::new());
        self.wasUpdated = true;

        // make sure to not call UpdateRender when using closures
        let borderedSize = {
            if self.bordered {  1  }
            else {  0  }
        };
        let mut updated = false;
        for index in borderedSize..self.size.1 as usize - borderedSize {
            if self.updated[index] {  continue;  }
            self.updated[index] = true;
            updated = true;

            let (text, size);
            if index - borderedSize < self.lines.len() {
                (text, size) = self.lines[index - borderedSize].0.Join();
                self.lines[index - borderedSize].1 = text.clone();
                self.lines[index - borderedSize].2 = size;
            } else {
                (text, size) = (String::new(), 0);
            }

            // creating the closure
            let color = borderColor.clone();
            let windowSize = self.size;  // idk a better way to do this other than cloning
            let bordered = self.bordered;

            let closure = move || {
                Window::RenderWindowSlice(color, bordered, (text, size), windowSize)
            };
            renderClosures.push((Box::new(closure), self.position.0, self.position.1 + index as u16, self.depth + 1));
        }

        if updated && self.bordered {
            self.updated[self.size.1 as usize - 1] = true;
            self.updated[0] = true;

            // adding the top and bottom lines to the closures
            let color = borderColor.clone();
            let windowSize = self.size.0;  // idk a better way to do this other than cloning
            let closure = move || {  // top
                let mut text = String::new();
                text.push_str(&color.0);
                text.push('└');
                text.push_str(&"─".repeat(windowSize as usize - 2));
                text.push('┘');
                text.push_str(CLEAR);
                text
            };
            renderClosures.push((Box::new(closure), self.position.0, self.position.1 + self.size.1 - 1, self.depth + 1));

            // bottom
            let color = borderColor;  // consuming border color here
            let windowSize = self.size.0;  // idk a better way to do this other than cloning
            let title = self.title.clone();
            let closure = move || {
                let mut text = String::new();
                text.push_str(&color.0);
                text.push('┌');
                let half = windowSize / 2 - title.1 as u16 / 2 - 1;
                text.push_str(&"─".repeat(half as usize));
                text.push_str(CLEAR);
                text.push_str(&title.0.Join().0);
                text.push_str(&color.0);
                text.push_str(&"─".repeat(windowSize as usize - 2 - half as usize - title.1));
                text.push('┐');
                text.push_str(CLEAR);
                text
            };
            renderClosures.push((Box::new(closure), self.position.0, self.position.1, self.depth + 1));
        }

        renderClosures
    }

    // Gets the rendered text for the individual window
    // This shouldn't crash when rendering out of bounds unlike certain other libraries...
    pub fn GetRender (&self) -> Vec <String> {
        let mut text = vec![String::new()];
        let color = self.color.GetText(&mut String::new());

        // handling the top border
        let borderSize =
            if self.bordered {
                let mut lineSize = 1;
                text[0].push_str(&color.0);
                text[0].push('┌');
                let splitSize = (self.size.0 - 2) / 2 - self.title.1 as u16 / 2;
                lineSize += splitSize;
                text[0].push_str(&"─".repeat(splitSize as usize));
                lineSize += self.title.1 as u16;
                text[0].push_str(&self.title.0.Join().0);
                //let lineSize = text[0].len();
                text[0].push_str(&"─".repeat(
                    (self.size.0 as usize).saturating_sub(1 + lineSize as usize)
                ));
                text[0].push('┐');
                text[0].push_str(CLEAR);
                //text[0].push('\n');  // fix this
                text.push(String::new());
                2
            }
            else {  0  };
        let bordered = borderSize / 2;
        for index in bordered..self.size.1 as usize - bordered {
            let lineText;
            let lineSize;
            if index <= self.lines.len() {
                let line = &self.lines[index - 1];//self.lines[0..self.size.1 as usize - borderSize][0];
                lineText = Window::ClampStringVisibleUTF_8(
                    &line.1, self.size.0 as usize - borderSize
                );
                lineSize = std::cmp::min(self.lines[index - 1].2, self.size.0 as usize - borderSize);
            } else {
                lineText = String::new();
                lineSize = 0;
            }

            // handling the side borders
            if self.bordered {
                text[index].push_str(&color.0);
                text[index].push('│');
                text[index].push_str(CLEAR);
                text[index].push_str(&lineText);
                text[index].push_str(CLEAR);
                let padding = (self.size.0 as usize - 2) - lineSize;
                text[index].push_str(&" ".repeat(padding));
                text[index].push_str(&color.0);
                text[index].push('│');
                text[index].push_str(CLEAR);
            } else {
                text[index].push_str(&lineText);
                let padding = (self.size.0 as usize) - lineSize;
                text[index].push_str(&" ".repeat(padding));
            }
            text.push(String::new());
        }

        // handling the bottom border
        let lastIndex = text.len() - 1;
        if self.bordered {
            text[lastIndex].push_str(&color.0);
            text[lastIndex].push('└');
            text[lastIndex].push_str(&"─".repeat(self.size.0 as usize - 2));
            text[lastIndex].push('┘');
            text[lastIndex].push_str(CLEAR);
        } else {
            // removing the last \n
            text.pop();
        }
        text
    }

    // Replaces a single line with an updated version
    pub fn UpdateLine (&mut self, index: usize, span: Span) {
        if index >= self.lines.len() {  return;  }
        self.lines[index] = (span, String::new(), 0);
        self.updated[index] = false;
        self.wasUpdated = false;
    }

    // Appends a single line to the window
    pub fn AddLine (&mut self, span: Span) {
        self.lines.push((span, String::new(), 0));
        self.updated.push(false);
        self.wasUpdated = false;
    }

    // Takes a vector of type Span
    // That Span replaces the current set of lines for the window
    pub fn FromLines (&mut self, lines: Vec <Span>) {
        self.lines.clear();// self.updated.clear();
        let mut index = {
            if self.bordered {  1  }
            else {  0  }
        };
        for span in lines {
            self.lines.push((span, String::new(), 0));
            self.updated[index] = false;
            self.wasUpdated = false;
            index += 1;
        }
    }

    // checks to see if any lines need to be updated
    pub fn TryUpdateLines (&mut self, mut lines: Vec <Span>) {
        if lines.len() != self.lines.len() {
            self.UpdateAll();  // making sure every line gets updated (incase it was shrunk)
            self.wasUpdated = false;
            self.lines.clear();
            for (index, span) in lines.into_iter().enumerate() {
                if index >= self.updated.len() {  break;  }
                self.lines.push((span, String::new(), 0));
            }
            return;
        }
        let mut index = lines.len();
        let bordered = {
            if self.bordered {  1  }
            else {  0  }
        };
        while let Some(span) = lines.pop() {
            index -= 1;  // the pop already subtracted one
            if self.lines[index].0 != span {
                self.lines[index] = (span, String::new(), 0);
                self.updated[index + bordered] = false;  // it was as easy as adding a plus 1....... me sad
                self.wasUpdated = false;
            }
        }
    }

    pub fn IsEmpty (&self) -> bool {
        self.lines.is_empty()
    }

    pub fn UpdateAll (&mut self) {
        for line in self.updated.iter_mut() {
            *line = false;
        }
        self.wasUpdated = false;
    }

    pub fn SupressUpdates (&mut self) {
        for line in self.updated.iter_mut() {
            *line = true;
        }
        self.wasUpdated = true;
    }
}


// the main window/application that handles all the windows
#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct Rect {
    pub width: u16,
    pub height: u16,
}

// the main application. It stores and handles the active windows
// It also handles rendering the cumulative sum of the windows
#[derive(Debug, Default)]
pub struct App {
    area: Rect,
    activeWindows: Vec <(Window, Vec <String>)>,  // window, mods
    windowReferences: std::collections::HashMap <String, usize>,
    changeWindowLayout: bool,
    updated: bool,
    renderHandle: Option <std::thread::JoinHandle <()>>,
    buffer: std::sync::Arc <parking_lot::RwLock <String>>,
    resetWindows: bool,
}

impl Drop for App {
    fn drop (&mut self) {
        print!("{SHOW_CURSOR}");  // showing the cursor

        // clearing the screen
        //print!("\x1B[2J\x1B[H\x1b");
        print!("\x1B[0m");
        print!("\x1B[?1049l");
        print!("\x1B[2K\x1B[E");
        print!("\x1Bc");

        // I don't really care if an error is thrown at this point
        let _ = std::io::stdout().flush();
    }
}

impl App {
    pub fn new () -> Self {  // 1049h
        print!("\x1B7");
        print!("\x1B[?1049h");
        print!("\x1B[?25l");
        App {
            area: Rect::default(),
            activeWindows: vec![],
            windowReferences: std::collections::HashMap::new(),
            changeWindowLayout: true,
            updated: true,
            renderHandle: None,
            buffer: std::sync::Arc::new(parking_lot::RwLock::new(String::new())),
            resetWindows: false,
        }
    }

    pub fn ContainsWindow (&self, name: String) -> bool {
        self.windowReferences.contains_key(&name)
    }

    pub fn GetWindowReference (&self, name: String) -> &Window {
        &self.activeWindows[self.windowReferences[&name]].0
    }

    pub fn GetWindowReferenceMut (&mut self, name: String) -> &mut Window {
        //self.updated = true;  // assuming something is being changed
        &mut self.activeWindows[self.windowReferences[&name]].0
    }

    pub fn UpdateWindowLayoutOrder (&mut self) {
        self.changeWindowLayout = true;
        //self.updated = true;
    }

    pub fn GetTerminalSize (&self) -> Result <(u16, u16), std::io::Error> {
        crossterm::terminal::size()
    }

    // Gets the current window size and position
    // This returns a reference to this instance
    pub fn GetWindowArea (&self) -> &Rect {
        &self.area
    }

    // Adds a new active window
    pub fn AddWindow (&mut self, window: Window, name: String, keywords: Vec <String>) {
        if !window.hidden {  self.changeWindowLayout = true;  }  // if the window is hidden, it shouldn't change anything
        self.windowReferences.insert(name, self.windowReferences.len());
        self.activeWindows.push((window, keywords));
        //self.updated = true;
    }

    // Pops an active window.
    // Returns Ok(window) if the index is valid, or Err if out of bounds
    pub fn RemoveWindow (&mut self, name: String) -> Result <Window, String> {
        self.changeWindowLayout = true;
        self.resetWindows = true;
        //self.updated = true;

        if !self.windowReferences.contains_key(&name) {
            return Err(format!("No window named '{}' found", name));
        }
        // the None case would produce a value too large, so it should throw the expected error
        let index = *self.windowReferences.get(&name).unwrap_or(&usize::MAX);
        if index >= self.activeWindows.len() {
            return Err(format!(
                "Invalid index; Accessed at {}, but the size is {}",
                index, self.activeWindows.len()
            ));
        }

        // updating the references list
        self.windowReferences.remove(&name);
        let mut keysToModify = vec![];
        for key in self.windowReferences.iter() {
            if *key.1 > index {
                keysToModify.push(key.0.clone());
            }
        }
        for key in keysToModify {
            // all the keys should still exist
            *self.windowReferences.get_mut(&key).unwrap() -= 1;
        }

        Ok(self.activeWindows.remove(index).0)
    }

    /// Gathers the specified range of the string while accounting for non-visible
    /// UTF-8 character escape codes. Instead of each byte being a character, the characters
    /// are determined based on character boundaries and escape code sequences.
    pub fn GetSliceUTF_8 (text: &str, range: std::ops::Range <usize>) -> String
    where
        std::ops::Range<usize>: Iterator<Item = usize>
    {
        let mut visible = 0;
        let mut inEscape = false;
        let mut slice = String::new();
        for chr in text.chars() {
            if chr == '\x1b' {
                inEscape = true;

                // making sure to keep the initial escape codes
                slice.push(chr);
            } else if inEscape {
                inEscape = chr != 'm';

                // making sure to keep the initial escape codes
                slice.push(chr);
            } else {
                visible += 1;
                if visible >= range.start {
                    if visible < range.end {
                        // adding the element to the slice
                        slice.push(chr);
                        continue;
                    }
                    return slice;  // no need to continue
                }
            }
        } slice
    }

    fn HandleRenderWindowChanges (&mut self, size: &(u16, u16)) {
        let handle = self.renderHandle.take();
        if let Some(handle) = handle {
            // if an error was thrown, I don't care anymore
            let _ = handle.join();
        }

        self.buffer.write().clear();
        if size.0 != self.area.width || size.1 != self.area.height || self.resetWindows {
            self.resetWindows = false;
            *self.buffer.write() = String::with_capacity((size.0 * size.1 * 3) as usize);

            // making sure the windows get updated
            //self.updated = true;
            for window in &mut self.activeWindows {
                if window.0.hidden {  continue;  }  // hidden windows don't need re-rendering
                window.0.UpdateAll();
            }

            // replace with an actual clear..... this doesn't work (it just shifts the screen--or does it???)
            print!("\x1b[2J\x1b[H");  // re-clearing the screen (everything will need to update)
        }
    }

    // Renders all the active windows to the consol
    // It also clears the screen from previous writing
    pub fn Render (&mut self, terminalSize: Option <(u16, u16)>) -> usize {
        // incase the size is needed and thus calculated elsewhere (to prevent recalculation which is slow)
        // (aka I'm too lazy to update the code I already made.....)
        let size = terminalSize.unwrap_or(self.GetTerminalSize().unwrap());
        self.HandleRenderWindowChanges(&size);

        self.area = Rect {
            width: size.0,
            height: size.1,
        };

        // only re-rendering on updates (otherwise the current results are perfectly fine)
        // this should reduce CPU usage by a fair bit and allow a fast refresh rate if needed
        let mut updated = false;
        for window in &self.activeWindows {
            if window.0.wasUpdated {  continue;  }
            updated = true;
            break;
        }
        if !updated {  return 0;  }
        
        // stores the draw calls
        let mut drawCalls = vec![];

        // going through the sorted windows
        for window in &mut self.activeWindows {
            //let window = &mut self.activeWindows[*index];
            drawCalls.append(&mut window.0.GetRenderClosure());
        }

        let numCalls = drawCalls.len();

        let size = (self.area.width, self.area.height);
        let buffer = self.buffer.clone();
        //println!("Num calls: {}", drawCalls.len());
        self.renderHandle = Some(std::thread::spawn(move || {
            // the buffer for the render string

            // sorting the calls by action row (and left to right for same row calls)
            // drawCall.3 is the depth; higher numbers will be rendered last thus being on top (each depth is a unique layer)
            drawCalls.sort_by_key(|drawCall| drawCall.2 * size.0 + drawCall.1 + drawCall.3 * size.0 * size.1);

            // iterating through the calls (consuming drawCalls)
            let writeBuffer = &mut *buffer.write();
            for call in drawCalls {
                // moving the cursor into position
                // ESC[{line};{column}H
                writeBuffer.push_str("\x1b[");
                App::PushU16(writeBuffer, call.2);
                writeBuffer.push(';');
                App::PushU16(writeBuffer, call.1);
                writeBuffer.push('H');

                let output = call.0();
                writeBuffer.push_str(&output);
            }

            // moving the cursor to the bottom right
            writeBuffer.push_str("\x1b[");
            App::PushU16(writeBuffer, size.1);
            writeBuffer.push(';');
            App::PushU16(writeBuffer, size.0);
            writeBuffer.push_str("H ");

            // rendering the buffer
            let mut out = std::io::stdout().lock();
            out.write_all(writeBuffer.as_bytes()).unwrap();
            out.flush().unwrap();
        }));

        numCalls

        //let elapsed = start.elapsed();
        //panic!("Render thread completed in {:?}", elapsed);
    }

    /// Takes an u16 value and pushes the text form of it in an efficient manner.
    pub fn PushU16 (buffer: &mut String, mut value: u16) {
        let mut reserved = [0u32; 5];
        let mut i = 0;
        //println!(": {}", value);
        loop {
            reserved[i] = (value % 10) as u32;
            if value < 10 {  break;  }
            value /= 10;
            i += 1;
        }
        //println!("[{}, {}; {:?}]", value, i, reserved);
        for index in (0..=i).rev() {
            //println!("({:?}, {})", char::from_digit(reserved[index], 10), reserved[index]);
            buffer.push(char::from_digit(reserved[index], 10).unwrap_or_default());
        }
    }

    /// Returns a vector of references to the window names.
    /// References are being used to prevent unnecessary clones.
    pub fn GetWindowNames (&self) -> Vec<&String> {
        let mut names = vec![];
        for name in  self.windowReferences.keys() {
            names.push(name);
        } names
    }

    /// Prunes all windows which contain one of the specified keywords.
    /// Returns the number of windows pruned.
    pub fn PruneByKeywords (&mut self, keywords: Vec <String>) -> usize {
        let mut pruned = vec![];
        for (index, window) in self.activeWindows.iter().enumerate() {
            for word in &window.1 {
                if keywords.contains(word) {
                    //println!("\n:{:?}::", (index, word));
                    pruned.push(index);
                    break;
                }
            }
        }
        if pruned.is_empty() {  return 0;  }
        self.changeWindowLayout = true;
        self.resetWindows = true;
        self.updated = true;

        let mut numPruned = 0;
        // pruned should be in ascending order
        for index in &pruned {
            self.PruneUpdate(*index, &mut numPruned);
        } numPruned
    }

    fn PruneUpdate (&mut self, index: usize, numPruned: &mut usize) {
        // shifting all the indexes
        let mut toRemove = vec![];
        for pair in self.windowReferences.iter_mut() {
            if *pair.1 == index-*numPruned {
                toRemove.push(pair.0.clone());
                continue;
            }
            if *pair.1 >= index-*numPruned {  *pair.1 -= 1  }
        }
        for key in toRemove {
            self.windowReferences.remove(&key);
        }
        let _ = self.activeWindows.remove(index-*numPruned);
        *numPruned += 1;
    }

    /// Prunes all windows based on a given key (closure).
    /// Returns the number of windows pruned.
    /// If the closure returns true, the element is pruned. If it returns false it's kept.
    pub fn PruneByKey (&mut self, key: Box <dyn Fn (&Vec <String>) -> bool>) -> usize {
        let mut pruned = vec![];
        for (index, window) in self.activeWindows.iter().enumerate() {
            if key(&window.1) {
                pruned.push(index);
            }
        }
        if pruned.is_empty() {  return 0;  }
        self.changeWindowLayout = true;
        self.resetWindows = true;
        self.updated = true;

        let mut numPruned = 0;
        // pruned should be in ascending order
        for index in &pruned {
            self.PruneUpdate(*index, &mut numPruned);
        } numPruned
    }

    /// Gets the names to all windows which contain at least one of the
    /// specified keywords.
    pub fn GetWindowsByKeywords (&self, keywords: Vec <String>) -> Vec <&String> {
        let mut names = vec![];
        for name in &self.windowReferences {
            for keyword in &self.activeWindows[*name.1].1 {
                if keywords.contains(keyword) {
                    names.push(name.0);
                    break;
                }
            }
        }
        names
    }

    pub fn GetWindowsByKeywordsNonRef (&self, keywords: Vec <String>) -> Vec <String> {
        let mut names = vec![];
        for name in &self.windowReferences {
            for keyword in &self.activeWindows[*name.1].1 {
                if keywords.contains(keyword) {
                    names.push(name.0.clone());
                    break;
                }
            }
        } names
    }

    /// Gets the names to all windows which satisfy the given key (closure).
    /// If the closure returns true, the name is provided. Otherwise, it's
    /// considered unrelated.
    pub fn GetWindowsByKey (&self, key: Box <dyn Fn (&Vec <String>) -> bool>) -> Vec <&String> {
        let mut names = vec![];
        for name in &self.windowReferences {
            if key(&self.activeWindows[*name.1].1) {
                names.push(name.0);
            }
        }
        names
    }

    /// Checks if a given window contains a specific keyword.
    pub fn WindowContainsKeyword (&self, windowName: &String, keyword: &String) -> bool {
        let windowIndex = self.windowReferences[windowName];
        self.activeWindows[windowIndex].1.contains(keyword)
    }

    pub fn ChangedWindowLayout (&self) -> bool {
        self.changeWindowLayout
    }
}


use chrono;
use chrono::Datelike;
use serde;
use crate::TermRender;
use crate::TermRender::{Colorize};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Logs(Vec<Log>);

impl Logs {
    pub fn len(&self) -> usize { self.0.len() }
    pub fn push(&mut self, item: Log) { self.0.push(item) }
    pub fn new() -> Self { Self {0: vec![]} }
}

impl std::ops::Index<usize> for Logs {
    type Output = Log; // The type of the element returned by indexing
    
    fn index(&self, index: usize) -> &Self::Output {
        // Perform bounds checking and return a reference to the element
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Logs {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[derive(Clone)]
pub struct Log {
    pub entry_date: String,
    pub entry_title: String,
    pub entry_text: String,
    pub events: Option<Vec<String>>,
}

impl Log {
    pub fn new(entry_title: String, entry_text: String) -> Self {
        let time = chrono::Local::now();
        // time.month(), time.day(), time.year(), time.weekday()
        let entry_date = format!("{}, the {} of {}, {}", Self::get_week_day(&time), Self::get_day(&time), Self::get_month(&time), Self::get_year(&time));
        Self {
            entry_date,
            entry_title,
            entry_text,
            events: None,
        }
    }
    
    pub fn get_render_text(&self) -> Vec<TermRender::Span> {
        let date_span = TermRender::Span::FromTokens(vec![
            "     ".Colorizes(vec![]),
            self.entry_date.Colorizes(vec![TermRender::ColorType::White, TermRender::ColorType::Bold]),
        ]);
        let title_span = TermRender::Span::FromTokens(vec![
            " *".Colorizes(vec![]),
            self.entry_title.Colorizes(vec![TermRender::ColorType::White, TermRender::ColorType::Italic]),
            "*".Colorizes(vec![]),
        ]);
        let text_span = TermRender::Span::FromTokens(vec![
            self.entry_text.Colorizes(vec![TermRender::ColorType::White])
        ]);
        let mut events = vec![
            TermRender::Span::FromTokens(vec![]),
            TermRender::Span::FromTokens(vec![
                " Events:".Colorizes(vec![TermRender::ColorType::White, TermRender::ColorType::Italic])
            ]),
        ];
        for event in self.events.as_ref().unwrap_or(&vec![]) {
            let span = TermRender::Span::FromTokens(vec![
                "  * ".Colorizes(vec![TermRender::ColorType::White, TermRender::ColorType::Italic]),
                event.Colorizes(vec![TermRender::ColorType::White])
            ]);
            events.push(span);
        }
        vec![vec![date_span, title_span, text_span], events].concat()
    }
    
    pub fn get_title(&self) -> String {
        self.entry_title.clone()
    }
    
    pub fn get_date(&self) -> String {
        self.entry_date.clone()
    }
    
    pub fn add_event(&mut self, event: String) {
        if self.events.is_none() {  self.events = Some(vec![]);  }
        let events = self.events.as_mut().unwrap();
        events.push(event);
    }
    
    fn get_week_day(time: &chrono::prelude::DateTime<chrono::Local>) -> String {
        String::from(match time.weekday() {
            chrono::Weekday::Sun => "Sunday",
            chrono::Weekday::Mon => "Monday",
            chrono::Weekday::Tue => "Tuesday",
            chrono::Weekday::Wed => "Wednesday",
            chrono::Weekday::Thu => "Thursday",
            chrono::Weekday::Fri => "Friday",
            chrono::Weekday::Sat => "Saturday",
        })
    }
    
    fn get_day(time: &chrono::prelude::DateTime<chrono::Local>) -> String {
        let day = time.day();
        format!("{}{}", day, match day {
            1 | 21 | 31 => "st",
            2 | 22 => "nd",
            3 | 23 => "rd",
            _ => "th"
        })
    }
    
    fn get_month(time: &chrono::prelude::DateTime<chrono::Local>) -> String {
        String::from(match time.month() {
            1  => {"January"},
            2  => {"February"},
            3  => {"March"},
            4  => {"April"},
            5  => {"May"},
            6  => {"June"},
            7  => {"July"},
            8  => {"August"},
            9  => {"September"},
            10 => {"October"},
            11 => {"November"},
            _  => {"December"},
        })
    }
    
    fn get_year(time: &chrono::prelude::DateTime<chrono::Local>) -> String {
        format!("{}", time.year())
    }
}


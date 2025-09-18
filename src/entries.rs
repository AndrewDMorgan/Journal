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
    pub fn remove(&mut self, index: usize) -> Log { self.0.remove(index) }
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Mood {
    pub quality: usize,
    pub description: String,
    pub reason: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Log {
    pub entry_date: String,
    pub entry_title: String,
    pub entry_text: String,
    pub events: Option<Vec<String>>,
    pub food: Option<Vec<String>>,
    pub mood: Option<Mood>,
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
            food: None,
            mood: None,
        }
    }
    
    pub fn get_render_text(&self) -> Vec<TermRender::Span> {
        let date_span = TermRender::Span::FromTokens(vec![
            "  - ".Colorizes(vec![]),
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
        let mut events = vec![];
        if !self.events.as_ref().unwrap_or(&vec![]).is_empty() {
            events = vec![
                TermRender::Span::FromTokens(vec![]),
                TermRender::Span::FromTokens(vec![
                    " Events:".Colorizes(vec![TermRender::ColorType::White, TermRender::ColorType::Italic])
                ]),
            ];
        }
        for event in self.events.as_ref().unwrap_or(&vec![]) {
            let span = TermRender::Span::FromTokens(vec![
                "  * ".Colorizes(vec![TermRender::ColorType::White, TermRender::ColorType::Italic]),
                event.Colorizes(vec![TermRender::ColorType::White])
            ]);
            events.push(span);
        }
        
        let mut foods = vec![];
        if !self.food.as_ref().unwrap_or(&vec![]).is_empty() {
            foods = vec![
                TermRender::Span::FromTokens(vec![]),
                TermRender::Span::FromTokens(vec![
                    " Food:".Colorizes(vec![TermRender::ColorType::White, TermRender::ColorType::Italic])
                ]),
            ];
        }
        for item in self.food.as_ref().unwrap_or(&vec![]) {
            let span = TermRender::Span::FromTokens(vec![
                "  * ".Colorizes(vec![TermRender::ColorType::White, TermRender::ColorType::Italic]),
                item.Colorizes(vec![TermRender::ColorType::White])
            ]);
            foods.push(span);
        }
        
        let mut mood_text = vec![];
        if let Some(mood) = &self.mood {
            mood_text = vec![
                TermRender::Span::FromTokens(vec![]),
                TermRender::Span::FromTokens(vec![
                    " Mood:".Colorizes(vec![TermRender::ColorType::White, TermRender::ColorType::Italic])
                ]),
            ];
            
            // mood ranges from 1 through 10
            let mood_icon = match mood.quality {
                1 | 2 => "😞",
                3 | 4 => "😕",
                5 | 6 => "😐",
                7 | 8 => "🙂",
                9 | 10 => "😄",
                _ => "❓",
            };
            mood_text.push(
                TermRender::Span::FromTokens(vec![
                    " * ".Colorizes(vec![]),
                    mood_icon.Colorizes(vec![TermRender::ColorType::White]),
                    format!(" ({}/10)", mood.quality).Colorizes(vec![TermRender::ColorType::White]),
                    " ".Colorizes(vec![]),
                    mood.description.Colorizes(vec![TermRender::ColorType::White]),
                ])
            );
            if let Some(reason) = &mood.reason {
                mood_text.push(
                    TermRender::Span::FromTokens(vec![
                        " * Reason: ".Colorizes(vec![TermRender::ColorType::White, TermRender::ColorType::Italic]),
                        reason.Colorizes(vec![TermRender::ColorType::White]),
                    ])
                );
            }
        }
        
        vec![vec![date_span, title_span, text_span], events, foods, mood_text].concat()
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
    
    pub fn add_food(&mut self, item: String) {
        if self.food.is_none() {  self.food = Some(vec![]);  }
        let food = self.food.as_mut().unwrap();
        food.push(item);
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


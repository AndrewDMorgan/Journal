#[allow(non_snake_case)]
mod TermRender;
#[allow(non_snake_case)]
mod eventHandler;

mod entries;
use entries::Logs;

use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use eventHandler::*;
use tokio;
use tokio::io::{self, AsyncReadExt};
use crate::TermRender::{Colorize};

struct App {
    renderer: TermRender::App,
    area: TermRender::Rect,
    logs: Logs,
    scrolled: usize,
    selected: Option<usize>,
    creator_button: Option<CreatorButton>,
    editing_index: Option<usize>,
}

impl App {
    pub fn new() -> Self {

        let save = match std::fs::File::open("logs.json") {
            Ok(logs) => {
                let reader = std::io::BufReader::new(logs);
                serde_json::from_reader(reader).unwrap()
            },
            _ => Logs::new(),
        };
        //save.push(Log::new(String::from("Title"), String::from("Text")));
        let app = App {
            renderer: TermRender::App::new(),
            area: TermRender::Rect::default(),
            logs: save,
            scrolled: 0,
            selected: None,
            creator_button: None,
            editing_index: None,
        };
        Self::save(&app);
        app
    }
    
    fn save(&self) {
        let file = std::fs::File::create("logs.json").unwrap();
        serde_json::to_writer(file, &self.logs).unwrap();
    }
    
    async fn run(&mut self) {
        let result = self.run_internal().await;
        println!("{:?}", result);
        self.save();  // even if it crashes gently, it should hopefully save still
    }
    
    async fn run_internal(&mut self) -> io::Result<()> {
        let terminal_size = self.renderer.GetTerminalSize()?;
        self.area = TermRender::Rect {
            width: terminal_size.0,
            height: terminal_size.1
        };
        
        let mut stdout = std::io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        
        let mut parser = vte::Parser::new();
        let mut key_parser = KeyParser::new();
        let mut buffer = [0; 128];
        let mut stdin = io::stdin();
        
        self.scrolled = self.logs.len().saturating_sub(self.area.height as usize / 2 - 1);
        
        self.render_logs().await;
        self.renderer.Render(Some((self.area.width, self.area.height)));
        
        loop {
            let term_size = self.renderer.GetTerminalSize()?;
            self.area = TermRender::Rect {
                width: term_size.0,
                height: term_size.1,
            };
            
            // processing the events
            key_parser.ClearEvents();
            let result = tokio::time::timeout(tokio::time::Duration::from_secs_f64(0.2), stdin.read(&mut buffer)).await;
            if let Ok(Ok(n)) = result {
                key_parser.bytes = n;
                
                if n == 1 && buffer[0] == 0x1B {
                    key_parser.keyEvents.insert(KeyCode::Escape, true);
                } else {
                    parser.advance(&mut key_parser, &buffer[..n]);
                }
            }
            // control + c  ends the program
            if key_parser.ContainsModifier(&KeyModifiers::Control) && key_parser.ContainsChar('c') {  break;  }
            
            self.handle_events(&key_parser).await;
            self.render_logs().await;
            self.render_log_creation().await;
            self.renderer.Render(Some((self.area.width, self.area.height)));
        }
        self.save();
        
        Ok(())
    }
    
    async fn handle_events(&mut self, key_parser: &KeyParser) {
        self.handle_mouse_events(key_parser).await;
        if let Some(button) = &mut self.creator_button {
            button.handle_events(key_parser);
            if button.dead {
                let _ = self.renderer.RemoveWindow(String::from("CreatorMenu"));
                self.creator_button = None;
                self.save();  // making sure, if it did create a new entry, it is saved (any ways, saving such a small set of data isn't even that slow)
                self.editing_index = None;  // making sure it isn't still editing if it happened to be editing
            }
        }
    }
    
    async fn handle_mouse_events(&mut self, key_parser: &KeyParser) {
        if let Some(event) = &key_parser.mouseEvent {
            if event.eventType == MouseEventType::Left {
                // checking the bounds
                if self.creator_button.is_some() && event.position.0 > 25 && event.position.1 > 5 &&
                    event.position.0 < self.area.width - 25 && event.position.1 < self.area.height - 5 {
                    if event.state == MouseState::Release {
                        self.creator_button.as_mut().unwrap().handle_mouse_events_for_creator(key_parser, event, &self.area, &mut self.logs, self.editing_index);
                    } else {
                        self.creator_button.as_mut().unwrap().handle_held_mouse(key_parser, event, &self.area, &mut self.logs, self.editing_index);
                    }
                } else if event.position.0 < 50 {
                    if event.state != MouseState::Release {  return;  }
                    // getting the height
                    let index = event.position.1 as usize / 2 + self.scrolled - 1;
                    if index < self.logs.len() {
                        if self.creator_button.is_some() && self.renderer.ContainsWindow(String::from("CreatorMenu")) {
                            // to make sure no weird overlapping happens
                            self.renderer.GetWindowReferenceMut(String::from("CreatorMenu")).UpdateAll();
                        }
                        if self.selected.is_some() && self.selected.as_ref().unwrap() == &index {
                            self.selected = None;
                            // removing the render window for the log
                            let _ = self.renderer.RemoveWindow(String::from("LogView"));
                            let _ = self.renderer.RemoveWindow(String::from("EditButton"));
                            let _ = self.renderer.RemoveWindow(String::from("DelButton"));
                        } else {
                            if self.selected.is_some() {
                                self.renderer.GetWindowReferenceMut(String::from("EditButton")).UpdateAll();  // so it isn't clipped in half
                                self.renderer.GetWindowReferenceMut(String::from("DelButton")).UpdateAll();  // so it isn't clipped in half
                                self.renderer.GetWindowReferenceMut(String::from("Create")).UpdateAll();  // so it isn't clipped in half
                            }
                            self.selected = Some(index);
                        }
                    }
                } else if event.state != MouseState::Release {
                    return;  // only care about clicks, not releases outside the menu and scrolling in log list
                } else if event.position.0 >= self.area.width - 16 && event.position.0 < self.area.width &&
                          event.position.1 >= self.area.height - 3 && event.position.1 < self.area.height && self.editing_index.is_none() {
                    // opening the creation menu
                    if self.creator_button.is_none() {
                        self.creator_button = Some(CreatorButton::new());
                        self.editing_index = None;  // not editing rn
                    } else {
                        let _ = self.renderer.RemoveWindow(String::from("CreatorMenu"));
                        self.creator_button = None;
                    }
                } else if event.position.0 >= self.area.width - 11 && event.position.1 <= self.area.width - 2 &&
                          event.position.1 > 1 && event.position.1 < 5 && self.creator_button.is_none() &&
                          self.renderer.ContainsWindow(String::from("EditButton")) {
                    // editing the tab    unwrapping should be safe because the edit button is only created when a menu is open
                    if self.creator_button.is_none() {
                        self.editing_index = Some(self.selected.unwrap());
                        
                        let mut button = CreatorButton::new();
                        let log = &self.logs[self.selected.unwrap()];
                        button.events = log.events.clone().unwrap_or(vec![]);
                        button.title = log.entry_title.clone();
                        button.text = log.entry_text.clone();
                        button.events = log.events.as_ref().unwrap_or(&vec![]).clone();
                        button.food = log.food.as_ref().unwrap_or(&vec![]).clone();
                        button.mood_quality = log.mood.as_ref().map_or(5, |m| m.quality);
                        button.mood_description = log.mood.as_ref().map_or(String::new(), |m| m.description.clone());
                        button.mood_reason = log.mood.as_ref().and_then(|m| m.reason.clone()).unwrap_or(String::new());
                        button.update_cursors();
                        
                        self.creator_button = Some(button);
                    } else {
                        let _ = self.renderer.RemoveWindow(String::from("CreatorMenu"));
                        self.creator_button = None;
                    }
                } else if event.position.0 >= self.area.width - 22 && event.position.1 <= self.area.width - 13 &&
                    event.position.1 > 1 && event.position.1 < 5 && self.creator_button.is_none() &&
                    self.renderer.ContainsWindow(String::from("DelButton")) && self.creator_button.is_none() {
                    // deleting the log
                    if self.selected.is_some() {
                        self.logs.remove(self.selected.unwrap());
                        self.selected = None;
                        let _ = self.renderer.RemoveWindow(String::from("LogView"));
                        let _ = self.renderer.RemoveWindow(String::from("EditButton"));
                        let _ = self.renderer.RemoveWindow(String::from("DelButton"));
                        self.scrolled = self.scrolled.saturating_sub(1);
                        self.save();  // saving the result
                    }
                }
            } else if event.position.0 < 50 {
                // checking for scrolling
                if event.eventType == MouseEventType::Down {
                    self.scrolled = usize::min(
                        self.scrolled + (key_parser.scrollAccumulate * 4.) as usize,
                        self.logs.len().saturating_sub(1)
                    );
                }
                if event.eventType == MouseEventType::Up {
                    self.scrolled = self.scrolled.saturating_sub((key_parser.scrollAccumulate * -4.) as usize);
                }
            }
        }
    }
    
    async fn render_log_creation(&mut self) {
        // rendering the create log button
        if self.renderer.ContainsWindow(String::from("Create")) {
            let button = self.renderer.GetWindowReferenceMut(String::from("Create"));
            button.Resize((15, 3));
            button.Move((self.area.width - 16, self.area.height - 3))
        } else {
            let mut window = TermRender::Window::new((self.area.width - 16, self.area.height - 3), 1, (15, 3));
            window.AddLine(TermRender::Span::FromTokens(vec!["  New Entry    ".Colorizes(vec![TermRender::ColorType::White, TermRender::ColorType::OnBrightBlack])]));
            window.Bordered();
            window.Colorize(TermRender::ColorType::OnBrightBlack);
            self.renderer.AddWindow(window, String::from("Create"), vec![])
        }
        
        // rendering the creator button
        if let Some(button) = &self.creator_button {
            if self.renderer.ContainsWindow(String::from("CreatorMenu")) {
                let button_renderer = self.renderer.GetWindowReferenceMut(String::from("CreatorMenu"));
                button_renderer.Resize((self.area.width - 50, self.area.height - 10));
                button_renderer.TryUpdateLines(button.get_window_text(&self.area));
            } else {
                let mut window = TermRender::Window::new((25, 5), 2, (self.area.width - 50, self.area.height - 10));
                window.Bordered();
                window.Colorize(TermRender::ColorType::Bold);
                let create_text = match self.editing_index {
                    Some(_) => " Editing Entry ",
                    None =>    "Create New Entry"
                };
                window.TitledColored(TermRender::Span::FromTokens(vec![
                    create_text.Colorizes(vec![TermRender::ColorType::Bold, TermRender::ColorType::BrightWhite])
                ]));
                window.FromLines(button.get_window_text(&self.area));
                self.renderer.AddWindow(window, String::from("CreatorMenu"), vec![String::from("Pop Up")]);
            }
        }
    }
    
    async fn render_logs(&mut self) {
        let mut render = vec![];
        let start_index = self.scrolled;
        for index in start_index..self.logs.len() {
            let log = &self.logs[index];
            // printing the title and date
            let mut tokens = vec![" ".Colorizes(vec![]), log.get_title().Colorizes(vec![
                TermRender::ColorType::BrightWhite, TermRender::ColorType::Bold
            ]), "                                                                      ".Colorizes(vec![])];
            if self.selected.as_ref().unwrap_or(&usize::MAX) == &index {
                for token in tokens.iter_mut(){
                    token.AddColor(TermRender::ColorType::OnBrightBlack);
                }
            }
            render.push(TermRender::Span::FromTokens(tokens));
            let mut tokens = vec!["    ".Colorizes(vec![]), log.get_date().Colorizes(vec![
                TermRender::ColorType::White, TermRender::ColorType::Italic
            ]), "                                                                      ".Colorizes(vec![])];
            if self.selected.as_ref().unwrap_or(&usize::MAX) == &index {
                for token in tokens.iter_mut(){
                    token.AddColor(TermRender::ColorType::OnBrightBlack);
                }
            }
            render.push(TermRender::Span::FromTokens(tokens));
        }
        
        if self.renderer.ContainsWindow(String::from("Logs")) {
            let logs = self.renderer.GetWindowReferenceMut(String::from("Logs"));
            logs.Resize((50, self.area.height));
            logs.TryUpdateLines(render);
        } else {
            let mut window = TermRender::Window::new((1, 1), 0, (50, self.area.height));
            window.FromLines(render);
            window.Bordered();
            self.renderer.AddWindow(window, String::from("Logs"), vec![]);
        }
        
        // rendering the actual log if one is open
        if self.selected.is_none() {  return;  }
        if self.renderer.ContainsWindow(String::from("LogView")) {
            let log = self.renderer.GetWindowReferenceMut(String::from("LogView"));
            log.TryUpdateLines(self.logs[*self.selected.as_ref().unwrap()].get_render_text());
            log.Resize((self.area.width - 49, self.area.height));
            
            // adding the edit button     String::from("EditButton")
            let log = self.renderer.GetWindowReferenceMut(String::from("EditButton"));
            log.Move((self.area.width - 11, 2));
            
            let log = self.renderer.GetWindowReferenceMut(String::from("DelButton"));
            log.Move((self.area.width - 22, 2));
        } else {
            let mut window = TermRender::Window::new((50, 1), 0, (self.area.width - 49, self.area.height));
            window.Bordered();
            // adding the text
            window.FromLines(self.logs[*self.selected.as_ref().unwrap()].get_render_text());
            self.renderer.AddWindow(window, String::from("LogView"), vec![]);
            
            // adding the edit button     String::from("EditButton")
            let mut window = TermRender::Window::new((self.area.width - 11, 2), 1, (10, 3));
            window.Bordered();
            window.AddLine(TermRender::Span::FromTokens(vec![
                "  Edit  ".Colorizes(vec![TermRender::ColorType::Bold, TermRender::ColorType::BrightWhite])
            ]));
            self.renderer.AddWindow(window, String::from("EditButton"), vec![]);
            
            let mut window = TermRender::Window::new((self.area.width - 22, 2), 1, (10, 3));
            window.Bordered();
            window.AddLine(TermRender::Span::FromTokens(vec![
                " Delete ".Colorizes(vec![TermRender::ColorType::Bold, TermRender::ColorType::BrightWhite])
            ]));
            self.renderer.AddWindow(window, String::from("DelButton"), vec![]);

            // updating the creation button
            self.renderer.GetWindowReferenceMut(String::from("Create")).UpdateAll();
            if self.creator_button.is_some() {
                self.renderer.GetWindowReferenceMut(String::from("CreatorMenu")).UpdateAll();
            }
        }
    }
}

#[derive(PartialEq, Eq)]
enum CreationField {
    Title,
    Text,
    Events,
    Foods,
    MoodDescription,
    MoodReason,
}

struct CreatorButton {
    selected_field: Option<CreationField>,
    title: String,
    text: String,
    cursors: [usize; 6],
    pub dead: bool,
    events: Vec<String>,
    food: Vec<String>,
    mood_quality: usize,
    mood_description: String,
    mood_reason: String,
}

impl CreatorButton {
    pub fn new() -> Self {
        CreatorButton {
            selected_field: None,
            title: String::new(),
            text: String::new(),
            cursors: [0usize; 6],
            dead: false,
            events: vec![],
            food: vec![],
            mood_quality: 5,
            mood_description: String::new(),
            mood_reason: String::new(),
        }
    }
    
    pub fn update_cursors(&mut self) {
        self.cursors = [
            self.title.len(),
            self.text.len(),
            0, 0,
            self.mood_description.len(),
            self.mood_reason.len(),
        ];
    }
    
    pub fn handle_events(&mut self, key_parser: &KeyParser) {
        if key_parser.ContainsKeyCode(KeyCode::Return) {
            self.selected_field = None;
        }
        if key_parser.ContainsKeyCode(KeyCode::Escape) {
            match &self.selected_field {
                Some(err) => self.selected_field = None,
                None => self.dead = true,
            }
        }
        
        let typed_text = match key_parser.keyModifiers.is_empty() {
            true => {
                let mut text = String::new();
                for char in &key_parser.charEvents {
                    text.push(*char);
                } text
            },
            false => String::new()
        };
        
        match self.selected_field {
            Some(CreationField::Title) => {
                self.title.insert_str(self.cursors[0], &typed_text);
                self.cursors[0] += typed_text.len();
                if key_parser.ContainsKeyCode(KeyCode::Delete) {
                    if self.cursors[0] > self.title.len() || self.title.is_empty() || self.cursors[0] == 0 {  return;  }
                    self.title.remove(self.cursors[0].saturating_sub(1));
                    self.cursors[0] -= 1;
                }
                if key_parser.ContainsKeyCode(KeyCode::Left) {
                    self.cursors[0] = self.cursors[0].saturating_sub(1);
                }
                if key_parser.ContainsKeyCode(KeyCode::Right) {
                    self.cursors[0] = usize::min(self.cursors[0] + 1, self.title.len());
                }
            },
            Some(CreationField::Text) => {
                self.text.insert_str(self.cursors[1], &typed_text);
                self.cursors[1] += typed_text.len();
                if key_parser.ContainsKeyCode(KeyCode::Delete) {
                    if self.cursors[1] > self.text.len() || self.text.is_empty() || self.cursors[1] == 0 {  return;  }
                    self.text.remove(self.cursors[1].saturating_sub(1));
                    self.cursors[1] -= 1;
                }
                if key_parser.ContainsKeyCode(KeyCode::Left) {
                    self.cursors[1] = self.cursors[1].saturating_sub(1);
                }
                if key_parser.ContainsKeyCode(KeyCode::Right) {
                    self.cursors[1] = usize::min(self.cursors[1] + 1, self.text.len());
                }
            },
            Some(CreationField::Events) => {
                if self.cursors[2] >= self.events.len() {  return;  }
                let text_field = &mut self.events[self.cursors[2]];
                text_field.push_str(&typed_text);
                if key_parser.ContainsKeyCode(KeyCode::Delete) {
                    text_field.pop();
                }
            },
            Some(CreationField::Foods) => {
                if self.cursors[3] >= self.food.len() {  return;  }
                let text_field = &mut self.food[self.cursors[3]];
                text_field.push_str(&typed_text);
                if key_parser.ContainsKeyCode(KeyCode::Delete) {
                    text_field.pop();
                }
            },
            Some(CreationField::MoodDescription) => {
                self.mood_description.insert_str(self.cursors[4], &typed_text);
                self.cursors[4] += typed_text.len();
                if key_parser.ContainsKeyCode(KeyCode::Delete) {
                    if self.cursors[4] > self.mood_description.len() || self.mood_description.is_empty() || self.cursors[4] == 0 {  return;  }
                    self.mood_description.remove(self.cursors[4].saturating_sub(1));
                    self.cursors[4] -= 1;
                }
                if key_parser.ContainsKeyCode(KeyCode::Left) {
                    self.cursors[4] = self.cursors[4].saturating_sub(1);
                }
                if key_parser.ContainsKeyCode(KeyCode::Right) {
                    self.cursors[4] = usize::min(self.cursors[4] + 1, self.mood_description.len());
                }
            },
            Some(CreationField::MoodReason) => {
                self.mood_reason.insert_str(self.cursors[5], &typed_text);
                self.cursors[5] += typed_text.len();
                if key_parser.ContainsKeyCode(KeyCode::Delete) {
                    if self.cursors[5] > self.mood_reason.len() || self.mood_reason.is_empty() || self.cursors[5] == 0 {  return;  }
                    self.mood_reason.remove(self.cursors[5].saturating_sub(1));
                    self.cursors[5] -= 1;
                }
                if key_parser.ContainsKeyCode(KeyCode::Left) {
                    self.cursors[5] = self.cursors[5].saturating_sub(1);
                }
                if key_parser.ContainsKeyCode(KeyCode::Right) {
                    self.cursors[5] = usize::min(self.cursors[5] + 1, self.mood_reason.len());
                }
            },
            _ => {}
        }
    }
    
    pub fn handle_held_mouse (&mut self, _key_parser: &KeyParser, event: &MouseEvent, area: &TermRender::Rect, _logs: &mut Logs, _index: Option<usize>) {
        let half_width = area.width / 2;
        let starting_index = 15 + self.events.len() + 2 + self.food.len() + 2;
        if event.position.1 == starting_index as u16 - 1 && event.position.0 >= half_width - 10 && event.position.0 <= half_width + 10 {
            // adjusting the mood quality
            let quality = event.position.0 - (half_width - 10);
            self.mood_quality = (quality / 2).clamp(1, 10) as usize;
            return;
        }
    }
    
    pub fn handle_mouse_events_for_creator(&mut self, _key_parser: &KeyParser, event: &MouseEvent, area: &TermRender::Rect, logs: &mut Logs, index: Option<usize>) {
        // checking for a text field being selected
        let title_width = self.title.len() as u16 / 2 + 5;
        let half_width = area.width / 2;
        if event.position.0 >= half_width - title_width && event.position.0 <= half_width + title_width && event.position.1 == 8 {
            self.selected_field = match &self.selected_field {
                Some(field) if field == &CreationField::Title => None,
                _ => Some(CreationField::Title),
            };
            return;
        }
        let text_width = self.text.len() as u16 / 2 + 5;
        if event.position.0 >= half_width - text_width && event.position.0 <= half_width + text_width && event.position.1 == 11 {
            self.selected_field = match &self.selected_field {
                Some(field) if field == &CreationField::Text => None,
                _ => Some(CreationField::Text),
            };
            return;
        }
        
        // checking for the create button being pushed
        //area.width as usize - 50 - 13
        if event.position.0 >= area.width - 38 && event.position.0 < area.width - 26 &&
           event.position.1 <= area.height - 6 && event.position.1 >= area.height - 9 {
            // creating the thingy
            if let Some(index) = index {
                logs[index] = entries::Log::new(self.title.clone(), self.text.clone());
                for event in &self.events {
                    logs[index].add_event(event.clone());
                }
                for food in &self.food {
                    logs[index].add_food(food.clone());
                }
                logs[index].mood = Some(entries::Mood {
                    quality: self.mood_quality,
                    description: self.mood_description.clone(),
                    reason: if self.mood_reason.is_empty() { None } else { Some(self.mood_reason.clone()) },
                });
                self.dead = true;
                return;
            }
            let index = logs.len();
            logs.push(entries::Log::new(self.title.clone(), self.text.clone()));
            for event in &self.events {
                logs[index].add_event(event.clone());
            }
            for food in &self.food {
                logs[index].add_food(food.clone());
            }
            logs[index].mood = Some(entries::Mood {
                quality: self.mood_quality,
                description: self.mood_description.clone(),
                reason: if self.mood_reason.is_empty() { None } else { Some(self.mood_reason.clone()) },
            });
            self.dead = true;
            return;
        }
        
        if event.position.0 >= half_width - 10 && event.position.0 <= half_width + 10 && event.position.1 == 13 {
            self.selected_field = Some(CreationField::Events);
            self.cursors[2] = self.events.len();
            self.events.push(String::new());
            return;
        }
        
        // checking for individual event elements
        if event.position.1 > 13 && event.position.1 <= 13 + self.events.len() as u16 &&
           event.position.0 >= half_width - 5 - self.events[event.position.1 as usize - 14].len() as u16 / 2 &&
           event.position.0 <= half_width + 5 + self.events[event.position.1 as usize - 14].len() as u16 / 2
        {
            self.selected_field = Some(CreationField::Events);
            self.cursors[2] = event.position.1 as usize - 14;
            
            return;
        }
        
        // checking for foods
        let starting_index = 15 + self.events.len();
        if event.position.0 >= half_width - 10 && event.position.0 <= half_width + 10 && event.position.1 == starting_index as u16 {
            self.selected_field = Some(CreationField::Foods);
            self.cursors[3] = self.food.len();
            self.food.push(String::new());
            return;
        }
        
        // checking for individual event elements
        if event.position.1 > starting_index as u16 && event.position.1 <= starting_index as u16 + self.food.len() as u16 &&
            event.position.0 >= half_width - 5 - self.food[event.position.1 as usize - starting_index - 1].len() as u16 / 2 &&
            event.position.0 <= half_width + 5 + self.food[event.position.1 as usize - starting_index - 1].len() as u16 / 2
        {
            self.selected_field = Some(CreationField::Foods);
            self.cursors[3] = event.position.1 as usize - starting_index - 1;
            return;
        }
        
        let starting_index = 15 + self.events.len() + 2 + self.food.len() + 2;
        
        if event.position.1 == starting_index as u16 + 2 && event.position.0 >= half_width - self.mood_description.len() as u16 - 5 &&
           event.position.0 <= half_width + self.mood_description.len() as u16 + 5
        {
            self.selected_field = Some(CreationField::MoodDescription);
            return;
        }
        
        if event.position.1 == starting_index as u16 + 5 && event.position.0 >= half_width - self.mood_reason.len() as u16 - 5 &&
           event.position.0 <= half_width + self.mood_reason.len() as u16 + 5
        {
            self.selected_field = Some(CreationField::MoodReason);
            return;
        }
        
        self.selected_field = None;  // clicked outside a field
    }
    
    pub fn get_window_text(&self, area: &TermRender::Rect) -> Vec<TermRender::Span> {
        // a bunch of blank elements to make it easier (in other words, I'm lazy)
        
        let mut render = vec![];
        for _ in 0..area.height - 12 {
            render.push(TermRender::Span::FromTokens(vec!["".Colorizes(vec![])]));
        }
        
        // adding the field for title
        render[1] = TermRender::Span::FromTokens(vec![
            Self::center_padding(area, "*Title*".len()),
            "*Title*".Colorizes(vec![TermRender::ColorType::Italic, TermRender::ColorType::BrightWhite]),
        ]);
        let field_text = String::from(match self.title.is_empty() {
            true => "-- Title Here --",
            false => &self.title
        });
        render[2] = TermRender::Span::FromTokens(vec![
            Self::center_padding(area, field_text.len()),
            field_text.Colorizes({
                match self.selected_field {
                    Some(CreationField::Title) => vec![TermRender::ColorType::White, TermRender::ColorType::Underline],
                    _ => vec![TermRender::ColorType::White],
                }
            }),
        ]);
        
        // adding the field for text
        render[4] = TermRender::Span::FromTokens(vec![
            Self::center_padding(area, "*Entry Text*".len()),
            "*Entry Text*".Colorizes(vec![TermRender::ColorType::Italic, TermRender::ColorType::BrightWhite]),
        ]);
        let field_text = String::from(match self.text.is_empty() {
            true => "-- Text Here --",
            false => &self.text
        });
        render[5] = TermRender::Span::FromTokens(vec![
            Self::center_padding(area, field_text.len()),
            field_text.Colorizes({
                match self.selected_field {
                    Some(CreationField::Text) => vec![TermRender::ColorType::White, TermRender::ColorType::Underline],
                    _ => vec![TermRender::ColorType::White],
                }
            }),
        ]);
        
        // rendering the button to add another event
        render[7] = TermRender::Span::FromTokens(vec![
            Self::center_padding(area, "*Add Event*".len()),
            "*Add Events*".Colorizes(vec![TermRender::ColorType::BrightWhite, TermRender::ColorType::Italic])
        ]);
        
        // rendering the current events
        let mut index = 8;
        for event in &self.events {
            let field_text = String::from(match event.is_empty() {
                true => "-- Text Here --",
                false => event
            });
            render[index] = TermRender::Span::FromTokens(vec![
                Self::center_padding(area, field_text.len()),
                field_text.Colorizes({
                    if self.selected_field == Some(CreationField::Events) && self.cursors[2] == index - 8 {
                        vec![TermRender::ColorType::White, TermRender::ColorType::Underline]
                    } else {  vec![TermRender::ColorType::White]  }
                })
            ]);
            index += 1;
        }
        
        // rendering the button to add another event
        index += 1;
        render[index] = TermRender::Span::FromTokens(vec![
            Self::center_padding(area, "*Add Foods*".len()),
            "*Add Foods*".Colorizes(vec![TermRender::ColorType::BrightWhite, TermRender::ColorType::Italic])
        ]);
        index += 1;
        
        // rendering the current events
        let start_index = index;
        for item in &self.food {
            let field_text = String::from(match item.is_empty() {
                true => "-- Text Here --",
                false => item
            });
            render[index] = TermRender::Span::FromTokens(vec![
                Self::center_padding(area, field_text.len()),
                field_text.Colorizes({
                    if self.selected_field == Some(CreationField::Foods) && self.cursors[3] == index - start_index {
                        vec![TermRender::ColorType::White, TermRender::ColorType::Underline]
                    } else {  vec![TermRender::ColorType::White]  }
                })
            ]);
            index += 1;
        }
        
        index += 1;
        render[index] = TermRender::Span::FromTokens(vec![
            Self::center_padding(area, "*Mood (1-10)*".len()),
            "*Mood (1-10)*".Colorizes(vec![TermRender::ColorType::BrightWhite, TermRender::ColorType::Italic])
        ]);
        index += 1;
        
        // rendering mood quality (slider of sorts ig)
        // slides from left to right, using a white background with bright white slider, and black text
        let quality_text = vec![
            "==".repeat(self.mood_quality - 1).Colorizes(vec![TermRender::ColorType::OnWhite, TermRender::ColorType::BrightBlack]),
            format!("{:=>2}", self.mood_quality).Colorizes(vec![TermRender::ColorType::Black, TermRender::ColorType::OnBrightWhite]),
            "==".repeat(10 - self.mood_quality).Colorizes(vec![TermRender::ColorType::OnWhite, TermRender::ColorType::BrightBlack]),
        ];  // the total size is 2 * 10 or 20
        render[index] = TermRender::Span::FromTokens(vec![
            Self::center_padding(area, 20),
            quality_text[0].clone(),
            quality_text[1].clone(),
            quality_text[2].clone(),
        ]);
        index += 2;
        
        render[index] = TermRender::Span::FromTokens(vec![
            Self::center_padding(area, "*Mood Description*".len()),
            "*Mood Description*".Colorizes(vec![TermRender::ColorType::BrightWhite, TermRender::ColorType::Italic])
        ]);
        index += 1;
        
        // rendering mood description
        let field_text = String::from(match self.mood_description.is_empty() {
            true => "-- Text Here --",
            false => &self.mood_description
        });
        render[index] = TermRender::Span::FromTokens(vec![
            Self::center_padding(area, field_text.len()),
            field_text.Colorizes(match self.selected_field {
                Some(CreationField::MoodDescription) => vec![TermRender::ColorType::White, TermRender::ColorType::Underline],
                _ => vec![TermRender::ColorType::White],
            })
        ]);
        index += 2;
        
        render[index] = TermRender::Span::FromTokens(vec![
            Self::center_padding(area, "*Mood Reason*".len()),
            "*Mood Reason*".Colorizes(vec![TermRender::ColorType::BrightWhite, TermRender::ColorType::Italic])
        ]);
        index += 1;
        
        // rendering mood reason
        let field_text = String::from(match self.mood_reason.is_empty() {
            true => "-- Text Here --",
            false => &self.mood_reason
        });
        render[index] = TermRender::Span::FromTokens(vec![
            Self::center_padding(area, field_text.len()),
            field_text.Colorizes(match self.selected_field {
                Some(CreationField::MoodReason) => vec![TermRender::ColorType::White, TermRender::ColorType::Underline],
                _ => vec![TermRender::ColorType::White],
            })
        ]);
        index += 2;
        
        // adding the button for completion
        let render_len = render.len() - 1;
        let padding = " ".repeat(area.width as usize - 50 - 13).Colorizes(vec![]);
        render[render_len - 2] = TermRender::Span::FromTokens(vec![
            padding.clone(), "┌────────┐".Colorizes(vec![])
        ]);
        render[render_len - 1] = TermRender::Span::FromTokens(vec![
            padding.clone(), "│ Create │".Colorizes(vec![])
        ]);
        render[render_len    ] = TermRender::Span::FromTokens(vec![
            padding,         "└────────┘".Colorizes(vec![])
        ]);
        
        render
    }
    
    fn center_padding(area: &TermRender::Rect, text_size: usize) -> TermRender::Colored {
        let text_size = text_size / 2;
        let center = (area.width - 50) as usize / 2;
        let offset = center - text_size;
        " ".repeat(offset).Colorizes(vec![])
    }
}

/*
todo!s:

    todo! add scrolling before I get flooded!!!
        -- now just de-jank-ify it (could be a lot smoother, but it seems to work for now at a minimum)
     
    at some point think about adding a delete option ig  (could finally get rid of the initial test file)
        -- also delete options for the elements in lists like events
    
    todo! make term render correctly render borders when using emojis (which aren't represented with escape codes)
 
*/

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> io::Result<()> {
    // this runtime is implemented in a way where blocking tasks/blocking thread sleeps don't block others tasks from running
    // each task gets its own thread so blocking is safe unless the section requires a safe/soft exit instead of a hard drop
    enableMouseCapture().await;
    enable_raw_mode()?;
    
    // starting the app
    let mut app = App::new();
    app.run().await;
    
    disableMouseCapture().await;
    disable_raw_mode()?;
    
    Ok(())
}

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
        };
        Self::save(&app);
        app
    }
    
    fn save(&self) {
        let file = std::fs::File::create("logs.json").unwrap();
        serde_json::to_writer(file, &self.logs).unwrap();
    }
    
    async fn run(&mut self) {
        let _result = self.run_internal().await;
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
    }
    
    async fn handle_mouse_events(&mut self, key_parser: &KeyParser) {
        if let Some(event) = &key_parser.mouseEvent {
            if event.eventType == MouseEventType::Left && event.state == MouseState::Release {
                // checking the bounds
                if event.position.0 < 50 {
                    // getting the height
                    let index = event.position.1 as usize / 2 + self.scrolled - 1;
                    if index < self.logs.len() {
                        if self.selected.is_some() {
                            self.selected = None;
                            // removing the render window for the log
                            let _ = self.renderer.RemoveWindow(String::from("LogView"));
                        }
                        else {  self.selected = Some(index);  }
                    }
                } else if event.position.0 >= self.area.width - 16 && event.position.0 < self.area.width &&
                          event.position.1 >= self.area.height - 3 && event.position.1 < self.area.height {
                    // opening the creation menu
                    if self.creator_button.is_none() {  self.creator_button = Some(CreatorButton::new());  }
                    else {
                        let _ = self.renderer.RemoveWindow(String::from("CreatorMenu"));
                        self.creator_button = None;
                    }
                } else if self.creator_button.is_some() && event.position.0 > 25 && event.position.1 > 5 &&
                          event.position.0 < self.area.width - 25 && event.position.1 < self.area.height - 5 {
                    self.creator_button.as_mut().unwrap().handle_mouse_events_for_creator(key_parser, event, &self.area);
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
                window.TitledColored(TermRender::Span::FromTokens(vec![
                    "Create New Entry".Colorizes(vec![TermRender::ColorType::Bold, TermRender::ColorType::BrightWhite])
                ]));
                window.FromLines(button.get_window_text(&self.area));
                self.renderer.AddWindow(window, String::from("CreatorMenu"), vec![String::from("Pop Up")]);
            }
        }
    }
    
    async fn render_logs(&mut self) {
        let mut render = vec![];
        let start_index = self.logs.len().saturating_sub(self.scrolled + (self.area.height / 2) as usize);
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
        } else {
            let mut window =TermRender::Window::new((50, 1), 0, (self.area.width - 49, self.area.height));
            window.Bordered();
            // adding the text
            window.FromLines(self.logs[*self.selected.as_ref().unwrap()].get_render_text());
            self.renderer.AddWindow(window, String::from("LogView"), vec![]);
            
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
}

struct CreatorButton {
    selected_field: Option<CreationField>,
    title: String,
    text: String,
}

impl CreatorButton {
    pub fn new() -> Self {
        CreatorButton {
            selected_field: None,
            title: String::new(),
            text: String::new(),
        }
    }
    
    pub fn handle_mouse_events_for_creator(&mut self, _key_parser: &KeyParser, event: &MouseEvent, area: &TermRender::Rect) {
        // checking for a text field being selected
        let title_width = self.title.len() as u16 / 2 + 5;
        let half_width = area.width / 2;
        if event.position.0 >= half_width - title_width && event.position.0 <= half_width + title_width && event.position.1 == 8 {
            self.selected_field = match &self.selected_field {
                Some(field) if field == &CreationField::Title => None,
                _ => Some(CreationField::Title),
            }
        }
        let text_width = self.text.len() as u16 / 2 + 5;
        if event.position.0 >= half_width - text_width && event.position.0 <= half_width + text_width && event.position.1 == 11 {
            self.selected_field = match &self.selected_field {
                Some(field) if field == &CreationField::Text => None,
                _ => Some(CreationField::Text),
            }
        }
    }
    
    pub fn get_window_text(&self, area: &TermRender::Rect) -> Vec<TermRender::Span> {
        // a bunch of blank elements to make it easier (in other words, I'm lazy)
        
        let mut render = vec![
            TermRender::Span::FromTokens(vec!["".Colorizes(vec![])]),
            TermRender::Span::FromTokens(vec!["".Colorizes(vec![])]),  // field name
            TermRender::Span::FromTokens(vec!["".Colorizes(vec![])]),  // title field
            TermRender::Span::FromTokens(vec!["".Colorizes(vec![])]),
            TermRender::Span::FromTokens(vec!["".Colorizes(vec![])]),  // field name
            TermRender::Span::FromTokens(vec!["".Colorizes(vec![])]),  // text field
            TermRender::Span::FromTokens(vec!["".Colorizes(vec![])]),
            TermRender::Span::FromTokens(vec!["".Colorizes(vec![])]),
            TermRender::Span::FromTokens(vec!["".Colorizes(vec![])]),
            TermRender::Span::FromTokens(vec!["".Colorizes(vec![])]),
        ];
        
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
        
        render
    }
    
    fn center_padding(area: &TermRender::Rect, text_size: usize) -> TermRender::Colored {
        let text_size = text_size / 2;
        let center = (area.width - 50) as usize / 2;
        let offset = center - text_size;
        " ".repeat(offset).Colorizes(vec![])
    }
}

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

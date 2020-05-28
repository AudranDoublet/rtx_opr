use std::collections::{BTreeMap, VecDeque};
use std::convert::Into;
use std::iter::once;
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, List, Paragraph, Text},
    Terminal,
};

pub struct TermiDrawer {
    messages: VecDeque<String>,
    messages_max_nb: usize,

    ignore: bool,

    variables: BTreeMap<String, String>,

    fps_range: std::ops::Range<f64>,
    fps_track: Vec<(f64, f64)>,
}

impl TermiDrawer {
    pub fn new(messages_max_nb: usize, ignore: bool) -> Self {
        TermiDrawer {
            messages: VecDeque::new(),
            messages_max_nb,

            ignore,

            variables: BTreeMap::new(),
            fps_track: Vec::new(),
            fps_range: std::f64::MAX..std::f64::MIN,
        }
    }

    pub fn log(&mut self, msg: String) {
        if self.ignore {
            return;
        }
        self.messages.push_back(msg);
        if self.messages.len() > self.messages_max_nb {
            self.messages.pop_front();
        }
    }

    fn fps_data(&self) -> ([f64; 2], [f64; 2], &[(f64, f64)]) {
        let x_boundaries = [0., self.fps_track.len() as f64];
        let y_boundaries = [self.fps_range.start, self.fps_range.end];

        (x_boundaries, y_boundaries, self.fps_track.as_slice().into())
    }

    pub fn update_var(&mut self, name: String, value: String) {
        if self.ignore {
            return;
        }
        self.variables.insert(name, value);
    }

    pub fn update_fps(&mut self, fps: f64) {
        if self.ignore {
            return;
        }

        let idx = self.fps_track.len();
        self.fps_range.start = self.fps_range.start.min((fps - fps * 0.01).floor());
        self.fps_range.end = self.fps_range.end.max((fps + fps * 0.01).ceil());
        self.fps_track.push((idx as f64, fps));
    }

    pub fn draw<W: std::io::Write>(
        &mut self,
        terminal: &mut Terminal<TermionBackend<W>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.ignore {
            return Ok(());
        }

        terminal.draw(|mut f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(size);

            {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                    .split(chunks[0]);

                let text: Vec<Text> = self
                    .variables
                    .iter()
                    .map(|(key, value)| {
                        (
                            Text::styled(key, Style::default().modifier(Modifier::BOLD)),
                            Text::raw(format!(" {}\n", value)),
                        )
                    })
                    .flat_map(|tup| once(tup.0).chain(once(tup.1)))
                    .collect();

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title_style(Style::default().modifier(Modifier::BOLD));

                let paragraph = Paragraph::new(text.iter())
                    .block(block.clone().title("Debug Variables"))
                    .alignment(Alignment::Left)
                    .wrap(true);
                f.render_widget(paragraph, chunks[0]);

                let logs = List::new(self.messages.iter().map(|s| Text::raw(format!("{}", s))))
                    .block(Block::default().borders(Borders::ALL).title("Logs"));

                f.render_widget(logs, chunks[1]);
            }

            let (x_bounds, y_bounds, fps_data) = self.fps_data();

            let x_labels = [
                format!("{}", x_bounds[0]),
                format!("{}", x_bounds[0] + (x_bounds[1] + x_bounds[0]) / 2.0),
                format!("{}", x_bounds[1]),
            ];

            let y_labels = [
                format!("{}", y_bounds[0]),
                format!("{}", y_bounds[0] + (y_bounds[1] - y_bounds[0]) / 2.0),
                format!("{}", y_bounds[1]),
            ];

            let datasets = [Dataset::default()
                .name("fps")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .graph_type(GraphType::Line)
                .data(fps_data)];

            let chart = Chart::default()
                .block(
                    Block::default()
                        .title("FPS Tracking")
                        .title_style(Style::default().fg(Color::Cyan).modifier(Modifier::BOLD))
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .title("X Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::ITALIC))
                        .bounds(x_bounds)
                        .labels(&x_labels),
                )
                .y_axis(
                    Axis::default()
                        .title("Y Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels_style(Style::default().modifier(Modifier::ITALIC))
                        .bounds(y_bounds)
                        .labels(&y_labels),
                )
                .datasets(&datasets);
            f.render_widget(chart, chunks[1]);
        })?;

        Ok(())
    }
}

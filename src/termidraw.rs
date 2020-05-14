use std::collections::HashMap;
use std::convert::Into;
use std::iter::once;
use tui::{
    backend::TermionBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Text},
    Terminal,
};

pub struct TermiDrawer {
    variables: HashMap<String, String>,

    fps_range: std::ops::Range<f64>,
    fps_track: Vec<(f64, f64)>,
}

impl TermiDrawer {
    pub fn new() -> Self {
        TermiDrawer {
            variables: HashMap::new(),
            fps_track: Vec::new(),
            fps_range: std::f64::MAX..std::f64::MIN,
        }
    }

    fn fps_data(&self) -> ([f64; 2], [f64; 2], &[(f64, f64)]) {
        let x_boundaries = [0., self.fps_track.len() as f64];
        let y_boundaries = [self.fps_range.start, self.fps_range.end];

        (x_boundaries, y_boundaries, self.fps_track.as_slice().into())
    }

    pub fn update_var(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }

    pub fn update_fps(&mut self, fps: f64) {
        let idx = self.fps_track.len();
        self.fps_range.start = self.fps_range.start.min((fps - fps * 0.01).floor());
        self.fps_range.end = self.fps_range.end.max((fps + fps * 0.01).ceil());
        self.fps_track.push((idx as f64, fps));
    }

    pub fn draw<W: std::io::Write>(
        &mut self,
        terminal: &mut Terminal<TermionBackend<W>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        terminal.draw(|mut f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(size);

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

            let (x_bounds, y_bounds, fps_data) = self.fps_data();

            let x_labels = [
                format!("{}", x_bounds[0]),
                format!("{}", (x_bounds[1] - x_bounds[0]) / 2.0),
                format!("{}", x_bounds[1]),
            ];

            let y_labels = [
                format!("{}", y_bounds[0]),
                format!("{}", (y_bounds[1] - y_bounds[0]) / 2.0),
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

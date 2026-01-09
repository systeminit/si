use std::{
    io,
    time::{
        Duration,
        Instant,
    },
};

use color_eyre::Result;
use crossterm::{
    ExecutableCommand,
    terminal::{
        EnterAlternateScreen,
        LeaveAlternateScreen,
        disable_raw_mode,
        enable_raw_mode,
    },
};
use ratatui::{
    Frame,
    Terminal,
    backend::CrosstermBackend,
    layout::{
        Alignment,
        Constraint,
        Direction,
        Layout,
        Rect,
    },
    style::{
        Color,
        Modifier,
        Style,
    },
    text::{
        Line,
        Span,
        Text,
    },
    widgets::Paragraph,
};
use tokio_util::sync::CancellationToken;

const SI_LOG: &str = include_str!("si_logo.txt");
const APP_NAME: &str = "anansi k(n)ows about nodes in system initiative";

pub struct SplashScreen {
    start_time: Instant,
    frame_count: u32,
    sparkles: Vec<Sparkle>,
}

struct Sparkle {
    x: f32,
    y: f32,
    lifetime: f32,
    max_lifetime: f32,
    char: char,
}

impl Sparkle {
    fn new(x: f32, y: f32) -> Self {
        let chars = ['✨', '⭐', '✦', '✧', '★', '☆', '·', '•'];
        Self {
            x,
            y,
            lifetime: 0.0,
            max_lifetime: 1.5 + rand_float() * 1.5,
            char: chars[(rand_float() * chars.len() as f32) as usize],
        }
    }

    fn update(&mut self, dt: f32) {
        self.lifetime += dt;
    }

    fn is_alive(&self) -> bool {
        self.lifetime < self.max_lifetime
    }

    fn alpha(&self) -> f32 {
        let progress = self.lifetime / self.max_lifetime;
        if progress < 0.2 {
            progress * 5.0
        } else if progress > 0.8 {
            (1.0 - progress) * 5.0
        } else {
            1.0
        }
    }
}

fn rand_float() -> f32 {
    // Simple pseudo-random using system time
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    ((nanos % 1000) as f32) / 1000.0
}

impl SplashScreen {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            frame_count: 0,
            sparkles: Vec::new(),
        }
    }

    pub async fn show(&mut self, cancel_token: CancellationToken, long_wait: bool) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let mut last_frame = Instant::now();
        let mut cancelled = false;
        let at_least_frames = if long_wait { 120 } else { 30 };

        loop {
            let now = Instant::now();
            let dt = now.duration_since(last_frame).as_secs_f32();
            last_frame = now;

            // Update sparkles
            self.update_sparkles(dt);

            // Draw the splash screen
            terminal.draw(|f| self.render(f))?;

            self.frame_count += 1;

            if !cancelled {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        cancelled = true;
                    }
                    _ = tokio::time::sleep(Duration::from_millis(16)) => {}
                }
            } else {
                // Keep spinning for minimum of 30 "frames"
                if self.frame_count < at_least_frames {
                    tokio::time::sleep(Duration::from_millis(16)).await;
                } else {
                    break;
                }
            }
        }

        // Restore terminal
        terminal.clear()?;
        disable_raw_mode()?;
        terminal.backend_mut().execute(LeaveAlternateScreen)?;

        Ok(())
    }

    fn update_sparkles(&mut self, dt: f32) {
        // Update existing sparkles
        for sparkle in &mut self.sparkles {
            sparkle.update(dt);
        }

        // Remove dead sparkles
        self.sparkles.retain(|s| s.is_alive());

        // Add new sparkles occasionally
        if self.frame_count % 3 == 0 && self.sparkles.len() < 20 {
            let x = rand_float() * 100.0;
            let y = rand_float() * 50.0;
            self.sparkles.push(Sparkle::new(x, y));
        }
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Render background sparkles first
        self.render_sparkles(frame, area);

        // Create vertical layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20), // Top spacing
                Constraint::Length(42),     // Logo height
                Constraint::Length(5),      // App name with spacing
                Constraint::Percentage(20), // Bottom spacing
            ])
            .split(area);

        self.render_logo(frame, chunks[1]);
        self.render_app_name(frame, chunks[2]);
    }

    fn render_sparkles(&self, frame: &mut Frame, area: Rect) {
        for sparkle in &self.sparkles {
            if sparkle.x < area.width as f32 && sparkle.y < area.height as f32 {
                let alpha = sparkle.alpha();
                let brightness = (alpha * 255.0) as u8;

                let style = Style::default().fg(Color::Rgb(brightness, brightness, brightness));

                let sparkle_widget = Paragraph::new(sparkle.char.to_string()).style(style);

                let sparkle_area = Rect {
                    x: area.x + sparkle.x as u16,
                    y: area.y + sparkle.y as u16,
                    width: 1,
                    height: 1,
                };

                frame.render_widget(sparkle_widget, sparkle_area);
            }
        }
    }

    fn render_logo(&self, frame: &mut Frame, area: Rect) {
        let logo_lines: Vec<&str> = SI_LOG.lines().collect();
        let logo_width = logo_lines.iter().map(|line| line.len()).max().unwrap_or(0) as u16;
        let logo_height = logo_lines.len() as u16;

        // Center the logo both horizontally and vertically
        let centered_area = {
            let h_padding = area.width.saturating_sub(logo_width) / 2;
            let v_padding = area.height.saturating_sub(logo_height) / 2;
            Rect {
                x: area.x + h_padding,
                y: area.y + v_padding,
                width: logo_width.min(area.width),
                height: logo_height.min(area.height),
            }
        };

        // Create multiple wave effects for more dramatic shimmer
        let time = self.frame_count as f32 / 10.0;
        let wave1 = time.sin();
        let wave2 = (time * 1.5).cos();
        let wave3 = (time * 0.7).sin();

        let mut styled_lines = Vec::new();

        for (line_idx, line) in logo_lines.iter().enumerate() {
            let mut spans = Vec::new();

            for (char_idx, ch) in line.chars().enumerate() {
                let color = if ch == '@' {
                    self.get_dramatic_shimmer_color(char_idx, line_idx, wave1, wave2, wave3)
                } else {
                    Color::Black
                };

                let style = if ch == '@' && self.should_glow(char_idx, line_idx) {
                    Style::default().fg(color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(color)
                };

                spans.push(Span::styled(ch.to_string(), style));
            }

            styled_lines.push(Line::from(spans));
        }

        let logo_text = Text::from(styled_lines);
        let logo_widget = Paragraph::new(logo_text);

        frame.render_widget(logo_widget, centered_area);
    }

    fn render_app_name(&self, frame: &mut Frame, area: Rect) {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        let fade_in = ((elapsed - 0.8) * 2.0).clamp(0.0, 1.0);

        let brightness = (fade_in * 180.0) as u8;

        let subtitle_widget = Paragraph::new(APP_NAME)
            .style(Style::default().fg(Color::Rgb(brightness, brightness, brightness)))
            .alignment(Alignment::Center);

        frame.render_widget(subtitle_widget, area);
    }

    fn get_dramatic_shimmer_color(
        &self,
        x: usize,
        y: usize,
        wave1: f32,
        wave2: f32,
        wave3: f32,
    ) -> Color {
        // Create complex interference patterns
        let distance_from_center = ((x as f32 - 40.0).powi(2) + (y as f32 - 20.0).powi(2)).sqrt();
        let radial_wave = (distance_from_center * 0.1 - self.frame_count as f32 * 0.05).sin();

        let horizontal_wave = ((x as f32 * 0.1) + wave1 * 5.0).sin();
        let vertical_wave = ((y as f32 * 0.2) + wave2 * 3.0).cos();
        let diagonal_wave = (((x + y) as f32 * 0.15) + wave3 * 4.0).sin();

        // Combine waves for complex shimmer
        let combined =
            (radial_wave * 0.3 + horizontal_wave * 0.3 + vertical_wave * 0.2 + diagonal_wave * 0.2)
                * 0.5
                + 0.5;

        // Map to color gradient: deep blue -> cyan -> white -> cyan -> deep blue
        let color_position = combined;

        if color_position < 0.2 {
            Color::Rgb(0, 50, 100) // Deep blue
        } else if color_position < 0.4 {
            let t = (color_position - 0.2) * 5.0;
            Color::Rgb(
                (50.0 * t) as u8,
                (50.0 + 100.0 * t) as u8,
                (100.0 + 100.0 * t) as u8,
            )
        } else if color_position < 0.6 {
            let t = (color_position - 0.4) * 5.0;
            Color::Rgb(
                (50.0 + 205.0 * t) as u8,
                (150.0 + 105.0 * t) as u8,
                (200.0 + 55.0 * t) as u8,
            )
        } else if color_position < 0.8 {
            let t = (color_position - 0.6) * 5.0;
            Color::Rgb((255.0 - 205.0 * t) as u8, (255.0 - 105.0 * t) as u8, 255)
        } else {
            let t = (color_position - 0.8) * 5.0;
            Color::Rgb(
                (50.0 - 50.0 * t) as u8,
                (150.0 - 100.0 * t) as u8,
                (255.0 - 155.0 * t) as u8,
            )
        }
    }

    fn should_glow(&self, x: usize, y: usize) -> bool {
        // Add occasional bright glows
        let glow_pattern = (x * 7 + y * 13 + self.frame_count as usize / 10) % 100;
        glow_pattern < 5
    }
}

/// Display the splash screen
pub async fn show_splash(cancel_token: CancellationToken, long_wait: bool) -> Result<()> {
    let mut splash = SplashScreen::new();
    splash.show(cancel_token, long_wait).await?;
    Ok(())
}

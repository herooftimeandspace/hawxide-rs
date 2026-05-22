use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

use crate::game::{
    BackgroundKind, BackgroundObject, Game, HAWK, Obstacle, Playfield, Sprite, background_origin,
    background_sprite_for, obstacle_origin, sprite_for, sprite_origin_for_lane,
};

const MAX_PLAYFIELD_WIDTH: u16 = 96;
const MAX_PLAYFIELD_HEIGHT: u16 = 36;
const MIN_PLAYFIELD_WIDTH: u16 = 70;
const MIN_PLAYFIELD_HEIGHT: u16 = 36;

pub struct GameWidget<'a> {
    game: &'a Game,
}

impl<'a> GameWidget<'a> {
    pub fn new(game: &'a Game) -> Self {
        Self { game }
    }

    pub fn playfield_from_area(area: Rect) -> Playfield {
        let playfield = Self::content_area(area);
        Playfield {
            width: playfield.width,
            height: playfield.height,
        }
    }

    fn frame_area(area: Rect) -> Rect {
        let width = area.width.min(MAX_PLAYFIELD_WIDTH.saturating_add(2));
        let height = area.height.min(MAX_PLAYFIELD_HEIGHT.saturating_add(2));

        Rect {
            x: area.x + area.width.saturating_sub(width) / 2,
            y: area.y + area.height.saturating_sub(height) / 2,
            width,
            height,
        }
    }

    fn content_area(area: Rect) -> Rect {
        let frame = Self::frame_area(area);
        Rect {
            x: frame.x.saturating_add(1),
            y: frame.y.saturating_add(1),
            width: frame.width.saturating_sub(2),
            height: frame.height.saturating_sub(2),
        }
    }
}

impl Widget for GameWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let frame = Self::frame_area(area);
        let block = Block::default()
            .title(" hawxide-rs ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray));
        let content = block.inner(frame);
        block.render(frame, buf);

        if content.width < MIN_PLAYFIELD_WIDTH || content.height < MIN_PLAYFIELD_HEIGHT {
            buf.set_string(
                content.x,
                content.y,
                "Terminal too small for hawxide-rs; resize to at least 72x38",
                Style::default().fg(Color::Yellow),
            );
            return;
        }

        let inner = Self::content_area(area);

        if !self.game.started() {
            draw_splash(inner, buf);
            return;
        }

        let playfield = Playfield {
            width: inner.width,
            height: inner.height,
        };

        for object in self.game.background_objects() {
            draw_background_object(inner, object, playfield, buf);
        }
        draw_score(inner, self.game, buf);
        draw_hawk(
            inner,
            &HAWK,
            sprite_origin_for_lane(&HAWK, self.game.player_lane(), playfield),
            buf,
        );

        for obstacle in self.game.obstacles() {
            draw_obstacle(inner, obstacle, playfield, buf);
        }

        if self.game.is_game_over() {
            let message = " GAME OVER - r restart / q quit ";
            let x = inner.x + inner.width.saturating_sub(message.len() as u16) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_string(x, y, message, Style::default().fg(Color::Red));
        } else {
            buf.set_string(
                inner.x + 1,
                inner.y,
                "Up/e rise  Down/d dive  q quit",
                Style::default().fg(Color::DarkGray),
            );
        }
    }
}

fn draw_splash(area: Rect, buf: &mut Buffer) {
    let title = [
        "H   H  AAAAA  W   W X   X IIIII DDDD  EEEEE",
        "H   H A     A W   W  X X    I   D   D E    ",
        "HHHHH AAAAAAA W W W   X     I   D   D EEEE ",
        "H   H A     A WW WW  X X    I   D   D E    ",
        "H   H A     A W   W X   X IIIII DDDD  EEEEE",
    ];

    let title_y = area.y + area.height.saturating_sub(10) / 2;
    for (offset, line) in title.iter().enumerate() {
        draw_centered(
            area,
            title_y + offset as u16,
            line,
            Style::default().fg(Color::Red),
            buf,
        );
    }

    draw_centered(
        area,
        title_y + title.len() as u16 + 2,
        "Press any key to start",
        Style::default().fg(Color::Yellow),
        buf,
    );
    draw_centered(
        area,
        title_y + title.len() as u16 + 4,
        "Up arrow or e: rise one lane",
        Style::default().fg(Color::Gray),
        buf,
    );
    draw_centered(
        area,
        title_y + title.len() as u16 + 5,
        "Down arrow or d: dive one lane",
        Style::default().fg(Color::Gray),
        buf,
    );
    draw_centered(
        area,
        title_y + title.len() as u16 + 6,
        "Avoid Oxide racks. q quits during play.",
        Style::default().fg(Color::DarkGray),
        buf,
    );
}

fn draw_centered(area: Rect, y: u16, text: &str, style: Style, buf: &mut Buffer) {
    if y >= area.y + area.height {
        return;
    }

    let x = area.x + area.width.saturating_sub(text.len() as u16) / 2;
    buf.set_string(x, y, text, style);
}

fn draw_score(area: Rect, game: &Game, buf: &mut Buffer) {
    let score = format!("Score: {}", game.score());
    let x = area.x + area.width.saturating_sub(score.len() as u16 + 1);
    buf.set_string(x, area.y, score, Style::default().fg(Color::Green));
}

fn draw_obstacle(area: Rect, obstacle: &Obstacle, playfield: Playfield, buf: &mut Buffer) {
    let sprite = sprite_for(obstacle.kind);
    draw_sprite_at(
        area,
        sprite,
        obstacle_origin(obstacle, playfield),
        buf,
        |ch| match ch {
            '▓' | '░' | '▌' => Style::default().fg(Color::LightGreen),
            '╔' | '╗' | '╚' | '╝' | '═' | '║' | '╠' | '╣' => {
                Style::default().fg(Color::Gray)
            }
            '╭' | '╮' | '╰' | '╯' | '╱' | '╲' | '╳' => {
                Style::default().fg(Color::Gray)
            }
            '┌' | '┐' | '└' | '┘' | '┬' | '┴' | '─' | '│' => {
                Style::default().fg(Color::Gray)
            }
            '○' => Style::default().fg(Color::DarkGray),
            'T' | 'R' | 'U' | 'C' | 'K' => Style::default().fg(Color::DarkGray),
            _ => Style::default().fg(Color::White),
        },
    );
}

fn draw_background_object(
    area: Rect,
    object: &BackgroundObject,
    playfield: Playfield,
    buf: &mut Buffer,
) {
    let sprite = background_sprite_for(object.kind);
    let style_for = |ch| match object.kind {
        BackgroundKind::Cloud => Style::default().fg(Color::LightBlue),
        BackgroundKind::Tree => match ch {
            '|' => Style::default().fg(Color::Rgb(139, 69, 19)),
            _ => Style::default().fg(Color::Rgb(0, 100, 0)),
        },
    };
    draw_sprite_at(
        area,
        sprite,
        background_origin(object, playfield),
        buf,
        style_for,
    );
}

fn draw_hawk(area: Rect, sprite: &Sprite, origin: (i16, u16), buf: &mut Buffer) {
    draw_sprite_at(area, sprite, origin, buf, |ch| match ch {
        'r' => Style::default().fg(Color::Rgb(203, 103, 32)),
        '@' => Style::default().fg(Color::Rgb(147, 100, 62)),
        '%' => Style::default().fg(Color::Rgb(71, 48, 34)),
        '#' => Style::default().fg(Color::Rgb(36, 25, 20)),
        '=' => Style::default().fg(Color::Rgb(221, 185, 130)),
        '+' => Style::default().fg(Color::Rgb(245, 204, 44)),
        '>' => Style::default().fg(Color::Rgb(38, 28, 24)),
        _ => Style::default().fg(Color::Rgb(221, 185, 130)),
    });
}

fn draw_sprite_at<F>(
    area: Rect,
    sprite: &Sprite,
    origin: (i16, u16),
    buf: &mut Buffer,
    style_for: F,
) where
    F: Fn(char) -> Style,
{
    for (dx, dy, ch) in sprite.occupied_cells() {
        let x = origin.0 + dx;
        let y = origin.1 + dy;
        if x < 0 || y >= area.height {
            continue;
        }

        let screen_x = area.x + x as u16;
        let screen_y = area.y + y;
        if screen_x < area.x + area.width && screen_y < area.y + area.height {
            buf[(screen_x, screen_y)]
                .set_char(ch)
                .set_style(style_for(ch));
        }
    }
}

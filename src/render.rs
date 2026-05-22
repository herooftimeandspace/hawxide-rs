use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::game::{
    BackgroundKind, BackgroundObject, Game, HAWK, Lane, Obstacle, Playfield, Sprite,
    background_origin, background_sprite_for, lane_band, obstacle_origin, sprite_for,
    sprite_origin_for_lane,
};

pub struct GameWidget<'a> {
    game: &'a Game,
}

impl<'a> GameWidget<'a> {
    pub fn new(game: &'a Game) -> Self {
        Self { game }
    }

    pub fn playfield_from_area(area: Rect) -> Playfield {
        Playfield {
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        }
    }
}

impl Widget for GameWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        draw_ascii_border(area, buf);
        let inner = Rect {
            x: area.x.saturating_add(1),
            y: area.y.saturating_add(1),
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        if inner.width < 30 || inner.height < 12 {
            buf.set_string(
                inner.x,
                inner.y,
                "Terminal too small for hawxide-rs",
                Style::default().fg(Color::Yellow),
            );
            return;
        }

        let playfield = Playfield {
            width: inner.width,
            height: inner.height,
        };

        draw_lane_guides(inner, playfield, buf);
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

fn draw_ascii_border(area: Rect, buf: &mut Buffer) {
    if area.width < 2 || area.height < 2 {
        return;
    }

    let border_style = Style::default().fg(Color::Gray);
    let left = area.x;
    let right = area.x + area.width - 1;
    let top = area.y;
    let bottom = area.y + area.height - 1;

    for x in left..=right {
        buf[(x, top)].set_char('-').set_style(border_style);
        buf[(x, bottom)].set_char('-').set_style(border_style);
    }
    for y in top..=bottom {
        buf[(left, y)].set_char('|').set_style(border_style);
        buf[(right, y)].set_char('|').set_style(border_style);
    }

    buf[(left, top)].set_char('+').set_style(border_style);
    buf[(right, top)].set_char('+').set_style(border_style);
    buf[(left, bottom)].set_char('+').set_style(border_style);
    buf[(right, bottom)].set_char('+').set_style(border_style);

    if area.width > 14 {
        buf.set_string(left + 2, top, " hawxide-rs ", border_style);
    }
}

fn draw_lane_guides(area: Rect, playfield: Playfield, buf: &mut Buffer) {
    for lane in [Lane::High, Lane::Middle, Lane::Low] {
        let band = lane_band(lane, playfield);
        if band.top > 0 {
            let y = area.y + band.top;
            for x in area.x..area.x + area.width {
                buf[(x, y)]
                    .set_char('-')
                    .set_style(Style::default().fg(Color::DarkGray));
            }
        }
    }
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
        |_| Style::default().fg(Color::White),
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
        'r' | '>' => Style::default().fg(Color::Red),
        '=' | '^' => Style::default().fg(Color::Rgb(139, 69, 19)),
        'h' | 'a' | 'w' | 'k' => Style::default().fg(Color::White),
        _ => Style::default().fg(Color::Yellow),
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

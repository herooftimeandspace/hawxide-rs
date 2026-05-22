use std::time::Duration;

pub const PLAYER_X: i16 = 8;
pub const MIN_SPAWN_GAP: i16 = 28;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lane {
    High,
    Middle,
    Low,
}

impl Lane {
    fn move_up(self) -> Self {
        match self {
            Self::High => Self::High,
            Self::Middle => Self::High,
            Self::Low => Self::Middle,
        }
    }

    fn move_down(self) -> Self {
        match self {
            Self::High => Self::Middle,
            Self::Middle => Self::Low,
            Self::Low => Self::Low,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MoveDirection {
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObstacleKind {
    Low,
    Truck,
    Parachute,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackgroundKind {
    Cloud,
    Tree,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sprite {
    pub lines: &'static [&'static str],
}

impl Sprite {
    pub fn width(&self) -> i16 {
        self.lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0)
            .try_into()
            .unwrap_or(0)
    }

    pub fn height(&self) -> u16 {
        self.lines.len().try_into().unwrap_or(0)
    }

    pub fn occupied_cells(&self) -> impl Iterator<Item = (i16, u16, char)> + '_ {
        self.lines.iter().enumerate().flat_map(|(y, line)| {
            line.chars()
                .enumerate()
                .filter(|(_, ch)| *ch != ' ')
                .map(move |(x, ch)| (x as i16, y as u16, ch))
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Obstacle {
    pub kind: ObstacleKind,
    pub x: i16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackgroundObject {
    pub kind: BackgroundKind,
    pub x: i16,
}

#[derive(Clone, Debug)]
pub struct Game {
    started: bool,
    player_lane: Lane,
    obstacles: Vec<Obstacle>,
    background_objects: Vec<BackgroundObject>,
    next_obstacle: usize,
    score_elapsed: Duration,
    scroll_progress: f64,
    background_progress: f64,
    game_over: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Playfield {
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Band {
    pub top: u16,
    pub bottom: u16,
}

pub static HAWK: Sprite = Sprite {
    lines: &["  /^^\\  ", "<=hawk=>", "  \\__r> "],
};

pub static LOW_RACK: Sprite = Sprite {
    lines: &["[OXIDE-RACK]", "[_________]"],
};

pub static TRUCK_RACK: Sprite = Sprite {
    lines: &[
        " [OXIDE]",
        " [RACK] ",
        " ______________ ",
        "|__TRUCK______|",
        "  O        O   ",
    ],
};

pub static PARACHUTE_RACK: Sprite = Sprite {
    lines: &[
        "   ___   ",
        " /     \\ ",
        "/_______\\",
        "  \\ | /  ",
        " [OXIDE] ",
        " [RACK]  ",
    ],
};

pub static CLOUD: Sprite = Sprite {
    lines: &["   .--.   ", " .(    ). ", "(___.__)_)"],
};

pub static TREE: Sprite = Sprite {
    lines: &["  /\\  ", " /**\\ ", "/****\\", "  ||  ", "  ||  "],
};

impl Game {
    pub fn new() -> Self {
        Self {
            started: false,
            player_lane: Lane::Middle,
            obstacles: Vec::new(),
            background_objects: Vec::new(),
            next_obstacle: 0,
            score_elapsed: Duration::ZERO,
            scroll_progress: 0.0,
            background_progress: 0.0,
            game_over: false,
        }
    }

    pub fn started(&self) -> bool {
        self.started
    }

    pub fn start(&mut self) {
        self.started = true;
    }

    pub fn player_lane(&self) -> Lane {
        self.player_lane
    }

    pub fn obstacles(&self) -> &[Obstacle] {
        &self.obstacles
    }

    pub fn background_objects(&self) -> &[BackgroundObject] {
        &self.background_objects
    }

    pub fn score(&self) -> u64 {
        self.score_elapsed.as_secs()
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn move_player(&mut self, direction: MoveDirection) {
        if !self.started || self.game_over {
            return;
        }

        self.player_lane = match direction {
            MoveDirection::Up => self.player_lane.move_up(),
            MoveDirection::Down => self.player_lane.move_down(),
        };
    }

    pub fn tick(&mut self, elapsed: Duration, playfield: Playfield) {
        if !self.started || self.game_over {
            return;
        }

        self.score_elapsed += elapsed;
        self.scroll_progress += elapsed.as_secs_f64() * self.speed_cells_per_second();
        self.background_progress += elapsed.as_secs_f64() * self.background_cells_per_second();

        let steps = self.scroll_progress.floor() as i16;
        if steps > 0 {
            self.scroll_progress -= f64::from(steps);
            for obstacle in &mut self.obstacles {
                obstacle.x -= steps;
            }
            self.obstacles
                .retain(|obstacle| obstacle.x + sprite_for(obstacle.kind).width() >= 0);
        }

        let background_steps = self.background_progress.floor() as i16;
        if background_steps > 0 {
            self.background_progress -= f64::from(background_steps);
            for object in &mut self.background_objects {
                object.x -= background_steps;
            }
            self.background_objects
                .retain(|object| object.x + background_sprite_for(object.kind).width() >= 0);
        }

        self.spawn_background_if_ready(playfield.width);
        self.spawn_if_ready(playfield.width);
        self.apply_collision(playfield);
    }

    pub fn detect_collision(&self, playfield: Playfield) -> bool {
        let hawk_origin = sprite_origin_for_lane(&HAWK, self.player_lane, playfield);
        let hawk_cells: Vec<(i16, u16)> = HAWK
            .occupied_cells()
            .map(|(x, y, _)| (hawk_origin.0 + x, hawk_origin.1 + y))
            .collect();

        self.obstacles.iter().any(|obstacle| {
            let sprite = sprite_for(obstacle.kind);
            let origin = obstacle_origin(obstacle, playfield);
            sprite.occupied_cells().any(|(x, y, _)| {
                let cell = (origin.0 + x, origin.1 + y);
                hawk_cells.contains(&cell)
            })
        })
    }

    pub fn apply_collision(&mut self, playfield: Playfield) {
        if self.detect_collision(playfield) {
            self.game_over = true;
        }
    }

    #[cfg(test)]
    pub fn force_obstacle_for_test(&mut self, kind: ObstacleKind, x: i16) {
        self.obstacles.push(Obstacle { kind, x });
    }

    fn spawn_if_ready(&mut self, viewport_width: u16) {
        if viewport_width == 0 {
            return;
        }

        let rightmost_edge = self
            .obstacles
            .iter()
            .map(|obstacle| obstacle.x + sprite_for(obstacle.kind).width())
            .max();

        if rightmost_edge.is_some_and(|edge| edge > viewport_width as i16 - MIN_SPAWN_GAP) {
            return;
        }

        let kind = match self.next_obstacle % 3 {
            0 => ObstacleKind::Low,
            1 => ObstacleKind::Truck,
            _ => ObstacleKind::Parachute,
        };
        self.next_obstacle += 1;
        self.obstacles.push(Obstacle {
            kind,
            x: viewport_width as i16,
        });
    }

    fn spawn_background_if_ready(&mut self, viewport_width: u16) {
        if viewport_width == 0 {
            return;
        }

        let rightmost_edge = self
            .background_objects
            .iter()
            .map(|object| object.x + background_sprite_for(object.kind).width())
            .max();

        if rightmost_edge.is_some_and(|edge| edge > viewport_width as i16 - 18) {
            return;
        }

        let kind = if self.background_objects.len().is_multiple_of(2) {
            BackgroundKind::Cloud
        } else {
            BackgroundKind::Tree
        };
        self.background_objects.push(BackgroundObject {
            kind,
            x: viewport_width as i16,
        });
    }

    fn speed_cells_per_second(&self) -> f64 {
        14.0 + (self.score() / 20) as f64
    }

    fn background_cells_per_second(&self) -> f64 {
        4.0
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

pub fn sprite_for(kind: ObstacleKind) -> &'static Sprite {
    match kind {
        ObstacleKind::Low => &LOW_RACK,
        ObstacleKind::Truck => &TRUCK_RACK,
        ObstacleKind::Parachute => &PARACHUTE_RACK,
    }
}

pub fn background_sprite_for(kind: BackgroundKind) -> &'static Sprite {
    match kind {
        BackgroundKind::Cloud => &CLOUD,
        BackgroundKind::Tree => &TREE,
    }
}

pub fn lane_band(lane: Lane, playfield: Playfield) -> Band {
    let lane_height = (playfield.height / 3).max(1);
    let high_top = 0;
    let middle_top = lane_height;
    let low_top = lane_height * 2;

    match lane {
        Lane::High => Band {
            top: high_top,
            bottom: middle_top,
        },
        Lane::Middle => Band {
            top: middle_top,
            bottom: low_top,
        },
        Lane::Low => Band {
            top: low_top,
            bottom: playfield.height,
        },
    }
}

pub fn obstacle_band(kind: ObstacleKind, playfield: Playfield) -> Band {
    match kind {
        ObstacleKind::Low => lane_band(Lane::Low, playfield),
        ObstacleKind::Truck => Band {
            top: lane_band(Lane::Middle, playfield).top,
            bottom: lane_band(Lane::Low, playfield).bottom,
        },
        ObstacleKind::Parachute => Band {
            top: lane_band(Lane::High, playfield).top,
            bottom: lane_band(Lane::Middle, playfield).bottom,
        },
    }
}

pub fn sprite_origin_for_lane(sprite: &Sprite, lane: Lane, playfield: Playfield) -> (i16, u16) {
    let band = lane_band(lane, playfield);
    let available_height = band.bottom.saturating_sub(band.top);
    let y_offset = available_height.saturating_sub(sprite.height()) / 2;
    (PLAYER_X, band.top + y_offset)
}

pub fn obstacle_origin(obstacle: &Obstacle, playfield: Playfield) -> (i16, u16) {
    let sprite = sprite_for(obstacle.kind);
    let band = obstacle_band(obstacle.kind, playfield);
    let available_height = band.bottom.saturating_sub(band.top);
    let y = match obstacle.kind {
        ObstacleKind::Low => band.top + available_height.saturating_sub(sprite.height()) / 2,
        ObstacleKind::Truck => band.top,
        ObstacleKind::Parachute => band.top,
    };
    (obstacle.x, y)
}

pub fn background_origin(object: &BackgroundObject, playfield: Playfield) -> (i16, u16) {
    let sprite = background_sprite_for(object.kind);
    let y = match object.kind {
        BackgroundKind::Cloud => {
            let high_band = lane_band(Lane::High, playfield);
            high_band.top + high_band.bottom.saturating_sub(high_band.top) / 3
        }
        BackgroundKind::Tree => playfield.height.saturating_sub(sprite.height()),
    };
    (object.x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FIELD: Playfield = Playfield {
        width: 80,
        height: 24,
    };

    #[test]
    fn player_starts_in_middle_lane() {
        assert_eq!(Game::new().player_lane(), Lane::Middle);
    }

    #[test]
    fn game_waits_on_splash_until_started() {
        let mut game = Game::new();

        assert!(!game.started());
        game.tick(Duration::from_secs(3), TEST_FIELD);

        assert_eq!(game.score(), 0);
        assert!(game.obstacles().is_empty());
    }

    #[test]
    fn start_allows_gameplay_ticks() {
        let mut game = Game::new();

        game.start();
        game.tick(Duration::from_secs(1), TEST_FIELD);

        assert!(game.started());
        assert_eq!(game.score(), 1);
        assert_eq!(game.obstacles().len(), 1);
    }

    #[test]
    fn movement_steps_one_lane_and_respects_boundaries() {
        let mut game = Game::new();
        game.start();

        game.move_player(MoveDirection::Up);
        assert_eq!(game.player_lane(), Lane::High);
        game.move_player(MoveDirection::Up);
        assert_eq!(game.player_lane(), Lane::High);

        game.move_player(MoveDirection::Down);
        assert_eq!(game.player_lane(), Lane::Middle);
        game.move_player(MoveDirection::Down);
        assert_eq!(game.player_lane(), Lane::Low);
        game.move_player(MoveDirection::Down);
        assert_eq!(game.player_lane(), Lane::Low);
    }

    #[test]
    fn obstacle_bands_match_their_allowed_vertical_space() {
        assert_eq!(obstacle_band(ObstacleKind::Low, TEST_FIELD).top, 16);
        assert_eq!(obstacle_band(ObstacleKind::Low, TEST_FIELD).bottom, 24);
        assert_eq!(obstacle_band(ObstacleKind::Truck, TEST_FIELD).top, 8);
        assert_eq!(obstacle_band(ObstacleKind::Truck, TEST_FIELD).bottom, 24);
        assert_eq!(obstacle_band(ObstacleKind::Parachute, TEST_FIELD).top, 0);
        assert_eq!(
            obstacle_band(ObstacleKind::Parachute, TEST_FIELD).bottom,
            16
        );
    }

    #[test]
    fn sprites_fit_inside_their_bands() {
        for kind in [
            ObstacleKind::Low,
            ObstacleKind::Truck,
            ObstacleKind::Parachute,
        ] {
            let sprite = sprite_for(kind);
            let band = obstacle_band(kind, TEST_FIELD);
            assert!(sprite.height() <= band.bottom - band.top);
        }
    }

    #[test]
    fn scoring_counts_survived_seconds() {
        let mut game = Game::new();
        game.start();

        game.tick(Duration::from_millis(999), TEST_FIELD);
        assert_eq!(game.score(), 0);
        game.tick(Duration::from_millis(1), TEST_FIELD);
        assert_eq!(game.score(), 1);
    }

    #[test]
    fn spawn_spacing_preserves_navigation_gap() {
        let mut game = Game::new();
        game.start();

        game.tick(Duration::from_millis(1), TEST_FIELD);
        assert_eq!(game.obstacles().len(), 1);
        game.tick(Duration::from_millis(1), TEST_FIELD);
        assert_eq!(game.obstacles().len(), 1);

        let first = &game.obstacles()[0];
        assert!(first.x + sprite_for(first.kind).width() > TEST_FIELD.width as i16 - MIN_SPAWN_GAP);
    }

    #[test]
    fn background_objects_do_not_trigger_collision() {
        let mut game = Game::new();
        game.start();

        game.tick(Duration::from_millis(1), TEST_FIELD);
        assert_eq!(game.background_objects().len(), 1);
        assert!(!game.detect_collision(TEST_FIELD));
    }

    #[test]
    fn visible_cell_overlap_collides() {
        let mut game = Game::new();
        game.force_obstacle_for_test(ObstacleKind::Truck, PLAYER_X - 1);

        assert!(game.detect_collision(TEST_FIELD));
    }

    #[test]
    fn transparent_sprite_space_does_not_collide() {
        let mut game = Game::new();
        game.force_obstacle_for_test(ObstacleKind::Truck, PLAYER_X + 10);

        assert!(!game.detect_collision(TEST_FIELD));
    }

    #[test]
    fn collision_sets_game_over_and_freezes_movement() {
        let mut game = Game::new();
        game.start();
        game.force_obstacle_for_test(ObstacleKind::Truck, PLAYER_X - 1);

        game.apply_collision(TEST_FIELD);
        assert!(game.is_game_over());

        game.move_player(MoveDirection::Up);
        assert_eq!(game.player_lane(), Lane::Middle);
    }
}

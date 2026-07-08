use macroquad::prelude::*;
use macroquad::rand::gen_range;

const WIDTH: f32 = 1100.0;
const HEIGHT: f32 = 700.0;
const SHIP_RADIUS: f32 = 13.0;
const BULLET_SPEED: f32 = 530.0;
const MAX_BULLETS: usize = 6;

fn window_conf() -> Conf {
    Conf {
        window_title: "Rust Asteroids".to_owned(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Title,
    Playing,
    Paused,
    GameOver,
}

#[derive(Clone, Copy)]
enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    fn radius(self) -> f32 {
        match self {
            Self::Large => 47.0,
            Self::Medium => 28.0,
            Self::Small => 16.0,
        }
    }

    fn score(self) -> u32 {
        match self {
            Self::Large => 20,
            Self::Medium => 50,
            Self::Small => 100,
        }
    }

    fn child(self) -> Option<Self> {
        match self {
            Self::Large => Some(Self::Medium),
            Self::Medium => Some(Self::Small),
            Self::Small => None,
        }
    }
}

struct Ship {
    pos: Vec2,
    vel: Vec2,
    angle: f32,
    shot_cooldown: f32,
    invulnerable: f32,
}

impl Ship {
    fn new() -> Self {
        Self {
            pos: vec2(WIDTH / 2.0, HEIGHT / 2.0),
            vel: Vec2::ZERO,
            angle: -std::f32::consts::FRAC_PI_2,
            shot_cooldown: 0.0,
            invulnerable: 2.2,
        }
    }

    fn nose(&self) -> Vec2 {
        self.pos + direction(self.angle) * 18.0
    }
}

struct Bullet {
    pos: Vec2,
    vel: Vec2,
    life: f32,
}

struct Asteroid {
    pos: Vec2,
    vel: Vec2,
    size: AsteroidSize,
    angle: f32,
    spin: f32,
    shape: Vec<f32>,
}

impl Asteroid {
    fn new(pos: Vec2, size: AsteroidSize, speed_bonus: f32) -> Self {
        let heading = gen_range(0.0, std::f32::consts::TAU);
        let speed = gen_range(34.0, 72.0) + speed_bonus;
        let points = gen_range(9, 14);
        let mut shape = Vec::with_capacity(points);
        for _ in 0..points {
            shape.push(gen_range(0.72, 1.12));
        }
        Self {
            pos,
            vel: direction(heading) * speed,
            size,
            angle: gen_range(0.0, std::f32::consts::TAU),
            spin: gen_range(-0.65, 0.65),
            shape,
        }
    }
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: f32,
    max_life: f32,
}

struct Star {
    pos: Vec2,
    brightness: f32,
    size: f32,
}

struct Game {
    screen: Screen,
    ship: Ship,
    bullets: Vec<Bullet>,
    asteroids: Vec<Asteroid>,
    particles: Vec<Particle>,
    stars: Vec<Star>,
    score: u32,
    high_score: u32,
    lives: i32,
    wave: u32,
    wave_delay: f32,
}

impl Game {
    fn new() -> Self {
        let stars = (0..115)
            .map(|_| Star {
                pos: vec2(gen_range(0.0, WIDTH), gen_range(0.0, HEIGHT)),
                brightness: gen_range(0.18, 0.62),
                size: gen_range(0.7, 1.8),
            })
            .collect();
        Self {
            screen: Screen::Title,
            ship: Ship::new(),
            bullets: Vec::new(),
            asteroids: Vec::new(),
            particles: Vec::new(),
            stars,
            score: 0,
            high_score: 0,
            lives: 3,
            wave: 0,
            wave_delay: 0.0,
        }
    }

    fn start(&mut self) {
        self.ship = Ship::new();
        self.bullets.clear();
        self.asteroids.clear();
        self.particles.clear();
        self.score = 0;
        self.lives = 3;
        self.wave = 0;
        self.wave_delay = 0.2;
        self.screen = Screen::Playing;
    }

    fn next_wave(&mut self) {
        self.wave += 1;
        let count = 3 + self.wave.min(7) as usize;
        for _ in 0..count {
            let pos = self.safe_edge_spawn();
            self.asteroids.push(Asteroid::new(
                pos,
                AsteroidSize::Large,
                self.wave as f32 * 3.0,
            ));
        }
    }

    fn safe_edge_spawn(&self) -> Vec2 {
        match gen_range(0, 4) {
            0 => vec2(gen_range(0.0, WIDTH), -45.0),
            1 => vec2(WIDTH + 45.0, gen_range(0.0, HEIGHT)),
            2 => vec2(gen_range(0.0, WIDTH), HEIGHT + 45.0),
            _ => vec2(-45.0, gen_range(0.0, HEIGHT)),
        }
    }

    fn update(&mut self, dt: f32) {
        match self.screen {
            Screen::Title | Screen::GameOver => {
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.start();
                }
                self.update_particles(dt);
                return;
            }
            Screen::Paused => {
                if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
                    self.screen = Screen::Playing;
                }
                return;
            }
            Screen::Playing => {}
        }

        if is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
            self.screen = Screen::Paused;
            return;
        }

        self.update_ship(dt);
        self.update_entities(dt);
        self.handle_bullet_collisions();
        self.handle_ship_collision();
        self.update_particles(dt);

        if self.asteroids.is_empty() {
            self.wave_delay -= dt;
            if self.wave_delay <= 0.0 {
                self.next_wave();
            }
        }
    }

    fn update_ship(&mut self, dt: f32) {
        let turning = if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            -1.0
        } else if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            1.0
        } else {
            0.0
        };
        self.ship.angle += turning * 3.6 * dt;

        let thrusting = is_key_down(KeyCode::Up) || is_key_down(KeyCode::W);
        if thrusting {
            self.ship.vel += direction(self.ship.angle) * 235.0 * dt;
            if gen_range(0.0, 1.0) > 0.35 {
                let rear = self.ship.pos - direction(self.ship.angle) * 12.0;
                self.particles.push(Particle {
                    pos: rear,
                    vel: -direction(self.ship.angle) * gen_range(60.0, 130.0)
                        + vec2(gen_range(-20.0, 20.0), gen_range(-20.0, 20.0)),
                    life: 0.25,
                    max_life: 0.25,
                });
            }
        }

        self.ship.vel *= 0.995_f32.powf(dt * 60.0);
        if self.ship.vel.length() > 330.0 {
            self.ship.vel = self.ship.vel.normalize() * 330.0;
        }
        self.ship.pos += self.ship.vel * dt;
        wrap_position(&mut self.ship.pos, SHIP_RADIUS);
        self.ship.shot_cooldown = (self.ship.shot_cooldown - dt).max(0.0);
        self.ship.invulnerable = (self.ship.invulnerable - dt).max(0.0);

        if is_key_down(KeyCode::Space)
            && self.ship.shot_cooldown <= 0.0
            && self.bullets.len() < MAX_BULLETS
        {
            let dir = direction(self.ship.angle);
            self.bullets.push(Bullet {
                pos: self.ship.nose(),
                vel: self.ship.vel + dir * BULLET_SPEED,
                life: 1.15,
            });
            self.ship.shot_cooldown = 0.17;
        }
    }

    fn update_entities(&mut self, dt: f32) {
        for bullet in &mut self.bullets {
            bullet.pos += bullet.vel * dt;
            wrap_position(&mut bullet.pos, 2.0);
            bullet.life -= dt;
        }
        self.bullets.retain(|bullet| bullet.life > 0.0);

        for asteroid in &mut self.asteroids {
            asteroid.pos += asteroid.vel * dt;
            asteroid.angle += asteroid.spin * dt;
            wrap_position(&mut asteroid.pos, asteroid.size.radius());
        }
    }

    fn handle_bullet_collisions(&mut self) {
        let mut bullet_index = self.bullets.len();
        while bullet_index > 0 {
            bullet_index -= 1;
            let hit = self.asteroids.iter().position(|asteroid| {
                wrapped_distance(self.bullets[bullet_index].pos, asteroid.pos)
                    < asteroid.size.radius()
            });
            if let Some(asteroid_index) = hit {
                let bullet = self.bullets.swap_remove(bullet_index);
                let asteroid = self.asteroids.swap_remove(asteroid_index);
                self.score += asteroid.size.score();
                self.high_score = self.high_score.max(self.score);
                self.explode(asteroid.pos, asteroid.size.radius(), 12);

                if let Some(child_size) = asteroid.size.child() {
                    for offset in [-0.65, 0.65] {
                        let mut child =
                            Asteroid::new(asteroid.pos, child_size, self.wave as f32 * 3.0);
                        let base = bullet.vel.y.atan2(bullet.vel.x) + offset;
                        child.vel = direction(base) * (child.vel.length() + 20.0);
                        self.asteroids.push(child);
                    }
                }
            }
        }
    }

    fn handle_ship_collision(&mut self) {
        if self.ship.invulnerable > 0.0 {
            return;
        }
        let collision = self.asteroids.iter().any(|asteroid| {
            wrapped_distance(self.ship.pos, asteroid.pos)
                < SHIP_RADIUS + asteroid.size.radius() * 0.78
        });
        if collision {
            self.explode(self.ship.pos, 35.0, 28);
            self.lives -= 1;
            self.bullets.clear();
            if self.lives <= 0 {
                self.screen = Screen::GameOver;
            } else {
                self.ship = Ship::new();
            }
        }
    }

    fn explode(&mut self, pos: Vec2, speed: f32, count: usize) {
        for _ in 0..count {
            let life = gen_range(0.35, 0.85);
            self.particles.push(Particle {
                pos,
                vel: direction(gen_range(0.0, std::f32::consts::TAU))
                    * gen_range(speed * 0.7, speed * 2.5),
                life,
                max_life: life,
            });
        }
    }

    fn update_particles(&mut self, dt: f32) {
        for particle in &mut self.particles {
            particle.pos += particle.vel * dt;
            particle.vel *= 0.98_f32.powf(dt * 60.0);
            particle.life -= dt;
        }
        self.particles.retain(|particle| particle.life > 0.0);
    }

    fn draw(&self) {
        clear_background(Color::from_rgba(4, 8, 16, 255));
        self.draw_background();
        for asteroid in &self.asteroids {
            draw_asteroid(asteroid);
        }
        for bullet in &self.bullets {
            draw_circle(
                bullet.pos.x,
                bullet.pos.y,
                2.2,
                Color::new(0.75, 1.0, 0.94, 1.0),
            );
        }
        for particle in &self.particles {
            let alpha = (particle.life / particle.max_life).clamp(0.0, 1.0);
            draw_circle(
                particle.pos.x,
                particle.pos.y,
                1.6,
                Color::new(0.42, 0.95, 0.8, alpha),
            );
        }

        if matches!(self.screen, Screen::Playing | Screen::Paused) {
            if self.ship.invulnerable <= 0.0 || ((self.ship.invulnerable * 8.0) as i32 % 2 == 0) {
                draw_ship(
                    &self.ship,
                    is_key_down(KeyCode::Up) || is_key_down(KeyCode::W),
                );
            }
            self.draw_hud();
        }

        match self.screen {
            Screen::Title => {
                centered_text(
                    "RUST ASTEROIDS",
                    HEIGHT * 0.35,
                    58,
                    Color::new(0.45, 1.0, 0.88, 1.0),
                );
                centered_text("A VECTOR-STYLE ARCADE RENDEZVOUS", HEIGHT * 0.44, 20, GRAY);
                centered_text("PRESS ENTER OR SPACE TO START", HEIGHT * 0.58, 24, WHITE);
                centered_text(
                    "W / ↑  THRUST    A D / ← →  TURN    SPACE  FIRE",
                    HEIGHT * 0.68,
                    18,
                    LIGHTGRAY,
                );
                centered_text("P / ESC  PAUSE", HEIGHT * 0.73, 18, LIGHTGRAY);
            }
            Screen::Paused => {
                draw_rectangle(0.0, 0.0, WIDTH, HEIGHT, Color::new(0.0, 0.0, 0.0, 0.56));
                centered_text("PAUSED", HEIGHT * 0.46, 48, WHITE);
                centered_text("PRESS P OR ESC TO RESUME", HEIGHT * 0.54, 20, LIGHTGRAY);
            }
            Screen::GameOver => {
                centered_text(
                    "GAME OVER",
                    HEIGHT * 0.39,
                    56,
                    Color::new(1.0, 0.43, 0.43, 1.0),
                );
                centered_text(
                    &format!("FINAL SCORE  {:06}", self.score),
                    HEIGHT * 0.50,
                    26,
                    WHITE,
                );
                centered_text(
                    "PRESS ENTER OR SPACE TO PLAY AGAIN",
                    HEIGHT * 0.61,
                    20,
                    LIGHTGRAY,
                );
            }
            Screen::Playing => {}
        }
    }

    fn draw_background(&self) {
        for star in &self.stars {
            draw_circle(
                star.pos.x,
                star.pos.y,
                star.size,
                Color::new(0.55, 0.75, 0.82, star.brightness),
            );
        }
        draw_rectangle_lines(
            1.0,
            1.0,
            WIDTH - 2.0,
            HEIGHT - 2.0,
            1.0,
            Color::new(0.1, 0.3, 0.34, 0.5),
        );
    }

    fn draw_hud(&self) {
        draw_text(format!("SCORE  {:06}", self.score), 28.0, 38.0, 25.0, WHITE);
        let high = format!("HIGH  {:06}", self.high_score);
        let width = measure_text(&high, None, 25, 1.0).width;
        draw_text(&high, WIDTH / 2.0 - width / 2.0, 38.0, 25.0, LIGHTGRAY);
        draw_text(
            format!("WAVE  {}", self.wave),
            WIDTH - 132.0,
            38.0,
            25.0,
            WHITE,
        );

        for i in 0..self.lives {
            draw_mini_ship(vec2(38.0 + i as f32 * 27.0, 66.0));
        }
    }
}

fn direction(angle: f32) -> Vec2 {
    vec2(angle.cos(), angle.sin())
}

fn wrap_position(pos: &mut Vec2, margin: f32) {
    if pos.x < -margin {
        pos.x = WIDTH + margin;
    } else if pos.x > WIDTH + margin {
        pos.x = -margin;
    }
    if pos.y < -margin {
        pos.y = HEIGHT + margin;
    } else if pos.y > HEIGHT + margin {
        pos.y = -margin;
    }
}

fn wrapped_distance(a: Vec2, b: Vec2) -> f32 {
    let dx = (a.x - b.x).abs().min(WIDTH - (a.x - b.x).abs());
    let dy = (a.y - b.y).abs().min(HEIGHT - (a.y - b.y).abs());
    vec2(dx, dy).length()
}

fn draw_ship(ship: &Ship, thrusting: bool) {
    let forward = direction(ship.angle);
    let side = direction(ship.angle + std::f32::consts::FRAC_PI_2);
    let nose = ship.pos + forward * 18.0;
    let left = ship.pos - forward * 13.0 + side * 11.0;
    let notch = ship.pos - forward * 7.0;
    let right = ship.pos - forward * 13.0 - side * 11.0;
    let color = Color::new(0.65, 1.0, 0.92, 1.0);
    draw_line(nose.x, nose.y, left.x, left.y, 2.0, color);
    draw_line(left.x, left.y, notch.x, notch.y, 2.0, color);
    draw_line(notch.x, notch.y, right.x, right.y, 2.0, color);
    draw_line(right.x, right.y, nose.x, nose.y, 2.0, color);

    if thrusting {
        let flame = ship.pos - forward * gen_range(20.0, 28.0);
        let flame_left = ship.pos - forward * 11.0 + side * 5.0;
        let flame_right = ship.pos - forward * 11.0 - side * 5.0;
        draw_line(flame_left.x, flame_left.y, flame.x, flame.y, 1.6, ORANGE);
        draw_line(flame.x, flame.y, flame_right.x, flame_right.y, 1.6, ORANGE);
    }
}

fn draw_mini_ship(pos: Vec2) {
    let color = Color::new(0.65, 1.0, 0.92, 1.0);
    draw_triangle_lines(
        pos + vec2(0.0, -9.0),
        pos + vec2(-7.0, 8.0),
        pos + vec2(7.0, 8.0),
        1.5,
        color,
    );
}

fn draw_asteroid(asteroid: &Asteroid) {
    let count = asteroid.shape.len();
    let mut previous = asteroid_vertex(asteroid, count - 1);
    for index in 0..count {
        let current = asteroid_vertex(asteroid, index);
        draw_line(
            previous.x,
            previous.y,
            current.x,
            current.y,
            2.0,
            Color::new(0.72, 0.8, 0.8, 1.0),
        );
        previous = current;
    }
}

fn asteroid_vertex(asteroid: &Asteroid, index: usize) -> Vec2 {
    let angle = asteroid.angle + index as f32 / asteroid.shape.len() as f32 * std::f32::consts::TAU;
    asteroid.pos + direction(angle) * asteroid.size.radius() * asteroid.shape[index]
}

fn centered_text(text: &str, y: f32, size: u16, color: Color) {
    let dimensions = measure_text(text, None, size, 1.0);
    draw_text(
        text,
        WIDTH / 2.0 - dimensions.width / 2.0,
        y,
        size as f32,
        color,
    );
}

fn draw_letterbox() {
    let scale = (screen_width() / WIDTH).min(screen_height() / HEIGHT);
    let viewport_w = WIDTH * scale;
    let viewport_h = HEIGHT * scale;
    let x = (screen_width() - viewport_w) / 2.0;
    let y = (screen_height() - viewport_h) / 2.0;
    set_camera(&Camera2D {
        zoom: vec2(2.0 / WIDTH, -2.0 / HEIGHT),
        target: vec2(WIDTH / 2.0, HEIGHT / 2.0),
        viewport: Some((x as i32, y as i32, viewport_w as i32, viewport_h as i32)),
        ..Default::default()
    });
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();
    loop {
        draw_letterbox();
        game.update(get_frame_time().min(0.05));
        game.draw();
        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_has_unit_length() {
        assert!((direction(1.234).length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn wrap_moves_objects_to_opposite_edge() {
        let mut pos = vec2(-11.0, HEIGHT + 11.0);
        wrap_position(&mut pos, 10.0);
        assert_eq!(pos, vec2(WIDTH + 10.0, -10.0));
    }

    #[test]
    fn asteroid_progression_is_correct() {
        assert!(matches!(
            AsteroidSize::Large.child(),
            Some(AsteroidSize::Medium)
        ));
        assert!(matches!(
            AsteroidSize::Medium.child(),
            Some(AsteroidSize::Small)
        ));
        assert!(AsteroidSize::Small.child().is_none());
    }
}

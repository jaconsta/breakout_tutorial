use macroquad::prelude::*;

const BALL_SIZE: f32 = 50f32;
const BALL_SPEED: f32 = 400f32;
const BLOCK_SIZE: Vec2 = Vec2::from_array([100f32, 40f32]);
const PLAYER_SIZE: Vec2 = Vec2::from_array([150f32, 40f32]); // width, height
const PLAYER_SPEED: f32 = 700f32;

pub fn draw_title_text(text: &str, font: Font) {
    let font_size = 50u16;
    let dimmension = measure_text(&text, Some(font), font_size, 1.0);
    draw_text_ex(
        text,
        screen_width() * 0.5f32 - dimmension.width * 0.5f32,
        screen_height() * 0.5f32 - dimmension.height * 0.5f32,
        TextParams {
            font,
            font_size,
            color: BLACK,
            ..Default::default()
        },
    );
}

pub enum GameState {
    Menu,
    Game,
    LevelCompleted,
    Dead,
}

struct Player {
    rect: Rect,
}
impl Player {
    pub fn new() -> Self {
        Self {
            rect: Rect::new(
                screen_width() * 0.5f32 - PLAYER_SIZE.x * 0.5f32, // Middle of of the screen
                screen_height() - 100f32,                         // almost bottom of the screen
                PLAYER_SIZE.x,
                PLAYER_SIZE.y,
            ),
        }
    }

    pub fn update(&mut self, dt: f32) {
        let (pressed_left, pressed_right) =
            (is_key_down(KeyCode::Left), is_key_down(KeyCode::Right));
        let x_move = match (pressed_left, pressed_right) {
            (true, false) => -1f32,
            (false, true) => 1f32,
            _ => 0f32,
        };
        self.rect.x += x_move * dt * PLAYER_SPEED;

        // Collision detection
        if self.rect.x < 0f32 {
            // Hit left wall
            self.rect.x = 0f32;
        } else if self.rect.x > screen_width() - self.rect.w {
            // Hit right wall
            self.rect.x = screen_width() - self.rect.w;
        }
    }

    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, BLUE);
    }
}

#[derive(PartialEq)]
pub enum BlockType {
    Regular,
    SpawnBallOnDeath,
}
struct Block {
    rect: Rect,
    lives: i32,
    block_type: BlockType,
}

impl Block {
    pub fn new(pos: Vec2, block_type: BlockType) -> Self {
        Self {
            rect: Rect::new(pos.x, pos.y, BLOCK_SIZE.x, BLOCK_SIZE.y),
            lives: 1,
            block_type,
        }
    }

    pub fn draw(&self) {
        let color = match self.block_type {
            BlockType::Regular => match self.lives {
                2 => RED,
                _ => ORANGE,
            },
            BlockType::SpawnBallOnDeath => GREEN,
        };
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, color);
    }
}

pub struct Ball {
    rect: Rect,
    vel: Vec2,
}

impl Ball {
    pub fn new(pos: Vec2) -> Self {
        Self {
            rect: Rect::new(pos.x, pos.y, BALL_SIZE, BALL_SIZE),
            vel: Vec2::new(rand::gen_range(-1f32, 1f32), 1f32).normalize(),
        }
    }
    pub fn update(&mut self, dt: f32) {
        self.rect.x += self.vel.x * dt * BALL_SPEED;
        self.rect.y += self.vel.y * dt * BALL_SPEED;

        // Collision detection
        if self.rect.x < 0f32 {
            self.vel.x = 1f32;
        } else if self.rect.x > screen_width() - self.rect.w {
            self.vel.x = -1f32;
        } else if self.rect.y < 0f32 {
            self.vel.y = 1f32;
        }
    }
    pub fn draw(&self) {
        draw_rectangle(self.rect.x, self.rect.y, self.rect.w, self.rect.h, DARKGRAY);
    }
}

fn resolve_collision(a: &mut Rect, vel: &mut Vec2, b: &Rect) -> bool {
    let intersection = match a.intersect(*b) {
        Some(intersection) => intersection,
        None => return false,
    };
    // Vector from a to b
    // let a_center = a.point() + a.size() * 0.5f32;
    let to = b.center() - a.center();
    // And arrow that determines the direction the vector is pointing towards
    let to_signum = to.signum();
    // Which side of the intersection h or w has mayor contact point
    // Aka on which direction did the collision took place?
    match intersection.w > intersection.h {
        true => {
            // Bounce on y
            a.y -= to_signum.y * intersection.h;
            vel.y = -to_signum.y * vel.y.abs();
        }
        false => {
            // Bounce on x
            a.x -= to_signum.x * intersection.w;
            vel.x = -to_signum.x * vel.x.abs();
        }
    };
    true
}

fn reset_game(
    score: &mut i32,
    player_lives: &mut i32,
    blocks: &mut Vec<Block>,
    balls: &mut Vec<Ball>,
    player: &mut Player,
) {
    *player = Player::new();
    *score = 0;
    *player_lives = 3;
    balls.clear();
    blocks.clear();
    init_blocks(blocks);
}
fn init_blocks(blocks: &mut Vec<Block>) {
    let (width, height) = (6, 6);
    let padding = 5f32;
    let total_block_size = BLOCK_SIZE + vec2(padding, padding);
    let board_start_pos = vec2(
        (screen_width() - (total_block_size.x * width as f32)) * 0.5f32, // (Total screen size - total block size ) / half a block
        50f32,                                                           // just 50px  down
    );
    for i in 0..width * height {
        // (1 % width) -< 0,1,2,3,4,5,0,1,2,3,4...
        let block_x = (i % width) as f32 * total_block_size.x;
        let block_y = (i / width) as f32 * total_block_size.y;
        blocks.push(Block::new(
            board_start_pos + vec2(block_x, block_y),
            BlockType::Regular,
        ));
    }
    // Make a few of them special so that they are SpawnBallOnDeath
    for _ in 0..3 {
        let rand_index = rand::gen_range(0, blocks.len());
        blocks[rand_index].block_type = BlockType::SpawnBallOnDeath;
    }
}

#[macroquad::main("breakout")]
async fn main() {
    let font = load_ttf_font("res/Heebo-VariableFont_wght.ttf")
        .await
        .unwrap();
    let mut game_state = GameState::Menu;
    let mut score = 0;
    let mut player_lives = 3;

    let mut player = Player::new();
    let mut blocks = Vec::new();
    let mut balls = Vec::new();
    init_blocks(&mut blocks);

    balls.push(Ball::new(vec2(
        (screen_width() - BALL_SIZE) * 0.5f32,
        screen_height() * 0.5f32 + 100f32,
    )));

    loop {
        match game_state {
            GameState::Menu => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Game;
                }
            }
            GameState::Game => {
                player.update(get_frame_time());
                balls
                    .iter_mut()
                    .for_each(|ball| ball.update(get_frame_time()));
                let mut spawn_ball_later = vec![];
                balls.iter_mut().for_each(|ball| {
                    resolve_collision(&mut ball.rect, &mut ball.vel, &player.rect);
                    blocks.iter_mut().for_each(|block| {
                        if resolve_collision(&mut ball.rect, &mut ball.vel, &block.rect) {
                            block.lives -= 1;
                            if block.lives <= 0 {
                                score += 10;
                                if block.block_type == BlockType::SpawnBallOnDeath {
                                    spawn_ball_later.push(Ball::new(ball.rect.point()));
                                }
                            }
                        };
                    });
                });
                spawn_ball_later
                    .into_iter()
                    .for_each(|ball| balls.push(ball));
                let balls_len = balls.len();
                balls.retain(|ball| ball.rect.y < screen_height() - 100f32); // Retain balls above the player
                if balls_len > balls.len() && balls.is_empty() {
                    player_lives -= 1;
                    balls.push(Ball::new(
                        player.rect.point() + vec2((player.rect.w - BALL_SIZE) * 0.5f32, -50f32),
                    )); // Right above the player
                    if player_lives <= 0 {
                        game_state = GameState::Dead;
                    }
                }
                blocks.retain(|block| block.lives > 0);
                if blocks.is_empty() {
                    game_state = GameState::LevelCompleted;
                }
            }
            GameState::LevelCompleted | GameState::Dead => {
                if is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Menu;
                    reset_game(
                        &mut score,
                        &mut player_lives,
                        &mut blocks,
                        &mut balls,
                        &mut player,
                    );

                    balls.push(Ball::new(
                        player.rect.point() + vec2((player.rect.w - BALL_SIZE) * 0.5f32, -50f32),
                    ));
                }
            }
        };

        clear_background(WHITE);

        player.draw();
        blocks.iter().for_each(|block| block.draw());
        balls.iter().for_each(|ball| ball.draw());

        match game_state {
            GameState::Menu => {
                draw_title_text("Press SPACE to start", font);
            }
            GameState::Game => {
                let score_text = format!("score: {}", score);
                let font_size = 30u16;
                let score_text_dim = measure_text(&score_text, Some(font), font_size, 1.0);
                draw_text_ex(
                    &score_text,
                    screen_width() * 0.5f32 - score_text_dim.width * 0.5f32,
                    40.0,
                    TextParams {
                        font,
                        font_size,
                        color: BLACK,
                        ..Default::default()
                    },
                );
                draw_text_ex(
                    &format!("lives: {}", player_lives),
                    30.0,
                    40.0,
                    TextParams {
                        font,
                        font_size,
                        color: BLACK,
                        ..Default::default()
                    },
                );
            }
            GameState::LevelCompleted => {
                draw_title_text(&format!("You win! {} Score", score), font);
            }
            GameState::Dead => {
                draw_title_text(&format!("You loose! {} Score", score), font);
            }
        };
        next_frame().await
    }
}

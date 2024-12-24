use anyhow::bail;
use itertools::Itertools;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use hashbrown::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Tile {
    Empty,
    Cookie,
    Milk,
    Wall,
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum Team {
    #[serde(rename = "cookie")]
    Cookie,
    #[serde(rename = "milk")]
    Milk,
}

pub struct Board {
    pub grid: HashMap<(usize, usize), Tile>,
}

impl Default for Board {
    fn default() -> Self {
        let mut grid = HashMap::new();

        (0..4).for_each(|y| {
            (1..5).for_each(|x| {
                grid.insert((x, y), Tile::Empty);
            });
        });

        (0..5).for_each(|y| {
            grid.insert((0, y), Tile::Wall);
            grid.insert((5, y), Tile::Wall);
        });
        (1..5).for_each(|x| {
            grid.insert((x, 4), Tile::Wall);
        });

        Self { grid }
    }
}

impl Board {
    pub fn reset(&mut self) {
        let new_board = Board::default();
        self.grid = new_board.grid;
    }
    pub fn display(&self) -> String {
        let mut result: String = (0..5)
            .map(|y| {
                (0..6)
                    .map(|x| match self.grid.get(&(x, y)) {
                        Some(Tile::Wall) => '⬜',
                        Some(Tile::Empty) => '⬛',
                        Some(Tile::Milk) => '🥛',
                        Some(Tile::Cookie) => '🍪',
                        _ => unreachable!("Grid not initialized properly"),
                    })
                    .join("")
            })
            .join("\n");

        result.push('\n');
        if let Some(winning_team) = self.has_winner() {
            let team = match winning_team {
                Team::Cookie => '🍪',
                Team::Milk => '🥛',
            };
            result.push_str(&format!("{team} wins!\n"));
        } else if self.is_full() {
            result.push_str("No winner.\n")
        }

        result
    }

    pub fn has_winner(&self) -> Option<Team> {
        for x in 1..5 {
            let ys = (0..4)
                .map(|y| self.grid.get(&(x, y)).unwrap())
                .copied()
                .collect::<Vec<Tile>>();
            if ys.iter().all(|&p| p == Tile::Cookie) {
                return Some(Team::Cookie);
            } else if ys.iter().all(|&p| p == Tile::Milk) {
                return Some(Team::Milk);
            }
        }
        for y in 0..4 {
            let xs = (1..5)
                .map(|x| self.grid.get(&(x, y)).unwrap())
                .copied()
                .collect::<Vec<Tile>>();
            if xs.iter().all(|&p| p == Tile::Cookie) {
                return Some(Team::Cookie);
            } else if xs.iter().all(|&p| p == Tile::Milk) {
                return Some(Team::Milk);
            }
        }

        let diags1: [(usize, usize); 4] = [(1, 0), (2, 1), (3, 2), (4, 3)];
        let diags2: [(usize, usize); 4] = [(4, 0), (3, 1), (2, 2), (1, 3)];
        let diag1 = diags1
            .iter()
            .map(|c| self.grid.get(c).unwrap())
            .copied()
            .collect::<Vec<Tile>>();
        let diag2 = diags2
            .iter()
            .map(|c| self.grid.get(c).unwrap())
            .copied()
            .collect::<Vec<Tile>>();
        if diag1.iter().all(|&p| p == Tile::Cookie) || diag2.iter().all(|&p| p == Tile::Cookie) {
            return Some(Team::Cookie);
        } else if diag1.iter().all(|&p| p == Tile::Milk) || diag2.iter().all(|&p| p == Tile::Milk) {
            return Some(Team::Milk);
        }

        None
    }
    pub fn is_column_full(&self, column: u8) -> bool {
        !(0..4).any(|y| self.grid.get(&(column as usize, y)) == Some(&Tile::Empty))
    }
    pub fn is_full(&self) -> bool {
        (1..5).all(|x| self.is_column_full(x))
    }

    pub fn place(&mut self, team: Team, column: u8) -> anyhow::Result<()> {
        if self.has_winner().is_some() {
            bail!("has winner");
        }
        if self.is_full() {
            bail!("is full");
        }
        if self.is_column_full(column) {
            bail!("Column is full");
        }

        for y in (0..4).rev() {
            if self.grid.get(&(column as usize, y)) == Some(&Tile::Empty) {
                *self.grid.entry((column as usize, y)).or_insert(Tile::Empty) = match team {
                    Team::Cookie => Tile::Cookie,
                    Team::Milk => Tile::Milk,
                };

                return Ok(());
            }
        }

        Ok(())
    }
}

pub fn day_12_routes() -> Router {
    Router::new()
        .route("/board", get(board))
        .route("/reset", post(reset))
        .route("/place/:team/:column", post(place_item))
        .with_state(Arc::new(Mutex::new(Board::default())))
}

pub async fn board(State(board): State<Arc<Mutex<Board>>>) -> impl IntoResponse {
    board.lock().unwrap().display()
}

pub async fn reset(State(board): State<Arc<Mutex<Board>>>) -> impl IntoResponse {
    let mut board = board.lock().unwrap();
    board.reset();

    board.display()
}

pub async fn place_item(
    Path((team, column)): Path<(Team, u8)>,
    State(board): State<Arc<Mutex<Board>>>,
) -> impl IntoResponse {
    if !(1..=4).contains(&column) {
        return (StatusCode::BAD_REQUEST, "out of range".to_string());
    }
    let mut board = board.lock().unwrap();
    match board.place(team, column) {
        Ok(_) => (StatusCode::OK, board.display()),
        _ => (StatusCode::SERVICE_UNAVAILABLE, board.display()),
    }
}

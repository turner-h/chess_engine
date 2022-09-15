use std::str::FromStr;

use rand::{self, Rng, seq::IteratorRandom};
use chess::{ Board, MoveGen, Color, Square, ChessMove, Piece };
use bevy::prelude::*;

const WIDTH: f32 = 600.0;
const HEIGHT: f32 = 600.0;
const DEPTH: u16 = 4;

const PAWN_EVAL: [[i32; 8]; 8] =  [[0,  0,  0,  0,  0,  0,  0,  0],
                                   [50, 50, 50, 50, 50, 50, 50, 50],
                                   [10, 10, 20, 30, 30, 20, 10, 10],
                                   [5,  5, 10, 25, 25, 10,  5,  5],
                                   [0,  0,  0, 20, 20,  0,  0,  0],
                                   [5, -5,-10,  0,  0,-10, -5,  5],
                                   [5, 10, 10,-20,-20, 10, 10,  5],
                                   [0,  0,  0,  0,  0,  0,  0,  0]];

const KNIGHT_EVAL: [[i32; 8]; 8] = [[-50,-40,-30,-30,-30,-30,-40,-50],
                                    [-40,-20,  0,  0,  0,  0,-20,-40],
                                    [-30,  0, 10, 15, 15, 10,  0,-30],
                                    [-30,  5, 15, 20, 20, 15,  5,-30],
                                    [-30,  0, 15, 20, 20, 15,  0,-30],
                                    [-30,  5, 10, 15, 15, 10,  5,-30],
                                    [-40,-20,  0,  5,  5,  0,-20,-40],
                                    [-50,-40,-30,-30,-30,-30,-40,-50]];

const BISHOP_EVAL: [[i32; 8]; 8] = [[-20,-10,-10,-10,-10,-10,-10,-20],
                                    [-10,  0,  0,  0,  0,  0,  0,-10],
                                    [-10,  0,  5, 10, 10,  5,  0,-10],
                                    [-10,  5,  5, 10, 10,  5,  5,-10],
                                    [-10,  0, 10, 10, 10, 10,  0,-10],
                                    [-10, 10, 10, 10, 10, 10, 10,-10],
                                    [-10,  5,  0,  0,  0,  0,  5,-10],
                                    [-20,-10,-10,-10,-10,-10,-10,-20]];           
                                    
const ROOK_EVAL: [[i32; 8]; 8] =   [[0,  0,  0,  0,  0,  0,  0,  0],
                                    [5, 10, 10, 10, 10, 10, 10,  5],
                                    [-5,  0,  0,  0,  0,  0,  0, -5],
                                    [-5,  0,  0,  0,  0,  0,  0, -5],
                                    [-5,  0,  0,  0,  0,  0,  0, -5],
                                    [-5,  0,  0,  0,  0,  0,  0, -5],
                                    [-5,  0,  0,  0,  0,  0,  0, -5],
                                    [0,  0,  0,  5,  5,  0,  0,  0]];     
                                    
const QUEEN_EVAL: [[i32; 8]; 8] = [[-20,-10,-10, -5, -5,-10,-10,-20],
                                    [-10,  0,  0,  0,  0,  0,  0,-10],
                                    [-10,  0,  5,  5,  5,  5,  0,-10],
                                    [ -5,  0,  5,  5,  5,  5,  0, -5],
                                    [  0,  0,  5,  5,  5,  5,  0, -5],
                                    [-10,  5,  5,  5,  5,  5,  0,-10],
                                    [-10,  0,  5,  0,  0,  0,  0,-10],
                                    [-20,-10,-10, -5, -5,-10,-10,-20]];

const KING_EVAL: [[i32; 8]; 8] = [[-30,-40,-40,-50,-50,-40,-40,-30],
                                    [-30,-40,-40,-50,-50,-40,-40,-30],
                                    [-30,-40,-40,-50,-50,-40,-40,-30],
                                    [-30,-40,-40,-50,-50,-40,-40,-30],
                                    [-20,-30,-30,-40,-40,-30,-30,-20],
                                    [-10,-20,-20,-20,-20,-20,-20,-10],
                                    [ 20, 20,  0,  0,  0,  0, 20, 20],
                                    [ 20, 30, 10,  0,  0, 10, 30, 20]];

#[derive(Default)]
struct Chess {
    board: Board,
    old_board: Board,
    pieces: Vec<Entity>,
    moving: bool,
    starting_square: Square,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("sprites/board.png"),
        transform: Transform::from_scale(Vec3 { x: 0.439, y: 0.439, z: 1.0 }),
        ..default()
    });
}

fn spawn_piece(x: i8, y: i8, piece: String, color: String, commands: &mut Commands, asset_server: &Res<AssetServer>) -> Entity {
    let path = "sprites/".to_string() + &piece + "_" + &color + ".png";
    let piece = commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load(&path),
        transform: Transform::from_xyz(board_to_screen_coords(x, y).0, board_to_screen_coords(x, y).1, 1.0),
        ..default()
    }).id();
    piece
}

fn spawn_board_from_fen_string(fen: &str, commands: &mut Commands, asset_server: &Res<AssetServer>, mut chess: ResMut<Chess>) {    
    for piece in chess.as_mut().pieces.as_slice() {
        commands.entity(*piece).despawn();
    }
    chess.as_mut().pieces.clear();

    let fen_clean: Vec<&str> = fen.split(" ").collect();
    let board: Vec<&str> = fen_clean[0].split("/").collect();
    let mut y = 0;
    for row in board {
        y += 1;
        let mut x = 0;
        for char in row.chars() {
            x += 1;
            match char {
                'p' => chess.as_mut().pieces.insert(0, spawn_piece(x, y, "p".to_string(), "black".to_string(), commands, &asset_server)),
                'n' => chess.as_mut().pieces.insert(0, spawn_piece(x, y, "n".to_string(), "black".to_string(), commands, &asset_server)),
                'r' => chess.as_mut().pieces.insert(0, spawn_piece(x, y, "r".to_string(), "black".to_string(), commands, &asset_server)),
                'b' => chess.as_mut().pieces.insert(0, spawn_piece(x, y, "b".to_string(), "black".to_string(), commands, &asset_server)),
                'k' => chess.as_mut().pieces.insert(0, spawn_piece(x, y, "k".to_string(), "black".to_string(), commands, &asset_server)),
                'q' => chess.as_mut().pieces.insert(0, spawn_piece(x, y, "q".to_string(), "black".to_string(), commands, &asset_server)),
                'P' => chess.as_mut().pieces.insert(0, spawn_piece(x, y, "p".to_string(), "white".to_string(), commands, &asset_server)),
                'R' => chess.as_mut().pieces.insert(0, spawn_piece(x, y, "r".to_string(), "white".to_string(), commands, &asset_server)),
                'N' => chess.as_mut().pieces.insert(0, spawn_piece(x, y, "n".to_string(), "white".to_string(), commands, &asset_server)),
                'B' => chess.as_mut().pieces.insert(0, spawn_piece(x, y, "b".to_string(), "white".to_string(), commands, &asset_server)),
                'K' => chess.as_mut().pieces.insert(0, spawn_piece(x, y, "k".to_string(), "white".to_string(), commands, &asset_server)),
                'Q' => chess.as_mut().pieces.insert(0, spawn_piece(x, y, "q".to_string(), "white".to_string(), commands, &asset_server)),
                _ => x += char.to_digit(10).unwrap() as i8 - 1,
            }
        }
    }
}

fn render_pieces(mut commands: Commands, asset_server: Res<AssetServer>, mut chess: ResMut<Chess>) {
    if chess.board != chess.old_board {
        chess.old_board = chess.board;
        spawn_board_from_fen_string(chess.board.to_string().as_str(), &mut commands, &asset_server, chess);
    }
}

fn rand_moves(mut chess: ResMut<Chess>) {
    if chess.as_ref().board.side_to_move() == Color::White {
        if rand::thread_rng().gen_range(0..24) == rand::thread_rng().gen_range(0..24) {
            let moves = MoveGen::new_legal(&chess.board);
            let tmp = chess.as_mut().board;
            tmp.make_move(moves.choose(&mut rand::thread_rng()).unwrap(), &mut chess.as_mut().board);
            
            //debug
            let moves_dbg = MoveGen::new_legal(&chess.board);
            println!("{}", moves_dbg.choose(&mut rand::thread_rng()).unwrap()); 
        }
    }
}

fn bot_algorithm(mut chess: ResMut<Chess>) {
    if chess.as_ref().board.side_to_move() == Color::White {
        if rand::thread_rng().gen_range(0..24) == rand::thread_rng().gen_range(0..24) {
            let moves = MoveGen::new_legal(&chess.board);
           
            let mut best_move: ChessMove = ChessMove::default();
            let mut best_eval = -2000;

            for chess_move in moves {
                let mut eval_board = chess.as_ref().board;
                chess.as_ref().board.make_move(chess_move, &mut eval_board);
           
                let eval = minmax(DEPTH, eval_board);

                println!("{}, eval: {}", chess_move, eval);

                if eval > best_eval {
                    best_eval = eval;
                    best_move = chess_move;
                }
            }

            let tmp = chess.as_mut().board;
            tmp.make_move(best_move, &mut chess.as_mut().board);
        }
    }
}

fn minmax(mut depth: u16, board: Board) -> i32{
    depth -= 1;

    let moves = MoveGen::new_legal(&board);

    for chess_move in moves {
        let mut eval_board = board;
        board.make_move(chess_move, &mut eval_board);

        let mut eval: i32;

        if depth != 0 {
            eval = minmax(depth, eval_board);
            return eval;
        } else {
            let eval = evaluate_board(eval_board);
            return eval;
        }
    }

    0
}

fn evaluate_board(eval_board: Board) -> i32 {
    let mut eval = 0;
    
    let eval_string = eval_board.to_string();
    let fen_clean: Vec<&str> = eval_string.split(" ").collect();
    let board: Vec<&str> = fen_clean[0].split("/").collect();
    let mut y = 0;
    for row in board {
        y += 1;
        let mut x = 0;
        for char in row.chars() {
            x += 1;
            match char{
                'p' => eval -= PAWN_EVAL[x - 1][y - 1],
                'n' => eval -= KNIGHT_EVAL[x - 1][y - 1],
                'r' => eval -= ROOK_EVAL[x - 1][y - 1],
                'b' => eval -= BISHOP_EVAL[x - 1][y - 1],
                'k' => eval -= KING_EVAL[x - 1][y - 1],
                'q' => eval -= QUEEN_EVAL[x - 1][y - 1],
                'P' => eval += PAWN_EVAL[x - 1][y - 1],
                'R' => eval += KNIGHT_EVAL[x - 1][y - 1],
                'N' => eval += ROOK_EVAL[x - 1][y - 1],
                'B' => eval += BISHOP_EVAL[x - 1][y - 1],
                'K' => eval += KING_EVAL[x - 1][y - 1],
                'Q' => eval += QUEEN_EVAL[x - 1][y - 1],
                _ => ()
            }
        }
    }

    eval
}

fn render_first_pieces(mut commands: Commands, asset_server: Res<AssetServer>, mut chess: ResMut<Chess>) {
    let board = chess.as_mut().board;
    spawn_board_from_fen_string(board.to_string().as_str(), &mut commands, &asset_server, chess);
}

fn board_to_screen_coords(board_x: i8, board_y: i8) -> (f32, f32) {
    let screen_x = ((board_x as f32 * (WIDTH / 8.0)) - (WIDTH / 2.0)) - (WIDTH / 16.0);
    let screen_y = ((board_y as f32 * (HEIGHT / 8.0)) - (HEIGHT / 2.0)) - (HEIGHT / 16.0);
    (screen_x, screen_y)
}

fn screen_to_board_coords(screen_pos: Vec2) -> (i8, i8) {
    let board_x = (((screen_pos[0]) + (WIDTH / 2.0)) / (WIDTH / 8.0) - 3.0) as i8;
    let board_y = (((screen_pos[1]) + (HEIGHT / 2.0)) / (HEIGHT / 8.0) - 3.0) as i8;
    (board_x, board_y)
}

fn coords_to_square(x: i8, y: i8) -> Square {
    let mut square_char = "";
    match x {
        1 => square_char = "a",
        2 => square_char = "b",
        3 => square_char = "c",
        4 => square_char = "d",
        5 => square_char = "e",
        6 => square_char = "f",
        7 => square_char = "g",
        8 => square_char = "h",
        _ => (),
    }
    let square: Square = Square::from_str((square_char.to_owned() + y.to_string().as_str()).as_str()).unwrap();
    square
}

fn player(mouse_input: Res<Input<MouseButton>>, windows: Res<Windows>, mut chess: ResMut<Chess>) {
    let win = windows.get_primary().expect("no primary window");
    if mouse_input.just_pressed(MouseButton::Left) {
        let x = screen_to_board_coords(win.cursor_position().unwrap()).0;
        let y = -(screen_to_board_coords(win.cursor_position().unwrap()).1) + 9;
        println!("click on {}, {}", x, y);

        let square = coords_to_square(x, y);
        
        let piece = chess.as_ref().board.piece_on(square);
        println!("{:?}", piece);

        if !chess.as_ref().moving {
            chess.as_mut().starting_square = square;
            chess.as_mut().moving = true;
        } else {
            let mut promotion: Option<Piece> = None;
            if y == 1 {
                promotion = Some(Piece::Queen);
            }
            
            let chess_move = ChessMove::new(chess.as_ref().starting_square, square, promotion);
            println!("{}", chess.as_ref().board.legal(chess_move));

            if chess.as_ref().board.legal(chess_move) {   
                let tmp = chess.as_mut().board;
                tmp.make_move(chess_move, &mut chess.as_mut().board);
                println!("{}", chess_move);
            }
            chess.as_mut().moving = false;
        }
    }
}

fn main() {
    App::new()
    .insert_resource(WindowDescriptor {
        title: "Chess Engine".to_string(),
        width: WIDTH,
        height: HEIGHT,
        ..Default::default()
    })
    .init_resource::<Chess>()
    .add_startup_system(setup)
    //.add_startup_system(render_first_pieces)
    .add_system(render_pieces)
    .add_system(bot_algorithm)
    //.add_system(rand_moves)
    .add_system(player)
    .add_plugins(DefaultPlugins).run();
}

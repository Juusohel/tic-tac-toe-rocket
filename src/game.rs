use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use rand::Rng;
use crate::game::GameStatus::{DRAW, OWon, XWon};

pub enum GameStatus {
    RUNNING,
    XWon,
    OWon,
    DRAW,
}

pub struct PlayerList {
    pub player_map: Mutex<HashMap<String, char>>
}

pub struct GameList{
    pub list: Mutex<HashMap<String,Game>>
}

#[derive(Clone)]
#[derive(Serialize)]
#[derive(Deserialize)]
pub struct Game {
    /// The game's UUID, read-only, generated by the server. The client can not POST or PUT this.
    id: Option<String>,

    /// The board state
    board: String,

    /// The game status, read-only, the client can not POST or PUT this
    status: Option<String>
}

impl Game {
    /// Creates a new game instance
    /// Checks whether the board is an acceptable starting board and returns and error if not.
    ///
    /// # Parameters
    ///
    /// board - Starting board
    ///
    /// player_list - The application's running list of players and their signs.
    pub fn new(mut board: String, player_list: &PlayerList) -> Result<Game, &'static str> {
        let player_move;
        let mut lock = player_list.player_map.lock().unwrap(); // Bringing player map
        let uuid = Some(Uuid::new_v4().to_string()); // Generating UUID
        let uuid_copy = uuid.clone().unwrap(); // copy for map use, Safely unwrappable


        // Validating board size
        if board.len() != 9 {
            return Err("Unable to create game: invalid board!");
        }
        // Correct characters and count
        let mut x_count= 0;
        let mut o_count = 0;
        for character in board.chars() {
            match character {
                'X' => {
                    x_count += 1;
                    continue
                },
                'O' => {
                    o_count += 1;
                    continue
                },
                '-' => continue,
                _ => return Err("Unable to create game: invalid board!"),
            }
        }
        // Checking if there's a valid number characters to start game
        if ((x_count > 1) || (o_count > 1)) || (x_count == 1 && o_count == 1) {
            return Err("Unable to create game: invalid starting board");
        }

        // If board started empty, make first move
        // Implementing a best move algorithm was out of scope for this so a random slot will be used
        if (x_count == 0) && (o_count == 0) {
            let mut rng = rand::thread_rng();
            let random = rng.gen_range(0..9); // Random number
            let sign_select = rng.gen_range(0..100);
            let first_move;

            // place random sign on random spot
            if (sign_select % 2) == 0 {
                first_move = "O";
                player_move = 'X';
            }
            else {
                first_move = "X";
                player_move = 'O';
            }
            // Making the first move by replacing a random tile with with the random sign.
            board.replace_range(random..random+1, first_move);
        } else if (x_count == 1) && (o_count == 0) {
            player_move = 'X' // If player has placed an X to start
        } else {
            player_move = 'O' // if board is not empty and not X then player placed O
        }


        // Creating game object to be returned
        let game = Game {
            id: uuid,
        status: Some(String::from("RUNNING")),
        board
        };

        // Adding player and game id to map
        let _ = lock.insert(uuid_copy, player_move);

        Ok(game)
    }

    pub fn set_board(&mut self, board: String) {
        self.board = board
    }

    pub fn get_board(&self) -> &String {
        &self.board
    }
    pub fn get_status(&self) -> &Option<String>  {
        &self.status
    }
    pub fn set_status(&mut self, game_status: GameStatus) {
        match game_status {
            GameStatus::RUNNING => self.status = Some(String::from("RUNNING")),
            GameStatus::XWon => self.status = Some(String::from("X_WON")),
            GameStatus::OWon => self.status = Some(String::from("O_WON")),
            GameStatus::DRAW => self.status = Some(String::from("DRAW"))
        }
    }

    pub fn get_id(&self) -> &Option<String> {
        &self.id
    }


    pub fn check_win_conditions(&mut self) -> bool {
        let board_rows: Vec<&str>;
        let current_board = &self.board.clone();
        let row0 = &current_board[0..3];
        let row1 = &current_board[3..6];
        let row2 = &current_board[6..];
        board_rows = vec!(row0, row1, row2);

        // temporary variables for logic use
        let mut win_x: bool = false;
        let mut win_o: bool = false;


        // This is a bit slow but there's no clever way to take the character as an input
        // since the game object stores it as a single string anyway and the function would
        // just have to be duplicated on each type of move function.
        // That and since the board is 9 characters long, the impact is negligible even on low power devices
        // Despite appearing rather convoluted, should only be O(5n)


        // Checking rows for X
        for row in &board_rows {
            win_x = true;
            for char in row.chars() {
                // If all chars are X, win is true and loop won't break
                if char != 'X' {
                    win_x = false;
                    break
                }
            }
            // terminates with a win, X has won, break loop
            if win_x {
                let _ = &self.set_status(XWon);
                return true;
            }
        }

        // Checking rows for O
        for row in &board_rows {
            win_o = true;
            for char in row.chars() {
                // If all chars are O, win is true and loop won't break
                if char != 'O' {
                    win_o = false;
                    break
                }
            }
            // terminates with a win, O has won, break loop
            if win_o {
                let _ = &self.set_status(OWon);
                return true;
            }
        }

        //Checking columns
        // Preparing rows for parallel iteration
        let r0 = row0.chars();
        let r12 = row1.chars().zip(row2.chars());

        // Iterating over all the rows parallel
        for (r0, r12) in r0.zip(r12) {
            let r0_char = r0;
            let (r1_char, r2_char) = r12;

            // If all characters are the same, check which one they are and behave accordingly
            if (r0_char == r1_char) && (r2_char == r0_char)  {
                match r0_char {
                    'X' => {
                        self.set_status(XWon);
                        return true
                    }
                    'O' => {
                        self.set_status(OWon);
                        return true
                    }
                    _ => continue
                }
            }
        }

        // Checking diagonals
        // Grabbing the characters we need to check
        // initializing with a default value and mutable because of rust security, overwritten by loop
        let mut zero = '-';
        let mut two= '-';
        let mut four= '-';
        let mut six= '-';
        let mut eight= '-';
        // Assigning the signs we want to a variable.
        for (i, char) in current_board.chars().enumerate() {
            match i {
                0 => zero = char,
                2 => two = char,
                4 => four = char,
                6 => six = char,
                8 => eight = char,
                _ => continue
            }
        }
        // Comparisons
        // 0 - 4 - 8 Diagonal
        if (zero == eight) && (zero == four){
            match zero {
                'X' => {
                    self.set_status(XWon);
                    return true;
                }
                'O' => {
                    self.set_status(OWon);
                    return true;
                }
                _ => {}
            }
        }
        // 2 - 4 - 8 Diagonal
        if (two == four) && (two == six) {
            match two {
                'X' => {
                    self.set_status(XWon);
                    return true;
                }
                'O' => {
                    self.set_status(OWon);
                    return true;
                }
                _ => {}
            }
        }

        // Finally, if no win conditions are met and the function returned, checking for a draw
        // If no slots are unfilled (-), and previous conditions did not return true, game is draw
        for char in current_board.chars() {
            if char == '-' {
                // no win conditions met, unfilled slot, game still live
                return false;
            }
        }
        // Game has no empty slots and no win conditions have been met
        self.set_status(DRAW);
        true
    }

    /// Accepts move by player, and makes a move in response
    /// Computer will make their own move randomly as implementing best move algorithm was out of scope
    /// for this.
    ///
    /// This function assumes that the front-end won't allow old tiles to be completely overwritten as
    /// tracking individual tiles state when the board is represented by a string is impractical,
    pub fn make_move(&mut self, new_board: String) -> bool{
        // check status running
        // Check player move from the uuid.
        // Count X O and -, make sure there's player sign is +1, - is -1, and non player move is the same
        // when comparing the strings
        // check win conditions, if move accepted
        // make response move
        // check win conditions
        // return true for successful move
        // false for failed or rejected. handle in the request
        false
    }
}



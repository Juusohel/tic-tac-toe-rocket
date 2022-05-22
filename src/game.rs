use crate::game::GameStatus::{OWon, XWon, DRAW, RUNNING};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

/// Used to keep track of game status
pub enum GameStatus {
    RUNNING,
    XWon,
    OWon,
    DRAW,
}

/// Container for a HashMap of Player X/O choices for each game by ID
///
/// This is stored separately to the game object itself as the game object has to be able to be returned
/// in a specific way with specific fields as outlined in the document.
///
/// The HashMap is wrapped in a Mutex to allow it to be handled asynchronously by all functions that need it.
pub struct PlayerList {
    pub player_map: Mutex<HashMap<String, char>>,
}

/// Container for a HashMap of games by ID.
///
/// This is used as the active storage for the program. Scalable in reasonable amounts considering the
/// performance of rust but a database would be preferable for a large scale deployment.
/// Database would be added complexity in anything but the largest deployments.
pub struct GameList {
    pub list: Mutex<HashMap<String, Game>>,
}

/// Struct that represents the game object that stores all the information about the game and
/// handles all the logic within its functions. Derives traits to allow it to be converted to json
/// and cloned
#[derive(Clone, Serialize, Deserialize)]
pub struct Game {
    /// The game's UUID, read-only. Generated on object creation.
    id: Option<String>,

    /// The board state
    board: String,

    /// The game status
    status: Option<String>,
}

impl Game {
    /// Creates a new game instance
    /// Checks whether the board is an acceptable starting board and returns and error if not.
    ///
    /// The function validates the initial board state and fails if the board is not a valid starting board.
    ///
    /// If the player has made a starting move, the function checks which sign the user has used and
    /// saves it to PlayerList.
    /// If the player has not made a move, the function will randomly assign itself (and the player)
    /// a sign, and makes a first move.
    ///
    /// Returns the new game object
    ///
    /// # Arguments
    ///
    /// * 'board' - Starting board
    ///
    /// * 'player_list' - Maintains a map of all players and their sign choice (X or O) in a mutex to handle async requests
    ///
    /// # Panics
    /// May panic if the the function is unable to open up the mutex
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
        let mut x_count = 0;
        let mut o_count = 0;
        for character in board.chars() {
            match character {
                'X' => {
                    x_count += 1;
                    continue;
                }
                'O' => {
                    o_count += 1;
                    continue;
                }
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
            } else {
                first_move = "X";
                player_move = 'O';
            }
            // Making the first move by replacing a random tile with with the random sign.
            board.replace_range(random..random + 1, first_move);
        } else if (x_count == 1) && (o_count == 0) {
            player_move = 'X'; // If player has placed an X to start

            // Computer response move
            board = make_computer_move(board, "O");
        } else {
            player_move = 'O'; // if board is not empty and not X then player placed O

            // Computer response move
            board = make_computer_move(board, "X");
        }

        // Creating game object to be returned
        let game = Game {
            id: uuid,
            status: Some(String::from("RUNNING")),
            board,
        };

        // Adding player and game id to map
        let _ = lock.insert(uuid_copy, player_move);

        Ok(game)
    }

    /// Sets the board game board
    /// Does NOT validate the board
    ///
    /// # Arguments
    /// * 'board' - A representation of the board
    pub fn set_board(&mut self, board: String) {
        self.board = board
    }

    /// Gets the current board
    ///
    /// Returns a string representing the current board.
    pub fn get_board(&self) -> &String {
        &self.board
    }

    /// Gets the current status of the game
    pub fn get_status(&self) -> &Option<String> {
        &self.status
    }

    /// Sets the status of the game to one of 4 options defined by GameStatus
    ///
    /// # Arguments
    ///
    /// 'game_status' - GameStatus used to set the game status
    fn set_status(&mut self, game_status: GameStatus) {
        match game_status {
            GameStatus::RUNNING => self.status = Some(String::from("RUNNING")),
            GameStatus::XWon => self.status = Some(String::from("X_WON")),
            GameStatus::OWon => self.status = Some(String::from("O_WON")),
            GameStatus::DRAW => self.status = Some(String::from("DRAW")),
        }
    }

    /// Returns the id of the game
    pub fn get_id(&self) -> &Option<String> {
        &self.id
    }

    /// Checks the board to determine if any win conditions are met.
    /// If win conditions are met, the status of the game will be updated.
    ///
    /// The function iterates through the board checking for each win condition.
    /// Multiple methods of determining win conditions are used for both proof of concept and convenience.
    ///
    /// Returns True if any win conditions are met
    /// Returns False if no win conditions are met
    /// DRAW counts as a win condition
    pub fn check_win_conditions(&mut self) -> bool {
        let board_rows: Vec<&str>;
        let current_board = &self.board.clone();
        let row0 = &current_board[0..3];
        let row1 = &current_board[3..6];
        let row2 = &current_board[6..];
        board_rows = vec![row0, row1, row2];

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
                    break;
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
                    break;
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
            if (r0_char == r1_char) && (r2_char == r0_char) {
                match r0_char {
                    'X' => {
                        self.set_status(XWon);
                        return true;
                    }
                    'O' => {
                        self.set_status(OWon);
                        return true;
                    }
                    _ => continue,
                }
            }
        }

        // Checking diagonals
        // Grabbing the characters we need to check
        // initializing with a default value and mutable because of rust security, overwritten by loop
        let mut zero = '-';
        let mut two = '-';
        let mut four = '-';
        let mut six = '-';
        let mut eight = '-';
        // Assigning the signs we want to a variable.
        for (i, char) in current_board.chars().enumerate() {
            match i {
                0 => zero = char,
                2 => two = char,
                4 => four = char,
                6 => six = char,
                8 => eight = char,
                _ => continue,
            }
        }
        // Comparisons
        // 0 - 4 - 8 Diagonal
        if (zero == eight) && (zero == four) {
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

    /// Accepts move by player, and makes a move in response.
    /// Computer will make their own move randomly as implementing best move algorithm was out of scope
    /// for this.
    ///
    /// This function assumes that the front-end won't allow old tiles to be completely overwritten as
    /// tracking individual tiles state when the board is represented by a string is impractical.
    ///
    /// # Arguments
    ///
    /// * 'new_board' - A representation of the updated board with a yet to be validated move.
    ///
    /// * 'player_list' - Maintains a map of all players and their sign choice (X or O) in a mutex to handle async requests
    pub fn make_move(&mut self, mut new_board: String, player_list: &PlayerList) -> bool {
        let game_status = self.status.clone().unwrap();
        let mut lock = player_list.player_map.lock().unwrap(); // Bringing player map
        let game_id = &self.id.clone().unwrap();
        let player_move = lock.get(game_id).unwrap(); // Function can't be called without the game existing, safe to unwrap
        let mut current_board = self.get_board().clone();
        let mut computer_sign = "";

        if game_status != String::from("RUNNING") {
            // Game is over, don't accept a move
            return false;
        }

        // Counting current characters
        let mut current_x = 0;
        let mut current_o = 0;
        let mut current_empty = 0;

        for char in current_board.chars() {
            match char {
                'X' => current_x += 1,
                'O' => current_o += 1,
                '-' => current_empty += 1,
                _ => panic!("Current board is not valid"), // Current board should never be invalid at this stage
            }
        }
        // Counting new board signs
        let mut new_x = 0;
        let mut new_o = 0;
        let mut new_empty = 0;

        for char in new_board.clone().chars() {
            match char {
                'X' => new_x += 1,
                'O' => new_o += 1,
                '-' => new_empty += 1,
                _ => return false, // New move contains an invalid board, move not accepted
            }
        }

        // Comparing boards to check validity of the move and setting computer sign
        match player_move {
            'X' => {
                computer_sign = "O";
                if !(((new_x - current_x) == 1)
                    && (((new_o - current_o) == 0) && ((current_empty - new_empty) == 1)))
                {
                    // If conditions above are not true, the move is not valid and rejected.
                    return false;
                }
            }
            'O' => {
                computer_sign = "X";
                if !(((new_o - current_o) == 1)
                    && (((new_x - current_x) == 0) && ((current_empty - new_empty) == 1)))
                {
                    // Same as above but with other player sign
                    return false;
                }
            }
            _ => panic!("Player move not set"), // Should be impossible, appropriate to panic
        }
        // If move is valid, set the updated board to be the current board
        self.set_board(new_board);

        // update current board variable
        current_board = self.get_board().clone();

        // Checking if player move has fulfilled win conditions, if not make counter move.
        if self.check_win_conditions() == false {
            // Making counter computer move
            let current_board = make_computer_move(current_board, computer_sign);

            // Updating board with computer move
            self.set_board(current_board);
        }

        // Checking win conditions after computer move
        self.check_win_conditions();

        true
    }
}

/// Makes a computer move. This function only updates the board and does not check being used
/// out of turn etc. Making this function public could break game logic.
///
/// Checks which positions are open ('-') in the string, and places their indexes into an array
/// A random number in that range is then generated and the move made in that slot
///
/// Returns updated board
///
/// # Arguments
///
/// * 'current_board' - Representation of the board as it is before a computer move is made
fn make_computer_move(mut current_board: String, computer_sign: &str) -> String {
    // Checks which positions are open ('-') in the string, and places their indexes into an array
    // A random number in that range is then generated and the move made in that slot
    let mut empty_spaces = vec![];
    for (i, char) in current_board.clone().chars().enumerate() {
        if char == '-' {
            empty_spaces.push(i);
        }
    }

    // Generating random number to choose the slot to make computer move
    let mut rng = rand::thread_rng();
    let random_choice = rng.gen_range(0..empty_spaces.len());

    // Making computer move
    let index_to_be_replaced = empty_spaces[random_choice];
    current_board.replace_range(
        index_to_be_replaced..index_to_be_replaced + 1,
        computer_sign,
    );

    //returning updated board
    current_board
}

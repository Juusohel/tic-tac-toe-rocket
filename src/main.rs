mod game;

#[macro_use] extern crate rocket;

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Mutex;
use rocket::http::{ContentType, Status};
use rocket::response::{Redirect, Responder};
use rocket::{Request, Response, response, State};
use rocket::http::uri::Uri;
use rocket::serde::json::Json;
use rocket::serde::json::serde_json::json;
use url::Url;
use crate::game::{Game, GameList, PlayerList};

/// Container for HTTP responses
struct APIResponse<T> {
    /// Json payload for the response
    json: Json<T>,
    /// HTTP Response status code
    status: Status,
}

// Response build structure modelled after https://stackoverflow.com/a/70563341

impl <'r, T: serde::Serialize> Responder<'r, 'r> for APIResponse<T> {
    /// Builds response
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(&req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}



/// Base index response
///
/// Unused in API context but left here to avoid not having any kind of index
#[get("/")]
fn index() -> &'static str {
    "Nothing here go to /games"
}


/// Gets a list of all games and returns them as as an array
///
///
/// # Arguments
///
/// * 'game_list' - Maintains a map of all games in a mutex to handle asynchronous requests
///
#[get("/games")]
fn all_games(game_list: &State<GameList>) -> APIResponse<Vec<Game>> {
    let lock = game_list.inner(); // Getting state
    let guard = lock.list.lock().unwrap();
    let all_games = guard.values().cloned().collect::<Vec<Game>>();

    APIResponse {
        json: Json(all_games),
        status: Status::Ok,
    }

}

/// Returns the current game object based on its ID which is parsed from the URL.
///
/// # Arguments
///
/// * 'id' - Parsed from the URL, ID of the game
///
/// * 'game_list' - Maintains a map of all games in a mutex to handle asynchronous requests
///
/// # Panics
/// May panic if the the function is unable to open up the mutex
#[get("/games/<id>")]
fn game_board(id: String, game_list: &State<GameList>) -> Result<APIResponse<Game>, Status> {
    let lock = game_list.inner(); // Getting state
    let current_game;
    if lock.list.lock().unwrap().contains_key(&*id) { // If id exists, get the game
        let guard = lock.list.lock().unwrap();
        let map_entry = guard.get(&*id);
        match map_entry {
            Some(game) => current_game = game,
            _ => {
                return Err(Status::InternalServerError); // Should be unreachable;
            }
        }
        return
            Ok(APIResponse {
            json: Json(current_game.clone()),
            status: Status::Ok,
        })
    }
    Err(Status::NotFound)
}

/// Handles the put request to make a new move to a specified game
///
/// Gets the active game by id parsed from the URL and tries to make the user defined moved
/// which is the payload in the PUT request.
///
/// Returns the updated game board with the computer's response move updated to the board
///
/// # Arguments
///
/// * 'id' - Parsed from the URL, ID of the game
///
/// * 'game_list' - Maintains a map of all games in a mutex to handle asynchronous requests
///
/// * 'game' - Payload in the PUT request, contains to game object with an updated board. (Player move)
///
/// * 'player_signs' - Maintains a map of all players and their sign choice (X or O) in a mutex to handle async requests
///
/// # Panics
/// May panic if the the function is unable to open up the mutex
#[put("/games/<id>" , format = "json", data = "<game>")]
fn put_player_move(id: String, game_list: &State<GameList>, game: Json<Game>, player_signs: &State<PlayerList>) -> Result<APIResponse<Game>, Status> {
    let game_list_lock = game_list.inner();
    let submitted_new_game_state = game;
    let current_game;

    let player_list_lock = player_signs.inner();


    // if game exists
    if game_list_lock.list.lock().unwrap().contains_key(&*id) {
        let mut guard = game_list_lock.list.lock().unwrap();
        let map_entry = guard.get_mut(&*id);

        match map_entry {
            Some(game) => current_game = game,
            _ => {
                return Err(Status::InternalServerError);
            }
        }
        let new_board = submitted_new_game_state.get_board().clone();// generate new board based on moves TEMP
        if current_game.make_move(new_board, player_list_lock) == false {
            return Err(Status::BadRequest);
        }
        // Maybe set status to something if needed
        return Ok(
            APIResponse {
                json: Json(current_game.clone()),
                status: Status::Ok
        })
    }
    Err(Status::NotFound)
}


/// Creates a new game with a board as defined in the POST request payload
///
/// The handler will validate a user defined first move and provide a response move from the computer
///
/// # Arguments
///
/// * 'board' - POST request payload, contains a representation of the game board
///
/// * 'game_list' - Maintains a map of all games in a mutex to handle asynchronous requests
///
/// * 'player_signs' - Maintains a map of all players and their sign choice (X or O) in a mutex to handle async requests
///
/// # Panics
/// May panic if the the function is unable to open up the mutex
#[post("/games", format = "json", data = "<board>")]
fn new_game(board: Json<Game> , game_list: &State<GameList>, player_signs: &State<PlayerList>) -> Result<APIResponse<Url>, Status> {
    // New getting board from the game object in the request
    let new_board = board.get_board().clone();

    // Pulling player map in
    let player_map = &player_signs.inner().player_map;

    // Creating new game object with the board
    let try_new_game = Game::new(new_board, player_signs);
    let new_game;
    match try_new_game {
        Ok(valid_game) => new_game = valid_game,
        Err(e) => {
            println!("{}", e);
            return Err(Status::BadRequest)
        }
    }

    // Getting game id for use in map of games and url
    let id = new_game.get_id().clone().unwrap();
    let id_for_redirect = id.clone();

    // Adding game to map
    let lock = game_list.inner();
    lock.list.lock().unwrap().insert(id,new_game);

    // redirecting to game
    // Would be set to actual host adress in prod with env variable
    let current_host ;
    match Url::parse("http://127.0.0.1:8000/") {
        Ok(host_url) => current_host = host_url,
        Err(e) => {
            println!("{}", e);
            return Err(Status::InternalServerError);
        }
    }

    let game_url;
    match current_host.join(&*format!("../games/{}", id_for_redirect)) {
        Ok( url) => game_url = url,
        Err(e) => {
            println!("{}", e);
            return Err(Status::InternalServerError);
        }
    }
    Ok(
        APIResponse {
            json: Json(game_url),
            status: Status::Created
        })
}


/// Deletes a game from the list of games and returns it.
///
/// # Arguments
///
/// * 'id' - Parsed from the URL, ID of the game
///
/// * 'game_list' - Maintains a map of all games in a mutex to handle asynchronous requests
///
/// # Panics
/// May panic if the the function is unable to open up the mutex
#[delete("/games/<id>")]
fn delete_game(id: String, game_list: &State<GameList>) -> Result<APIResponse<Game>, Status> {
    let lock = game_list.inner();
    let mut list = lock.list.lock().unwrap();
    let delete = list.remove(&*id);

    match delete {
        Some(game) => return Ok(
            APIResponse {
                json: Json(game),
                status: Status::Ok
        }),
        None => return Err(Status::NotFound)
    }

}



#[launch]
fn rocket() -> _ {

    // Launching rocket
    rocket::build()
        .manage(GameList { list: Mutex::new(HashMap::new()) })
        .manage(PlayerList { player_map: Mutex::new(HashMap::new())})
        .mount("/", routes![index])
        .mount("/", routes![all_games, game_board, new_game, put_player_move, delete_game])


}


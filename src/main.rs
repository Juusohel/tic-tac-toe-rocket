mod game;
#[macro_use] extern crate rocket;

use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::sync::Mutex;
use rocket::response::Redirect;
use rocket::State;
use rocket::serde::json::Json;
use crate::game::{Game, GameList};


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/games")]
fn all_games(game_list: &State<GameList>) -> Json<Vec<Game>> {
    let lock = game_list.inner(); // Getting state
    let guard = lock.list.lock().unwrap();
    let all_games = guard.values().cloned().collect::<Vec<Game>>();

    Json(all_games)
}

#[get("/games/<id>")]
fn game_board(id: String, game_list: &State<GameList>) -> Json<Game> {
    let lock = game_list.inner(); // Getting state
    let current_game;
    if lock.list.lock().unwrap().contains_key(&*id) { // If id exists, get the game
        let guard = lock.list.lock().unwrap();
        let map_entry = guard.get(&*id);
        match map_entry {
            Some(game) => current_game = game,
            _ => {
                panic!("unreachable");
            }
        }
        return Json(current_game.clone());
    }
    panic!("Game doesn't exist");
}


#[put("/games/<id>" , format = "json", data = "<game>")]
fn put_player_move(id: String, game_list: &State<GameList>, game: Json<Game>) -> Json<Game> {
    let lock = game_list.inner();
    let submitted_new_game_state = game;
    let current_game;
    if lock.list.lock().unwrap().contains_key(&*id) {
        let mut guard = lock.list.lock().unwrap();
        let map_entry = guard.get_mut(&*id);

        match map_entry {
            Some(game) => current_game = game,
            _ => {
                panic!("unreachable");
            }
        }
        // Validate move
        // make computer move
        let new_board = submitted_new_game_state.get_board().clone();// generate new board based on moves TEMP
        current_game.set_board(new_board);
        // Maybe set status to something if needed
        return Json(current_game.clone());
    }
    panic!("No game found")

}


#[post("/games", format = "json", data = "<board>")]
fn new_game(board: Json<Game> , game_list: &State<GameList>) -> Redirect {
    // New getting board from the game object in the request
    let new_board = board.get_board().clone();

    // Creating new game object with the board
    // TODO verify valid board
    let new_game = Game::new(new_board);
    // Getting game id for use in map of games and url
    let id = new_game.get_id().clone().unwrap();
    let id_for_redirect = id.clone();

    // Adding game to map
    let lock = game_list.inner();
    lock.list.lock().unwrap().insert(id,new_game);

    // redirecting to game
    Redirect::to(format!("games/{}",id_for_redirect))
}



#[launch]
fn rocket() -> _ {




    rocket::build()
        .manage(GameList { list: Mutex::new(HashMap::new()) })
        .mount("/", routes![index])
        .mount("/", routes![all_games, game_board, new_game, put_player_move])


}


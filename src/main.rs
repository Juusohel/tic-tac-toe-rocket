mod game;
#[macro_use] extern crate rocket;

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
fn all_games() -> &'static str {
    "World?"
}

#[get("/games/<id>")]
fn game_board(id: String, game_list: &State<GameList>) -> String {
    let lock = game_list.inner();
    let board_state;
    match lock.list.lock().unwrap().get(&*id) {
        Some(game) => board_state = game.get_board().clone(),
        None => board_state = String::from("No game found!")
    }
    board_state
}


#[post("/games", format = "json", data = "<board>")]
fn new_game(board: Json<Game> , game_list: &State<GameList>) -> Redirect {
    let new_board = board.get_board().clone();
    let new_game = Game::new(new_board);
    let id = new_game.get_id().clone().unwrap();
    let id_for_redirect = id.clone();
    let lock = game_list.inner();
    lock.list.lock().unwrap().insert(id,new_game);
    Redirect::to(format!("games/{}",id_for_redirect))
}



#[launch]
fn rocket() -> _ {




    rocket::build()
        .manage(GameList { list: Mutex::new(HashMap::new()) })
        .mount("/", routes![index])
        .mount("/", routes![all_games, game_board, new_game])


}


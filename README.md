# Tic Tac Toe backend API
 
Tic Tac Toe REST API that handles Tic Tac Toe games using
where the board is represented by a string. Game objects 
are parsed and returned as JSON objects.


### Requests
* GET /games
  * returns an array of all active games
* POST /games
  * Creates a new game using the board representation in the body of the request
    * Fails if board is not valid
  * Returns URL to the created game
* GET /games/`id`
  * Returns the game with the id parsed from the request
    * Fails if game does not exist
* PUT /games/`id`
  * Updates the board with the move made by the player using the representation of the board in the body of the request.
    * Move is validated by the server and an updated game board is returned if the request successful
* DELETE /games/`id`
  * Deletes the specified game
    * Fails if game not found

### Compiling and running
#### Prerequisites
* Rust
  * Install on any platform with rustup
* Other dependencies listed in Cargo.toml

#### Installation and running locally
- [] Clone repository
- [] Compile with `cargo build`
- [] Run with `cargo run`

### Configuration
Default: Program is configured to run locally on `localhost:8000`.
There is a variable `current_host` which in production should be set by environment variable (hardcorded for convenience in this repository).
**Change this if changing host** 

To change host and other API settings (such as 404 templates), refer to Rocket documentation
   
 * https://rocket.rs/v0.5-rc/guide/configuration/#configuration
 

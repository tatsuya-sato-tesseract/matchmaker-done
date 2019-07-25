use hdk::holochain_persistence_api::cas::content::Address;

use super::GameState;
use crate::game::Game;
use crate::game_move::Move;
use crate::your_game::MoveType;

/**
 *
 * To implement your own custom rule validation all you need to do is re-implement the function 'is_valid' on 'Move'
 *
 * This function takes the current game and the game state (which includes all the existing moves)
 * and deermines if a new candidate move is valid. Typically this will involve first matching on the move type
 * and then determining if the move is valid.
 *
 * It function must return Ok(()) if a move is valid and Err("Some error string") for an invalid move.
 * It is useful to provide descriptive error strings as these can be visible to the end user.
 */

impl Move {
    pub fn is_valid(&self, game: Game, game_state: GameState) -> Result<(), String> {
        // <<DEVCAMP-TODO>> Check if a move is valid given the current game and its state
        let is_player_turn_result = is_players_turn(self.author.clone(), &game, &game_state)?;
        is_the_right_move(&game_state, &self.move_type, is_player_turn_result, &game)?;
        Ok(())
    }
}

// some helper function for the validation of the moves

fn is_players_turn(
    player: Address,
    game: &Game,
    game_state: &GameState,
) -> Result<Address, String> {
    let moves = &game_state.moves;
    match moves.last() {
        Some(last_move) => {
            if last_move.author == player {
                Err("It is not this player turn".into())
            } else {
                Ok(player)
            }
        }
        None => {
            //need to handle when no one has made the first move yet
            if game.player_2 == player {
                Ok(player) // player 2 can start first by convention
            } else {
                Err("Player 2 must start the game".into())
            }
        }
    }
}

fn is_the_right_move(
    game_state: &GameState,
    move_type: &MoveType,
    player: Address,
    game: &Game,
) -> Result<(), String> {
    let player_2_suggest = game_state.player_2_suggests;
    // determine if the current player should suggest or predict
    if player == game.player_2 {
        if player_2_suggest == true {
            match move_type {
                MoveType::Suggest { .. } => Ok(()),
                MoveType::Predict { .. } => {
                    Err("Player 2 must suggest, not predict. Use swap to switch roles".into())
                }
                _ => Ok(()),
            }
        } else {
            match move_type {
                MoveType::Suggest { .. } => {
                    Err("Player 2 must predict, not suggest. Use Swap switch roles".into())
                }
                MoveType::Predict { .. } => Ok(()),
                _ => Ok(()),
            }
        }
    } else {
        if player_2_suggest == true {
            match move_type {
                MoveType::Suggest { .. } => {
                    Err("Player 1 must predict not suggest. Use swap switch roles".into())
                }
                MoveType::Predict { .. } => Ok(()),
                _ => Ok(()),
            }
        } else {
            match move_type {
                MoveType::Suggest { .. } => Ok(()),
                MoveType::Predict { .. } => {
                    Err("Player 1 must suggest not predict. Use swap to switch roles".into())
                }
                _ => Ok(()),
            }
        }
    }
}

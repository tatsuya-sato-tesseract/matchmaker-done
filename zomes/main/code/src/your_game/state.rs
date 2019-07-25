use hdk::holochain_json_api::{error::JsonError, json::JsonString};

use super::MoveType;
use crate::game::Game;
use crate::game_move::Move;

/**
 *
 * As a game autor you get to decide what the State object of your game looks like.
 * Most of the time you want it to include all of the previous moves as well.
 *
 * To customize the game state implement your own GameState struct. This must have a function called 'initial()'
 * which return the initial state.
 *
 */

#[derive(Clone, Debug, Serialize, Deserialize, DefaultJson)]
pub struct GameState {
    // <<DEVCAMP-TODO>>
    pub moves: Vec<Move>,
    pub suggestion: usize,
    pub player_1_successful_suggestion: usize,
    pub player_1_successful_prediction: usize,
    pub player_1_suggestion_attempts: usize,
    pub player_1_prediction_attempts: usize,
    pub player_2_successful_suggestion: usize,
    pub player_2_successful_prediction: usize,
    pub player_2_suggestion_attempts: usize,
    pub player_2_prediction_attempts: usize,
    pub player_2_suggests: bool,
    // Implement your own game state
    // May be helpful to split this into state for each player
}

impl GameState {
    pub fn initial() -> Self {
        // <DEVCAMP> return an initial state of a game
        Self {
            moves: Vec::new(),
            suggestion: 0,
            player_1_successful_suggestion: 0,
            player_1_successful_prediction: 0,
            player_1_suggestion_attempts: 0,
            player_1_prediction_attempts: 0,
            player_2_successful_suggestion: 0,
            player_2_successful_prediction: 0,
            player_2_suggestion_attempts: 0,
            player_2_prediction_attempts: 0,
            player_2_suggests: true,
        }
    }

    pub fn render(&self, game: &Game) -> String {
        // <<DEVCAMP>> return a pretty formatting string representation
        let game_string;
        let last_move = self.moves.last();
        // determine what kinf of string to display based on whose turn it
        // is currently and what kinf of move is allowed for that player.
        match last_move {
            Some(last_move) => {
                if last_move.author == game.player_1 {
                    if self.player_2_suggests == true {
                        game_string = "Waiting for player 2 to suggest a number..."
                    } else {
                        game_string = "Waiting for player 2 to predict the suggeted number..."
                    }
                } else {
                    if self.player_2_suggests == true {
                        game_string = "Waiting for player 1 to predict the suggested number..."
                    } else {
                        game_string = "Waiting for player 1 to suggest a number..."
                    }
                }
            }
            // when no one else made a move yet, player 2 must always make the first move so
            None => game_string = "Waiting for player 2 to suggest a number",
        }
        format!(" {} \nplayer 1 record: \n\tsuggestion: {}/{} \n\tprediction: {}/{} \nplayer 2 record: \n\tsuggestion: {}/{} \n\tprediction: {}/{}\n", game_string, self.player_1_successful_suggestion, self.player_1_suggestion_attempts, self.player_1_successful_prediction, self.player_1_prediction_attempts, self.player_2_successful_suggestion, self.player_2_suggestion_attempts, self.player_2_successful_prediction, self.player_2_prediction_attempts)
    }

    pub fn evolve(&self, game: Game, next_move: &Move) -> GameState {
        // <<DEVCAMP>>
        // given a current state, a game and a move, compute the next state
        // You can assume all moves are valid

        //unpack the move
        let mut moves = self.moves.clone();
        let mut new_suggestion = self.suggestion;
        let mut player_1_successful_suggestion = self.player_1_successful_suggestion;
        let mut player_1_successful_prediction = self.player_1_successful_prediction;
        let mut player_1_suggestion_attempts = self.player_1_suggestion_attempts;
        let mut player_1_prediction_attempts = self.player_1_prediction_attempts;

        let mut player_2_successful_suggestion = self.player_2_successful_suggestion;
        let mut player_2_successful_prediction = self.player_2_successful_prediction;
        let mut player_2_suggestion_attempts = self.player_2_suggestion_attempts;
        let mut player_2_prediction_attempts = self.player_1_prediction_attempts;

        let mut player_2_suggests = self.player_2_suggests;

        //add the new move to the state
        moves.push(next_move.clone());

        match next_move.move_type {
            // match to all the available MoveTypes.
            MoveType::Suggest { suggestion } => {
                new_suggestion = suggestion;
                // figure out which player made the move
                if game.player_1 == next_move.author {
                    player_1_suggestion_attempts += 1
                } else {
                    player_2_suggestion_attempts += 1
                }
            }
            MoveType::Predict { prediction } => {
                if game.player_1 == next_move.author {
                    if new_suggestion == prediction {
                        player_1_successful_prediction += 1
                    } else {
                        player_2_successful_suggestion += 1
                    }
                    player_1_prediction_attempts += 1
                } else if game.player_2 == next_move.author {
                    if new_suggestion == prediction {
                        player_2_successful_prediction += 1
                    } else {
                        player_1_successful_suggestion += 1
                    }
                    player_2_prediction_attempts += 1
                }
            }
            MoveType::Swap {} => {
                if player_2_suggests == true {
                    player_2_suggests = false
                } else {
                    player_2_suggests = true
                }
            }
        }

        //finally return the new state
        GameState {
            moves,
            suggestion: new_suggestion,
            player_1_successful_suggestion,
            player_1_successful_prediction,
            player_1_suggestion_attempts,
            player_1_prediction_attempts,
            player_2_successful_suggestion,
            player_2_successful_prediction,
            player_2_suggestion_attempts,
            player_2_prediction_attempts,
            player_2_suggests,
        }
    }
}

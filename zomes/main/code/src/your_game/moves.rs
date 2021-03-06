use hdk::holochain_json_api::{error::JsonError, json::JsonString};

/**
 *
 * The MoveType enum defines all the types of moves that are valid in your game and the
 * data they carry. In Checkers you can move a piece (MovePiece) from a location to another location.
 *
 */

#[derive(Clone, Debug, Serialize, Deserialize, DefaultJson, PartialEq)]
pub enum MoveType {
    // <<DEVCAMP-TODO>> YOUR MOVE ENUM VARIENTS HERE
    Suggest { suggestion: usize },
    Predict { prediction: usize },
    Swap {},
}

impl MoveType {
    pub fn describe() -> Vec<MoveType> {
        // <<DEVCAMP-TODO>> SHOULD RETURN AN EXAMPLE OF EACH VARIENT
        vec![
            MoveType::Suggest { suggestion: 0 },
            MoveType::Predict { prediction: 0 },
            MoveType::Swap {},
        ]
    }
}

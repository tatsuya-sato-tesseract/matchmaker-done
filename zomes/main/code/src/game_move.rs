use hdk::{
    entry_definition::ValidatingEntryType,
    holochain_core_types::{
        dna::entry_types::Sharing, entry::Entry, validation::EntryValidationData,
    },
    holochain_json_api::{error::JsonError, json::JsonString},
    holochain_persistence_api::cas::content::Address,
};

use crate::game::{get_game_local_chain, get_state_local_chain};
use crate::MoveType;

#[derive(Clone, Debug, Serialize, Deserialize, DefaultJson)]
pub struct MoveInput {
    pub game: Address,
    pub move_type: MoveType,
    pub timestamp: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, DefaultJson, PartialEq)]
pub struct Move {
    pub game: Address,
    pub author: Address,
    pub move_type: MoveType,
    pub previous_move: Address,
    pub timestamp: u32,
}

pub fn definition() -> ValidatingEntryType {
    entry!(
        name: "move",
        description: "A move by an agent in a game",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::ChainFull
        },

        validation: | validation_data: hdk::EntryValidationData<Move>| {
        /* In this entry's validation, we match the validation_data with the variants of EntryValidationData.
         * If the variant is Create, meaning if we are trying to create an instance of this entry,
         * then we run a validation in which we retrieve the source_chain_entries found in the validation_data
         * of the Create struct variant. we use ok_or to convert the Option<T> to Result<T, Err> then return an error if
         * we cant retrieve the source chain entry using ? operator.
         */
            match validation_data {
                EntryValidationData::Create{entry, validation_data} => {
                    let mut local_chain = validation_data.package.source_chain_entries
                        .ok_or("Could not retrieve source chain")?;
                    hdk::debug(format!("{:?}", local_chain))?;

                    // now load the game and game state from here
                    /* We use from to convert the entry to Move struct and store it in _new_move. */
                    let _new_move = Move::from(entry);

                    // Sometimes the validating entry is already in the chain when validation runs,
                    // To make our state reduction work correctly this must be removed
                    /*
                     * remove_item is a provided method for Vec<T> which is the
                     * source_chain_entries' type. so we use remove_item() with argument being a reference
                     * to Entry::App with "move" AppEntryType and _new_move as the AppEntryValue.
                     */
                    local_chain.remove_item(&Entry::App("move".into(), _new_move.clone().into()));

                    /*
                     * In order to get the state, we use the get_state_local_chain and call map_err
                     * method on it to change all Err to the string literal provided in the expression
                     * of map_err. Same with the get_game_local_chain.
                     */
                    let state = get_state_local_chain(local_chain.clone(), &_new_move.game)?;
                    let game = get_game_local_chain(local_chain, &_new_move.game)
                        .map_err(|_| "Could not load game during validation")?;

                    /* Finally, we call is_valid() on _new_move to make sure it is the player's turn,
                     * and make sure that the player is making the right move
                     */
                    _new_move.is_valid(game, state)
                },
                _ => {
                    Err("Cannot modify or delete a move".into())
                }
            }
        },
        links: [
            from!(
                "game",
                link_type: "",
                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },
                validation: | _validation_data: hdk::LinkValidationData| {
                    Ok(())
                }
            ),
            from!(
                "move",
                link_type: "",
                validation_package: || {
                    hdk::ValidationPackageDefinition::Entry
                },
                validation: | _validation_data: hdk::LinkValidationData| {
                    Ok(())
                }
            )
        ]
    )
}

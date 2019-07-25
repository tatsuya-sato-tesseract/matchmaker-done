use hdk::{
    entry_definition::ValidatingEntryType,
    error::{ZomeApiError, ZomeApiResult},
    holochain_core_types::{
        dna::entry_types::Sharing, entry::Entry, link::LinkMatch, validation::EntryValidationData,
    },
    holochain_json_api::{error::JsonError, json::JsonString},
    holochain_persistence_api::cas::content::{Address, AddressableContent},
    utils,
};
use std::convert::TryFrom;

use crate::game_move::Move;
use crate::GameState;

#[derive(Clone, Debug, Serialize, Deserialize, DefaultJson)]
pub struct Game {
    pub player_1: Address,
    pub player_2: Address,
    pub created_at: u32,
}

/*=====================================
=            DHT Functions            =
=====================================*/

/// Traverse the linked list rooted at a game to find all the moves
pub fn get_moves(game_address: &Address) -> ZomeApiResult<Vec<Move>> {
    match hdk::get_links(game_address, LinkMatch::Any, LinkMatch::Any)?
        .addresses()
        .into_iter()
        .next()
    {
        /* get links returns the ZomeApiResult<GetLinksResult>.
         * This will get entries that are linked to the first argument.
         * Since ZomeApiResult returns Result<T, ZomeApiError>(where T in this case is the GetLinksResult),
         * you can use the ? sugar to return the ZomeApiError if error then return the T if get_links is a success.
         * GetLinkResult has a method implemented called addresses() which returns a vector of Addresses.
         * into_iter() will iterate through this vector of addresses and move the value from the vector to an Iterator.
         * next() is a method for iterator where in it returns the next value of the Iterator (start at index 0) in Option<Self::Item>
         * Since next() returns an Option<Self::Item>, we can use the match operator to cater to all possible values of Option<Self: Item>
         */
        Some(first_move) => {
            let mut move_addresses = vec![first_move];
            let mut more = true;
            while more {
                more = match hdk::get_links(
                    move_addresses.last().unwrap(),
                    LinkMatch::Any,
                    LinkMatch::Any,
                )?
                .addresses()
                .into_iter()
                .next()
                {
                    Some(addr) => {
                        move_addresses.push(addr.clone());
                        true
                    }
                    None => false,
                }
            }
            /* In this match operator, we first cater to Some(first_move). The name is first_move because
             * the Game entry is always linked to the first_move made by Player 2.
             * So we store this first_move to a vector in a variable name move_addresses.
             * Then we create a while loop in order to store all the game_move entries that are linked to the first_move.
             * while more is true, we get the entries linked to the first_move, then the next move and the next move and
             * on and on and on until we finish all the linked moves. The way this works is, in the first argument of get_links,
             * we get the very last element of the move_addresses vector using last() which returns a Option<&T>.
             * Since we want the address itself wrapped in Option<&T>, we will use unwrap() to get the value of the Address.
             * In this way, we will always have the last address stored in move_addresses as our first argument in get_links.Address.
             * Then we do the same thing we did above to move the value from a vector of addresses to an Iterator then get the value with next().
             * Then we run the match operator again to store the address in the move_addresses using push() then return true to run the loop again.
             * Since next() returns None if there is no more value to be retrieved in the Iterator, we return false in None so that the loop ends after
             * we get all the moves that are linked together.
             */
            let moves: Vec<Move> = move_addresses
                .iter()
                .map(|addr| {
                    let move_entry = hdk::get_entry(addr).unwrap().unwrap();
                    if let Entry::App(_, move_struct) = move_entry {
                        Move::try_from(move_struct)
                            .expect("Entry at address is type other than Move")
                    } else {
                        panic!("Not an app entry!")
                    }
                })
                .collect();
            /* Now that we have a vector of addresses for all connected moves, we will now try to retrieve the data itself which can
             * be found in the Addresses we retrieved. First, we create a variable named moves which is a type of Vec<Move>. In this variable,
             * we will use the iter() method on move_addresses (note that we used iter() instead of into_iter() because we dont want
             * to move the value from move_addresses but rather have a referennce to the addresses found in the move_addresses.) and then
             * use map() method provided in Iterator. map() takes a closure and creates an iterator which calls that closure on each element.
             * the closure will have addr as an argument. The closure creates a variable named move_entry in which we will use the method
             * get_entry which takes an Address(HashString) type then return ZomeApiResult<Option<Entry>>. We then unwrap it twice to
             * retrieve the Entry itself. Then we use if let to match the move_entry with an Entry::App variant. This is because Entry
             * enum can have different variants and we need to makesure that the entry found in this address is an App variant. If not
             * then we throw a panic in else statement saying that it is not an app entry. Now if it is an app entry, we use the try_from method
             * to try to convert the Entry::App, which we assume to have the Move struct in the second element of App
             * variant(here named as move_struct) as the AppEntryValue type, to an actual Move struct. If the try_from fails then we throw an error
             * saying the Entry at the given address is not a Move type of entry. After we call the closure on all addresses in move_addresses,
             * we use the collect() to turn them into Vec<Move>. collect() would understand that the items should be collected into Vec<Move>
             * since that is the defined type for moves.
             */
            Ok(moves)
        }
        None => Ok(Vec::new()),
    }
}

pub fn get_state(game_address: &Address) -> ZomeApiResult<GameState> {
    let moves = get_moves(game_address)?;
    let game = get_game(game_address)?;
    let new_state = moves.iter().fold(GameState::initial(), |state, new_move| {
        state.evolve(game.clone(), new_move)
    });
    Ok(new_state)
    /* get_state takes the address of the game as a parameter and return a ZomeApiResult<GameState>. This is a reducer function.
     * First we create a vairable named moves and call the get_moves in it with the parameter game_address.
     * Since we have the ? operator in get_moves(), it will return the value T in Result<T, ZomeApiError> if nothing goes wrong.
     * T in this case is Vec<Move> which will also be the type of moves variable. next we create the game variable an call the get_game
     * with the game_address being its argument. get_game also returns ZomeApiResult with Game being the success value so we
     * use the ? to get the Game struct if no error occurs. with moves and game having the vectors we need, we will now create
     * a variable name new_state and call iter() on moves to turn it into an Iterator in order for us to call a method fold() on it.
     * fold() takes two arguments: an initial value, and a closure with two arguments: an 'accumulator', and an element.
     * The closure returns the value that the accumulator should have for the next iteration. In this case, the initial value is an empty
     * GameState created with initial() we associated with GameState. Then the accumulator will be named state which will hold the
     * initial value (empty GameState) we set. new_move will be each Move stored in moves. now we call the evolve() method we associated
     * with GameState in state.rs. evolve takes self, Game struct, and &Move so we clone game and give it as a first argument and a
     * reference to moves with new_move(automatically a reference since the element in fold has FnMut implemented). This evolve method will
     * add all the Move that is in the moves to the GameState which will be stored in new_state. now we can return this as Ok(new_state)
     */
}

pub fn get_game(game_address: &Address) -> ZomeApiResult<Game> {
    utils::get_as_type(game_address.to_owned())
    /* get_as_type load an entry from the given address in the argument then convert it to a given type wrapped in ZomeApiResult. In this case,
     * rust will infer that the type is Game since that is the return value of get_game function so it will convert the loaded entry from the
     * given address to ZomeApiResult<Game>
     */
}

/*=====  End of DHT Functions  ======*/

/*=============================================
=            Local chain functions            =
=============================================*/

pub fn get_game_local_chain(
    local_chain: Vec<Entry>,
    game_address: &Address,
) -> ZomeApiResult<Game> {
    local_chain
        .iter()
        .filter(|entry| entry.address() == game_address.clone())
        .filter_map(|entry| {
            if let Entry::App(_, entry_data) = entry {
                Some(Game::try_from(entry_data.clone()).unwrap())
            } else {
                None
            }
        })
        .next()
        .ok_or(ZomeApiError::HashNotFound)
    /* get_game_local_chain() gets all the Entry in the local_chain as well as the address of the game and will return ZomeApiResult<Game>.
     * now we will call the iter() method on the local_chain so that we can call the filter() method. filter() method will create an iterator
     * which uses a closure to determine if an element should be yielded. the closure must return true or false and if the closure returns
     * true on that element then filter() will return that element. if its a false it simply runs the same closure on the nexrt element.
     * now filter's closure check if the address of the each element found in the local_chain is equal to the address of game_address
     * by getting the address of each element in the localchain using address() method provided for the Entry type in hdk. we need to clone
     * the game_address because we are passing a reference in the parameter and we cant compare a reference to an actual value(not 100% sure
     * correct me if im wrong). If the address of the entry matches the game_address that is passed in the paramater, then we return that entry.
     * After getting all elements that have the address of game_address, we implement the filter_map() method which filters then maps.
     * filter_map() takes a closure as an argument which has to return Option<T>. If the closure returns Some(element) then we return the
     * element. If the closure returns None then we just skip and try the closure on the next element in local_chain. inside the closure,
     * we use the if let to make sure that each element is an Entry::App variant. If not we return None but if it is, then we use the try_from()
     * method on the entry_data found in the Entry::App and convert it to the Game struct cos at this point we are sure that the element
     * is an Entry::App variant that holds the Game struct as AppEntryValue. try_from returns Result<Self, Self::Error> so we use unwrap to get
     * the Self which in this case is Game. Since at this point, we are sure that there is only one match for the game_address provided
     * in the parameter, we use the next() to return the element. Since next() returns an Option<T>, we use the ok_or() method to turn
     * Option<T> to Result<T, E> and E here being the ZomeApiError::HashNotFound variant which indicates that the game_address provided in the
     * parameter did not match any entry in the local_chain. we return ZomeApiError::HashNotFound because ZomeApiResult expects any variant
     * of the ZomeApiError to be returned as an error value.
     */
}

pub fn get_moves_local_chain(
    local_chain: Vec<Entry>,
    game_address: &Address,
) -> ZomeApiResult<Vec<Move>> {
    Ok(local_chain
        .iter()
        .filter_map(|entry| {
            if let Entry::App(entry_type, entry_data) = entry {
                if entry_type.to_string() == "move" {
                    Some(Move::try_from(entry_data.clone()).unwrap())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .filter(|game_move| game_move.game == game_address.to_owned())
        .rev()
        .collect())
    /* This one is similar to get_game_local_chain. It takes the local_chain Entries and the game_address as the parameter and returns
     * a vector of Move wrapped in ZomeApiResult. We first call iter() again then use filter_map() to filter the entries in local chain
     * to Entry::App variant then if the entry_type (1st element of App variant) is equal to "move" then we return that entry using try_from
     * method and wrap it in Some(). else we return None if there is no Entry that has the entry_type of "move" and return None also if there
     * is no Entry:App variant in the local chain. After getting all entries with "move" as the entry_type, we need to filter them and only
     * yield "move" entry that has the game_address passed in the parameter. That's what the next filter() is for and we check if the game
     * field of the "move" entry we retrieve from filter_map equals to the game_address being passed in the parameter. We then use rev() to reverse
     * the iteration when we use the collect() method in order to collect them and turn them into Vec<Move>. // To verify:: why use rev()??
     */
}

pub fn get_state_local_chain(
    local_chain: Vec<Entry>,
    game_address: &Address,
) -> ZomeApiResult<GameState> {
    let moves = get_moves_local_chain(local_chain.clone(), game_address)?;
    let game = get_game_local_chain(local_chain, game_address)?;
    let new_state = moves
        .iter()
        .fold(GameState::initial(), move |state, new_move| {
            state.evolve(game.clone(), new_move)
        });
    Ok(new_state)
    /* get_state_local_chain is similar to get_state function. It takes local_chain and game_address as parameters and return the GameState.
     * we first get all the moves associated with the game_address given as parameter using get_moves_local_chain and store them in moves
     * variable. Then we get the game struct found at game_address using get_game_local_chain. We then create new_state and call iter()
     * on moves thne use fold() method. fold() will take an empty GameState sturct and then call evolve method on that empty GameState (sored in
     * state) to store all the Moves. the move keyword before the closure actually means is to move ownership of all of the captured
     * variables in the closure rather than borrow. To Verify: Not really sure which captured variables we are moving here rather than borrowing.
     */
}

/*=====  End of Local chain functions  ======*/

pub fn definition() -> ValidatingEntryType {
    entry!(
        name: "game",
        description: "Represents an occurence of a game between several agents",
        sharing: Sharing::Public,
        validation_package: || {
            hdk::ValidationPackageDefinition::Entry
        },

        validation: | validation_data: hdk::EntryValidationData<Game>| {
            match validation_data {
                EntryValidationData::Create{entry, validation_data: _} => {
                    let game = entry as Game;
                    if game.player_1 == game.player_2 {
                        return Err("Player 1 and Player 2 must be different agents.".into())
                    }
                    Ok(())
                },
                _ => {
                    Err("Cannot modify or delete a game".into())
                }
            }
        }
    )
}

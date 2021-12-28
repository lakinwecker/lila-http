use serde::{Deserialize, Serialize};
use serde_json::Value as JsValue;
use serde_with::{serde_as, FromInto};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Eq, PartialEq, Deserialize, Hash, Clone)]
pub struct ArenaId(pub String);

// naming is hard
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaShared {
    nb_players: u32,
    duels: JsValue,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    seconds_to_finish: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    seconds_to_start: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    is_started: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    is_finished: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    is_recently_finished: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    featured: Option<JsValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    podium: Option<JsValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pairings_closed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stats: Option<JsValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    team_standing: Option<JsValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    duel_teams: Option<JsValue>,
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Hash)]
pub struct UserId(pub String);
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameId(String);
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rank(usize);

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaFull {
    pub id: ArenaId,
    #[serde(flatten)]
    shared: ArenaShared,
    ongoing_user_games: HashMap<UserId, GameId>,
    // this duplicates info gotten from standing, remove
    #[serde_as(as = "FromInto<String>")]
    ranking: FullRanking,
    standing: Vec<JsValue>,
}

impl ArenaFull {
    fn user_from_id(&self, uid: &UserId) -> ClientMe {
        ClientMe::new(
            self.ranking.ranking.get(&uid),
            self.ongoing_user_games.get(&uid)
        )
    }

    fn standing_from_page(&self, page: usize) -> ClientStanding {
        ClientStanding::new(&self.standing, page)
    }
}

#[derive(Debug, Clone)]
struct FullRanking {
    ranking: HashMap<UserId, Rank>,
}

impl From<String> for FullRanking {
    fn from(user_ids_comma_separated: String) -> Self {
        let user_ids: Vec<UserId> = user_ids_comma_separated
            .split(",")
            .into_iter()
            .map(|uid| UserId(uid.to_string()))
            .collect();
        FullRanking {
            ranking: user_ids
                .into_iter()
                .enumerate()
                .map(|(index, uid)| (uid, Rank(index + 1)))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ClientMe {
    rank: Option<Rank>,
    withdraw: bool,
    game_id: Option<GameId>,
    pause_delay: Option<u32>,
}

impl ClientMe {
    pub fn new(rank: Option<&Rank>, game_id: Option<&GameId>) -> ClientMe {
        ClientMe {
            rank: rank.cloned(),
            withdraw: false,
            game_id: game_id.cloned(),
            pause_delay: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ClientStanding<'p> {
    page: u32,
    players: &'p [JsValue],
}

impl<'a> ClientStanding<'a> {
    pub fn new<'b>(full_standing: &'b Vec<JsValue>, page: usize) -> ClientStanding<'b> {
        ClientStanding {
            page: page as u32,
            players: &full_standing[((page - 1) * 10)..(page * 10 - 1)],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientData<'a> {
    #[serde(flatten)]
    shared: &'a ArenaShared,
    #[serde(skip_serializing_if = "Option::is_none")]
    me: Option<ClientMe>,
    standing: ClientStanding<'a>,
}

impl<'a> ClientData<'a> {
    pub fn new<'b>(full: &'b Arc<ArenaFull>, user_id: Option<UserId>) -> ClientData<'b> {
        let page = 1;
        ClientData {
            shared: &full.shared,
            me: user_id.map(|uid| full.user_from_id(&uid)),
            standing: full.standing_from_page(page)
        }
    }
}

//! Interpreting rooms in which PvP recently occurred. This is an "experimental" endpoint.

use crate::{
    data,
    error::{ApiError, Result},
    EndpointResult,
};

/// Call parameters for requesting recent pvp
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum RecentPvpArgs {
    /// Retrieves rooms where pvp has occurred recently, with a given number of game ticks.
    WithinLast {
        /// The interval of game ticks to request. It is unknown the maximum interval that may be requested.
        ticks: u32,
    },
    /// Retrieves rooms where pvp has occurred since a given game time.
    Since {
        /// The game "time" (tick number) to request PvP since. It is unknown how far back of a time may be requested.
        time: u32,
    },
}

impl RecentPvpArgs {
    /// Creates a new PvP call parameter to request any PvP occurring since the given game tick.
    pub fn since(tick: u32) -> RecentPvpArgs {
        RecentPvpArgs::Since { time: tick }
    }
    /// Creates a new PvP call parameter to request any PvP occurring within the last x ticks.
    pub fn within(ticks: u32) -> RecentPvpArgs {
        RecentPvpArgs::WithinLast { ticks: ticks }
    }
}

/// Recent PvP raw result.
#[derive(serde_derive::Deserialize, Clone, Hash, Debug)]
pub(crate) struct Response {
    ok: i32,
    #[serde(with = "::tuple_vec_map")]
    pvp: Vec<(String, InnerShard)>,
}

#[derive(serde_derive::Deserialize, Clone, Hash, Debug)]
struct InnerShard {
    rooms: Vec<InnerRoom>,
    time: u32,
}

#[derive(serde_derive::Deserialize, Clone, Hash, Debug)]
struct InnerRoom {
    _id: String,
    #[serde(rename = "lastPvpTime")]
    last_pvp_time: u32,
}

/// Result storing recent pvp matches for the entire world.
pub struct RecentPvp {
    /// A list of shard names and the recent pvp within that shard.
    pub shards: Vec<(String, ShardRecentPvp)>,
    /// Phantom data in order to allow adding any additional fields in the future.
    _non_exhaustive: (),
}

/// Result storing recent pvp matches for a particular shard.
#[derive(Clone, Hash, Debug)]
pub struct ShardRecentPvp {
    /// A list of room names in which pvp has recently occurred, and the time at which pvp last occurred.
    pub rooms: Vec<(data::RoomName, u32)>,
    /// The current game time of the server when the call was completed, the tick up to which pvp has been reported.
    pub reported_up_to: u32,
    /// Phantom data in order to allow adding any additional fields in the future.
    _non_exhaustive: (),
}

impl EndpointResult for RecentPvp {
    type RequestResult = Response;
    type ErrorResult = data::ApiError;

    fn from_raw(raw: Response) -> Result<RecentPvp> {
        let Response { ok, pvp } = raw;

        if ok != 1 {
            return Err(ApiError::NotOk(ok).into());
        }

        Ok(RecentPvp {
            shards: pvp
                .into_iter()
                .map(|(name, data)| {
                    Ok((
                        name,
                        ShardRecentPvp {
                            rooms: data
                                .rooms
                                .into_iter()
                                .map(|r| Ok((data::RoomName::new(&r._id)?, r.last_pvp_time)))
                                .collect::<Result<_>>()?,
                            reported_up_to: data.time,
                            _non_exhaustive: (),
                        },
                    ))
                })
                .collect::<Result<_>>()?,
            _non_exhaustive: (),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RecentPvp;
    use crate::EndpointResult;
    use serde_json;

    fn test_parse(json: serde_json::Value) {
        let response = serde_json::from_value(json).unwrap();

        let _ = RecentPvp::from_raw(response).unwrap();
    }

    #[test]
    fn parse_sample_pvp() {
        test_parse(json! ({
            "ok": 1,
            "pvp": {
                "shard0": {
                    "time": 20656327,
                    "rooms": [
                        {
                            "_id": "E5N39",
                            "lastPvpTime": 20656327,
                        },
                        {
                            "_id": "W15S23",
                            "lastPvpTime": 20656326,
                        },
                        {
                            "_id": "W63S12",
                            "lastPvpTime": 20656326,
                        },
                        {
                            "_id": "W1N20",
                            "lastPvpTime": 20656323,
                        },
                        {
                            "_id": "W54N97",
                            "lastPvpTime": 20656322,
                        }
                    ]
                },
                "shard1": {
                    "time": 265413,
                    "rooms": [
                        {
                            "_id": "E2S3",
                            "lastPvpTime": 265412,
                        }
                    ]
                }
            }
        }));
    }
}

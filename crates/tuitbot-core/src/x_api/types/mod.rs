//! X API v2 request and response types.
//!
//! All types derive Serde traits and match the X API v2 JSON field names.
//! Tweet IDs are strings because X API v2 returns them as strings and
//! some IDs exceed `i64` range.

mod tweet_types;
mod user_types;
mod api_types;

pub use tweet_types::{
    Tweet, PublicMetrics, SearchResponse, SearchMeta, MentionResponse,
    PostTweetRequest, ReplyTo, PostTweetResponse, PostedTweet,
    SingleTweetResponse, DeleteTweetResponse, DeleteTweetData, Includes,
};

pub use user_types::{
    User, UserMetrics, UserResponse, UsersResponse, UsersMeta, FollowUserRequest,
};

pub use api_types::{
    ImageFormat, MediaType, MediaId, MediaPayload, RateLimitInfo,
    RawApiResponse, XApiErrorResponse, LikeTweetRequest, BookmarkTweetRequest,
    RetweetRequest, ActionResultResponse, ActionResultData,
};

#[cfg(test)]
mod tests;

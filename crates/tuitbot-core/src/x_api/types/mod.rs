//! X API v2 request and response types.
//!
//! All types derive Serde traits and match the X API v2 JSON field names.
//! Tweet IDs are strings because X API v2 returns them as strings and
//! some IDs exceed `i64` range.

mod api_types;
mod tweet_types;
mod user_types;

pub use tweet_types::{
    DeleteTweetData, DeleteTweetResponse, Includes, MentionResponse, PostTweetRequest,
    PostTweetResponse, PostedTweet, PublicMetrics, ReplyTo, SearchMeta, SearchResponse,
    SingleTweetResponse, Tweet,
};

pub use user_types::{
    FollowUserRequest, User, UserMetrics, UserResponse, UsersMeta, UsersResponse,
};

pub use api_types::{
    ActionResultData, ActionResultResponse, BookmarkTweetRequest, ImageFormat, LikeTweetRequest,
    MediaId, MediaPayload, MediaType, RateLimitInfo, RawApiResponse, RetweetRequest,
    XApiErrorResponse,
};

#[cfg(test)]
mod tests;

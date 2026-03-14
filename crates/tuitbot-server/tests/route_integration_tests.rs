//! Route handler integration test suite (Task 3.4).
//!
//! Entry point that collects all api_tests sub-modules into a single
//! integration test binary.  Each sub-module tests a specific route group.
//!
//! Sub-modules:
//!   analytics     — /api/analytics/* (summary, followers, topics, recent-performance)
//!   approval      — /api/approval/*  (list, approve, reject, edit, approve-all)
//!   compose       — /api/content/*   (tweet compose, thread compose, targets)
//!   content       — /api/settings/* + content generator isolation
//!   discovery     — /api/connectors/* (OAuth, credential isolation)
//!   discovery_feed — /api/discovery/* (feed, keywords, queue-reply) [Task 3.4]

mod api_tests;

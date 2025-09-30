pub const MAX_BPS:u64 = 10_000; // 100%

pub const HASH_LENGTH:usize = 32;

// type HASHTYPE = [u8;HASH_LENGTH];

pub const MAX_MOVE_COUNT:usize = 64;

pub const MAX_MOVE_TYPE_COUNT:usize = 8;

pub const MAX_METADATA_LENGTH:usize = 64;

pub const DEFAULT_OFFSET:i64 = 60 * 60 * 36; // 36 hours

pub const INITIALIZE_GAME_ACTION:&str = "initialize-game";

pub const FINALIZE_WIN_ACTION:&str = "finalize-win";

pub const FINALIZE_LOSS_ACTION:&str = "finalize-loss";

pub const PRIVATE_SEED:&str = "private";

pub const PUBLIC_SEED:&str = "public";

pub const MOVE_TYPE_COUNT_SEED:&str = "move-type-count";
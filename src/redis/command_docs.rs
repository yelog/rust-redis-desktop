#[derive(Clone, PartialEq)]
pub struct RedisCommand {
    pub name: &'static str,
    pub syntax: &'static str,
    pub description: &'static str,
    pub group: &'static str,
}

pub const REDIS_COMMANDS: &[RedisCommand] = &[
    RedisCommand { name: "SET", syntax: "SET key value [NX|XX] [GET] [EX seconds|PX milliseconds|EXAT unix-time-seconds|PXAT unix-time-milliseconds|KEEPTTL]", description: "Set key to hold the string value", group: "string" },
    RedisCommand { name: "GET", syntax: "GET key", description: "Get the value of a key", group: "string" },
    RedisCommand { name: "DEL", syntax: "DEL key [key ...]", description: "Delete a key", group: "generic" },
    RedisCommand { name: "EXISTS", syntax: "EXISTS key [key ...]", description: "Determine if a key exists", group: "generic" },
    RedisCommand { name: "EXPIRE", syntax: "EXPIRE key seconds [NX|XX|GT|LT]", description: "Set a key's time to live in seconds", group: "generic" },
    RedisCommand { name: "TTL", syntax: "TTL key", description: "Get the time to live for a key in seconds", group: "generic" },
    RedisCommand { name: "PERSIST", syntax: "PERSIST key", description: "Remove the expiration from a key", group: "generic" },
    RedisCommand { name: "KEYS", syntax: "KEYS pattern", description: "Find all keys matching the given pattern", group: "generic" },
    RedisCommand { name: "SCAN", syntax: "SCAN cursor [MATCH pattern] [COUNT count] [TYPE type]", description: "Incrementally iterate the keys space", group: "generic" },
    RedisCommand { name: "TYPE", syntax: "TYPE key", description: "Determine the type stored at key", group: "generic" },
    RedisCommand { name: "RENAME", syntax: "RENAME key newkey", description: "Rename a key", group: "generic" },
    RedisCommand { name: "HSET", syntax: "HSET key field value [field value ...]", description: "Set the string value of a hash field", group: "hash" },
    RedisCommand { name: "HGET", syntax: "HGET key field", description: "Get the value of a hash field", group: "hash" },
    RedisCommand { name: "HGETALL", syntax: "HGETALL key", description: "Get all the fields and values in a hash", group: "hash" },
    RedisCommand { name: "HDEL", syntax: "HDEL key field [field ...]", description: "Delete one or more hash fields", group: "hash" },
    RedisCommand { name: "HEXISTS", syntax: "HEXISTS key field", description: "Determine if a hash field exists", group: "hash" },
    RedisCommand { name: "HKEYS", syntax: "HKEYS key", description: "Get all the fields in a hash", group: "hash" },
    RedisCommand { name: "HLEN", syntax: "HLEN key", description: "Get the number of fields in a hash", group: "hash" },
    RedisCommand { name: "LPUSH", syntax: "LPUSH key element [element ...]", description: "Prepend one or more elements to a list", group: "list" },
    RedisCommand { name: "RPUSH", syntax: "RPUSH key element [element ...]", description: "Append one or more elements to a list", group: "list" },
    RedisCommand { name: "LPOP", syntax: "LPOP key [count]", description: "Remove and get the first elements in a list", group: "list" },
    RedisCommand { name: "RPOP", syntax: "RPOP key [count]", description: "Remove and get the last elements in a list", group: "list" },
    RedisCommand { name: "LRANGE", syntax: "LRANGE key start stop", description: "Get a range of elements from a list", group: "list" },
    RedisCommand { name: "LLEN", syntax: "LLEN key", description: "Get the length of a list", group: "list" },
    RedisCommand { name: "LINDEX", syntax: "LINDEX key index", description: "Get an element from a list by its index", group: "list" },
    RedisCommand { name: "LSET", syntax: "LSET key index element", description: "Set the value of an element in a list by its index", group: "list" },
    RedisCommand { name: "SADD", syntax: "SADD key member [member ...]", description: "Add one or more members to a set", group: "set" },
    RedisCommand { name: "SREM", syntax: "SREM key member [member ...]", description: "Remove one or more members from a set", group: "set" },
    RedisCommand { name: "SMEMBERS", syntax: "SMEMBERS key", description: "Get all the members in a set", group: "set" },
    RedisCommand { name: "SISMEMBER", syntax: "SISMEMBER key member", description: "Determine if a given value is a member of a set", group: "set" },
    RedisCommand { name: "SCARD", syntax: "SCARD key", description: "Get the number of members in a set", group: "set" },
    RedisCommand { name: "ZADD", syntax: "ZADD key [NX|XX] [GT|LT] [CH] [INCR] score member [score member ...]", description: "Add one or more members to a sorted set", group: "sorted_set" },
    RedisCommand { name: "ZREM", syntax: "ZREM key member [member ...]", description: "Remove one or more members from a sorted set", group: "sorted_set" },
    RedisCommand { name: "ZRANGE", syntax: "ZRANGE key start stop [BYSCORE | BYLEX] [REV] [LIMIT offset count] [WITHSCORES]", description: "Return a range of members in a sorted set", group: "sorted_set" },
    RedisCommand { name: "ZCARD", syntax: "ZCARD key", description: "Get the number of members in a sorted set", group: "sorted_set" },
    RedisCommand { name: "ZSCORE", syntax: "ZSCORE key member", description: "Get the score associated with the given member in a sorted set", group: "sorted_set" },
    RedisCommand { name: "XADD", syntax: "XADD key [NOMKSTREAM] [<MAXLEN|MINID> [=|~] threshold [LIMIT count]] *|ID field value [field value ...]", description: "Append a new entry to a stream", group: "stream" },
    RedisCommand { name: "XRANGE", syntax: "XRANGE key start end [COUNT count]", description: "Return a range of elements in a stream", group: "stream" },
    RedisCommand { name: "XLEN", syntax: "XLEN key", description: "Return the number of entries in a stream", group: "stream" },
    RedisCommand { name: "XDEL", syntax: "XDEL key ID [ID ...]", description: "Remove the specified entries from the stream", group: "stream" },
    RedisCommand { name: "INFO", syntax: "INFO [section [section ...]]", description: "Get information and statistics about the server", group: "server" },
    RedisCommand { name: "DBSIZE", syntax: "DBSIZE", description: "Return the number of keys in the selected database", group: "server" },
    RedisCommand { name: "FLUSHDB", syntax: "FLUSHDB [ASYNC|SYNC]", description: "Remove all keys from the current database", group: "server" },
    RedisCommand { name: "FLUSHALL", syntax: "FLUSHALL [ASYNC|SYNC]", description: "Remove all keys from all databases", group: "server" },
    RedisCommand { name: "CLIENT", syntax: "CLIENT subcommand [arguments]", description: "A container for client connection commands", group: "server" },
    RedisCommand { name: "SLOWLOG", syntax: "SLOWLOG subcommand [argument]", description: "Manages the Redis slow queries log", group: "server" },
    RedisCommand { name: "CONFIG", syntax: "CONFIG subcommand [argument]", description: "Configure Redis server parameters", group: "server" },
    RedisCommand { name: "PING", syntax: "PING [message]", description: "Ping the server", group: "connection" },
    RedisCommand { name: "ECHO", syntax: "ECHO message", description: "Echo the given string", group: "connection" },
    RedisCommand { name: "SELECT", syntax: "SELECT index", description: "Change the selected database for the current connection", group: "connection" },
    RedisCommand { name: "AUTH", syntax: "AUTH [username] password", description: "Authenticate to the server", group: "connection" },
    RedisCommand { name: "QUIT", syntax: "QUIT", description: "Close the connection", group: "connection" },
    RedisCommand { name: "MULTI", syntax: "MULTI", description: "Mark the start of a transaction block", group: "transactions" },
    RedisCommand { name: "EXEC", syntax: "EXEC", description: "Execute all commands issued after MULTI", group: "transactions" },
    RedisCommand { name: "DISCARD", syntax: "DISCARD", description: "Discard all commands issued after MULTI", group: "transactions" },
    RedisCommand { name: "WATCH", syntax: "WATCH key [key ...]", description: "Watch the given keys to determine execution of the MULTI/EXEC block", group: "transactions" },
    RedisCommand { name: "UNWATCH", syntax: "UNWATCH", description: "Forget about all watched keys", group: "transactions" },
    RedisCommand { name: "INCR", syntax: "INCR key", description: "Increment the integer value of a key by one", group: "string" },
    RedisCommand { name: "DECR", syntax: "DECR key", description: "Decrement the integer value of a key by one", group: "string" },
    RedisCommand { name: "INCRBY", syntax: "INCRBY key increment", description: "Increment the integer value of a key by the given amount", group: "string" },
    RedisCommand { name: "APPEND", syntax: "APPEND key value", description: "Append a value to a key", group: "string" },
    RedisCommand { name: "STRLEN", syntax: "STRLEN key", description: "Get the length of the value stored in a key", group: "string" },
    RedisCommand { name: "MGET", syntax: "MGET key [key ...]", description: "Get the values of all the given keys", group: "string" },
    RedisCommand { name: "MSET", syntax: "MSET key value [key value ...]", description: "Set multiple keys to multiple values", group: "string" },
];

pub fn find_commands(prefix: &str) -> Vec<&'static RedisCommand> {
    let upper = prefix.to_uppercase();
    REDIS_COMMANDS
        .iter()
        .filter(|cmd| cmd.name.starts_with(&upper))
        .take(10)
        .collect()
}

pub fn find_command(name: &str) -> Option<&'static RedisCommand> {
    let upper = name.to_uppercase();
    REDIS_COMMANDS.iter().find(|cmd| cmd.name == upper)
}

export const PAGINATION = {
    DEFAULT_LIMIT: 20,
    MAX_LIMIT: 1000,
    MIN_PAGE: 1,
} as const;

export const REDIS_KEYS = {
    MERKLE_TREE_PREFIX: 'merkle_tree:',
    MERKLE_LAST_INDEX_PREFIX: 'merkle_last_index:',
} as const;

import type { OpenAPIV3 } from 'openapi-types';

export const openApiSpec: OpenAPIV3.Document = {
    openapi: '3.0.3',
    info: {
        title: 'Vortex API',
        version: '1.0.0',
        description: 'API for Vortex privacy protocol on Sui blockchain',
    },
    servers: [
        { url: 'http://localhost:5005', description: 'Development' },
        { url: 'https://api.vortexfi.xyz', description: 'Production' },
    ],
    tags: [
        { name: 'Health', description: 'Health check endpoints' },
        { name: 'Accounts', description: 'Vortex account management' },
        { name: 'Pools', description: 'Privacy pool queries' },
        { name: 'Commitments', description: 'Commitment queries' },
        { name: 'Merkle', description: 'Merkle tree operations' },
        { name: 'Relayer', description: 'Relayer information' },
        { name: 'Transactions', description: 'Sponsored transaction execution' },
    ],
    paths: {
        '/api/health': {
            get: {
                tags: ['Health'],
                summary: 'Health check',
                description: 'Check the health status of all services (MongoDB, Redis, Sui)',
                responses: {
                    '200': {
                        description: 'All services healthy',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/HealthResponse' },
                            },
                        },
                    },
                    '503': {
                        description: 'One or more services unhealthy',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/HealthResponse' },
                            },
                        },
                    },
                },
            },
        },
        '/api/v1/accounts': {
            get: {
                tags: ['Accounts'],
                summary: 'Get accounts by hashed secret',
                parameters: [
                    {
                        name: 'hashed_secret',
                        in: 'query',
                        required: true,
                        description: 'Poseidon hash of the secret (decimal string)',
                        schema: { type: 'string', pattern: '^[0-9]+$' },
                    },
                ],
                responses: {
                    '200': {
                        description: 'List of accounts',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/AccountsResponse' },
                            },
                        },
                    },
                    '400': {
                        description: 'Invalid request',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/ErrorResponse' },
                            },
                        },
                    },
                },
            },
            post: {
                tags: ['Accounts'],
                summary: 'Create a new Vortex account',
                description: 'Creates a new VortexAccount on-chain via sponsored transaction',
                requestBody: {
                    required: true,
                    content: {
                        'application/json': {
                            schema: { $ref: '#/components/schemas/CreateAccountRequest' },
                        },
                    },
                },
                responses: {
                    '201': {
                        description: 'Account created',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/AccountResponse' },
                            },
                        },
                    },
                    '400': {
                        description: 'Invalid request',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/ErrorResponse' },
                            },
                        },
                    },
                    '500': {
                        description: 'Server error',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/ErrorResponse' },
                            },
                        },
                    },
                },
            },
        },
        '/api/v1/pools': {
            get: {
                tags: ['Pools'],
                summary: 'Get privacy pools',
                parameters: [
                    {
                        name: 'page',
                        in: 'query',
                        schema: { type: 'integer', minimum: 1, default: 1 },
                    },
                    {
                        name: 'limit',
                        in: 'query',
                        schema: { type: 'integer', minimum: 1, maximum: 100, default: 20 },
                    },
                    {
                        name: 'coin_type',
                        in: 'query',
                        description: 'Filter by coin type (e.g., 0x2::sui::SUI)',
                        schema: { type: 'string' },
                    },
                ],
                responses: {
                    '200': {
                        description: 'Paginated list of pools',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/PoolsResponse' },
                            },
                        },
                    },
                },
            },
        },
        '/api/v1/commitments': {
            get: {
                tags: ['Commitments'],
                summary: 'Get commitments',
                parameters: [
                    {
                        name: 'coin_type',
                        in: 'query',
                        required: true,
                        description: 'Coin type (e.g., 0x2::sui::SUI)',
                        schema: { type: 'string', pattern: '^0x[a-fA-F0-9]+::\\w+::\\w+$' },
                    },
                    {
                        name: 'index',
                        in: 'query',
                        required: true,
                        description: 'Starting index',
                        schema: { type: 'integer', minimum: 0 },
                    },
                    {
                        name: 'op',
                        in: 'query',
                        description: 'Comparison operator',
                        schema: {
                            type: 'string',
                            enum: ['gt', 'gte', 'lt', 'lte'],
                            default: 'gte',
                        },
                    },
                    {
                        name: 'page',
                        in: 'query',
                        schema: { type: 'integer', minimum: 1, default: 1 },
                    },
                    {
                        name: 'limit',
                        in: 'query',
                        schema: { type: 'integer', minimum: 1, maximum: 500, default: 100 },
                    },
                ],
                responses: {
                    '200': {
                        description: 'Paginated list of commitments',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/CommitmentsResponse' },
                            },
                        },
                    },
                },
            },
        },
        '/api/v1/merkle/path': {
            post: {
                tags: ['Merkle'],
                summary: 'Get Merkle path for a commitment',
                description: 'Returns the Merkle path and root for proving ownership of a UTXO',
                requestBody: {
                    required: true,
                    content: {
                        'application/json': {
                            schema: { $ref: '#/components/schemas/MerklePathRequest' },
                        },
                    },
                },
                responses: {
                    '200': {
                        description: 'Merkle path',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/MerklePathResponse' },
                            },
                        },
                    },
                    '400': {
                        description: 'Invalid request',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/ErrorResponse' },
                            },
                        },
                    },
                },
            },
        },
        '/api/v1/relayer': {
            get: {
                tags: ['Relayer'],
                summary: 'Get relayer address',
                description: 'Returns the Sui public address of the relayer',
                responses: {
                    '200': {
                        description: 'Relayer address',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/RelayerResponse' },
                            },
                        },
                    },
                },
            },
        },
        '/api/v1/transactions': {
            post: {
                tags: ['Transactions'],
                summary: 'Execute a sponsored transaction',
                description:
                    'Takes transaction bytes from client, rebuilds, sponsors with Shinami, and executes',
                security: [{ ApiKeyAuth: [] }],
                requestBody: {
                    required: true,
                    content: {
                        'application/json': {
                            schema: { $ref: '#/components/schemas/ExecuteTransactionRequest' },
                        },
                    },
                },
                responses: {
                    '201': {
                        description: 'Transaction executed',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/TransactionResponse' },
                            },
                        },
                    },
                    '400': {
                        description: 'Invalid request',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/ErrorResponse' },
                            },
                        },
                    },
                    '401': {
                        description: 'Missing or invalid API key',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/ErrorResponse' },
                            },
                        },
                    },
                    '500': {
                        description: 'Transaction failed',
                        content: {
                            'application/json': {
                                schema: { $ref: '#/components/schemas/ErrorResponse' },
                            },
                        },
                    },
                },
            },
        },
    },
    components: {
        securitySchemes: {
            ApiKeyAuth: {
                type: 'apiKey',
                in: 'header',
                name: 'x-api-key',
                description: 'API key required for transaction execution',
            },
        },
        schemas: {
            ErrorResponse: {
                type: 'object',
                properties: {
                    success: { type: 'boolean', example: false },
                    error: { type: 'string' },
                },
                required: ['success', 'error'],
            },
            HealthResponse: {
                type: 'object',
                properties: {
                    success: { type: 'boolean', example: true },
                    data: {
                        type: 'object',
                        properties: {
                            status: { type: 'string', enum: ['healthy', 'degraded'] },
                            services: {
                                type: 'object',
                                properties: {
                                    mongodb: { type: 'string', enum: ['healthy', 'unhealthy'] },
                                    redis: { type: 'string', enum: ['healthy', 'unhealthy'] },
                                    sui: { type: 'string', enum: ['healthy', 'unhealthy'] },
                                },
                            },
                            timestamp: { type: 'string', format: 'date-time' },
                        },
                    },
                },
            },
            CreateAccountRequest: {
                type: 'object',
                properties: {
                    owner: {
                        type: 'string',
                        description: 'Sui address (66 characters)',
                        example:
                            '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
                    },
                    hashedSecret: {
                        type: 'string',
                        description: 'Poseidon hash of secret (decimal string)',
                        example: '12345678901234567890',
                    },
                },
                required: ['owner', 'hashedSecret'],
            },
            Account: {
                type: 'object',
                properties: {
                    id: { type: 'string', description: 'Unique identifier' },
                    objectId: { type: 'string', description: 'Sui object ID' },
                    hashedSecret: { type: 'string' },
                    owner: { type: 'string' },
                    createdAt: { type: 'string', format: 'date-time' },
                    txDigest: { type: 'string' },
                },
            },
            AccountResponse: {
                type: 'object',
                properties: {
                    success: { type: 'boolean', example: true },
                    data: { $ref: '#/components/schemas/Account' },
                },
            },
            AccountsResponse: {
                type: 'object',
                properties: {
                    success: { type: 'boolean', example: true },
                    data: {
                        type: 'array',
                        items: { $ref: '#/components/schemas/Account' },
                    },
                },
            },
            Pool: {
                type: 'object',
                properties: {
                    id: { type: 'string', description: 'Unique identifier' },
                    digest: { type: 'string' },
                    sender: { type: 'string' },
                    checkpoint: { type: 'integer' },
                    checkpointTimestampMs: { type: 'integer' },
                    objectId: { type: 'string', description: 'Sui object ID' },
                    coinType: { type: 'string' },
                },
            },
            Pagination: {
                type: 'object',
                properties: {
                    page: { type: 'integer' },
                    limit: { type: 'integer' },
                    total: { type: 'integer' },
                    totalPages: { type: 'integer' },
                    hasNext: { type: 'boolean' },
                    hasPrev: { type: 'boolean' },
                },
            },
            PoolsResponse: {
                type: 'object',
                properties: {
                    success: { type: 'boolean', example: true },
                    data: {
                        type: 'object',
                        properties: {
                            items: {
                                type: 'array',
                                items: { $ref: '#/components/schemas/Pool' },
                            },
                            pagination: { $ref: '#/components/schemas/Pagination' },
                        },
                    },
                },
            },
            Commitment: {
                type: 'object',
                properties: {
                    id: { type: 'string', description: 'Unique identifier' },
                    digest: { type: 'string' },
                    sender: { type: 'string' },
                    checkpoint: { type: 'integer' },
                    checkpointTimestampMs: { type: 'integer' },
                    coinType: { type: 'string' },
                    index: { type: 'integer' },
                    commitment: { type: 'string' },
                    encryptedOutput: { type: 'array', items: { type: 'integer' } },
                },
            },
            CommitmentsResponse: {
                type: 'object',
                properties: {
                    success: { type: 'boolean', example: true },
                    data: {
                        type: 'object',
                        properties: {
                            items: {
                                type: 'array',
                                items: { $ref: '#/components/schemas/Commitment' },
                            },
                            pagination: { $ref: '#/components/schemas/Pagination' },
                        },
                    },
                },
            },
            MerklePathRequest: {
                type: 'object',
                properties: {
                    coin_type: {
                        type: 'string',
                        description: 'Coin type',
                        example: '0x2::sui::SUI',
                    },
                    index: {
                        type: 'integer',
                        description: 'Commitment index in the tree',
                        minimum: 0,
                    },
                    amount: {
                        type: 'string',
                        description: 'UTXO amount (decimal string)',
                    },
                    public_key: {
                        type: 'string',
                        description: 'Public key (decimal string)',
                    },
                    blinding: {
                        type: 'string',
                        description: 'Blinding factor (decimal string)',
                    },
                    vortex_pool: {
                        type: 'string',
                        description: 'Vortex pool object ID',
                    },
                },
                required: ['coin_type', 'index', 'amount', 'public_key', 'blinding', 'vortex_pool'],
            },
            MerklePathResponse: {
                type: 'object',
                properties: {
                    success: { type: 'boolean', example: true },
                    data: {
                        type: 'object',
                        properties: {
                            path: {
                                type: 'array',
                                items: {
                                    type: 'array',
                                    items: { type: 'string' },
                                    minItems: 2,
                                    maxItems: 2,
                                },
                                description: 'Array of [left, right] hash pairs',
                            },
                            root: { type: 'string', description: 'Merkle root' },
                        },
                    },
                },
            },
            ExecuteTransactionRequest: {
                type: 'object',
                properties: {
                    txBytes: {
                        type: 'string',
                        description: 'Hex encoded transaction bytes from transaction.build()',
                    },
                },
                required: ['txBytes'],
            },
            TransactionResponse: {
                type: 'object',
                properties: {
                    success: { type: 'boolean', example: true },
                    data: {
                        type: 'object',
                        properties: {
                            digest: { type: 'string', description: 'Transaction digest' },
                        },
                    },
                },
            },
            RelayerResponse: {
                type: 'object',
                properties: {
                    success: { type: 'boolean', example: true },
                    data: {
                        type: 'object',
                        properties: {
                            address: {
                                type: 'string',
                                description: 'Sui address of the relayer',
                                example:
                                    '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
                            },
                        },
                    },
                },
            },
        },
    },
};

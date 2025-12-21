import type { Context } from 'hono';
import type {
    PoolsRepository,
    AccountsRepository,
    CommitmentsRepository,
} from '@/repositories/index.ts';
import type {
    AccountsService,
    HealthService,
    MerkleService,
    RelayerService,
    TransactionsService,
} from '@/services/index.ts';

export type AppBindings = {
    Variables: {
        pools: PoolsRepository;
        accounts: AccountsRepository;
        commitments: CommitmentsRepository;
        accountsService: AccountsService;
        healthService: HealthService;
        merkleService: MerkleService;
        relayerService: RelayerService;
        transactionsService: TransactionsService;
    };
};

export type AppContext = Context<AppBindings>;

export type ApiResponse<T> = { success: true; data: T } | { success: false; error: string };

export type Pagination = {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
};

export type PaginatedResponse<T> = {
    items: T[];
    pagination: Pagination;
};

export const buildPaginatedResponse = <T, D>(
    docs: D[],
    mapper: (doc: D) => T,
    params: { page: number; limit: number; total: number }
): PaginatedResponse<T> => {
    const totalPages = Math.ceil(params.total / params.limit);

    return {
        items: docs.map(mapper),
        pagination: {
            page: params.page,
            limit: params.limit,
            total: params.total,
            totalPages,
            hasNext: params.page < totalPages,
            hasPrev: params.page > 1,
        },
    };
};

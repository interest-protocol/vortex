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

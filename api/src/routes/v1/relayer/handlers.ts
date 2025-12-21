import type { Context } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { withErrorHandler } from '@/utils/handler.ts';

const getRelayerAddressHandler = (c: Context<AppBindings>) => {
    const relayerService = c.get('relayerService');
    const address = relayerService.getAddress();

    return Promise.resolve(c.json({ success: true, data: { address } }));
};

export const getRelayerAddress = withErrorHandler(
    getRelayerAddressHandler,
    'Failed to get relayer address'
);

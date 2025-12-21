import { Hono } from 'hono';
import type { AppBindings } from '@/types/index.ts';
import { getRelayerAddress } from './handlers.ts';

export const relayerRoutes = new Hono<AppBindings>().get('/', getRelayerAddress);

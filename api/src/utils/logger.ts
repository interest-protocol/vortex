import pino from 'pino';
import { env } from '@/config/env.js';

const baseConfig = {
    level: env.NODE_ENV === 'production' ? 'info' : 'debug',
    formatters: {
        level: (label: string) => ({ level: label }),
    },
    timestamp: pino.stdTimeFunctions.isoTime,
};

export const logger =
    env.NODE_ENV === 'development'
        ? pino({
              ...baseConfig,
              transport: {
                  target: 'pino/file',
                  options: { destination: 1 },
              },
          })
        : pino(baseConfig);

import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';

export default [
    eslint.configs.recommended,
    ...tseslint.configs.strictTypeChecked,
    ...tseslint.configs.stylisticTypeChecked,
    {
        languageOptions: {
            parserOptions: {
                projectService: {
                    allowDefaultProject: ['eslint.config.mjs', 'commitlint.config.js'],
                },
                tsconfigRootDir: import.meta.dirname,
            },
        },
        rules: {
            '@typescript-eslint/no-unused-vars': [
                'error',
                { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
            ],
            '@typescript-eslint/consistent-type-imports': ['error', { prefer: 'type-imports' }],
            '@typescript-eslint/no-misused-promises': [
                'error',
                { checksVoidReturn: { attributes: false } },
            ],
            '@typescript-eslint/consistent-type-definitions': ['error', 'type'],
        },
    },
    {
        ignores: ['dist/**', 'node_modules/**'],
    },
];

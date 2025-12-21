export default {
    extends: ['@commitlint/config-conventional'],
    parserPreset: {
        parserOpts: {
            headerPattern: /^(?:\p{Emoji_Presentation}|\p{Extended_Pictographic})*\s*(\w+)(?:\((.+)\))?:\s*(.+)$/u,
            headerCorrespondence: ['type', 'scope', 'subject'],
        },
    },
    rules: {
        'type-enum': [
            2,
            'always',
            [
                'feat',
                'fix',
                'docs',
                'style',
                'refactor',
                'perf',
                'test',
                'build',
                'ci',
                'chore',
                'revert',
            ],
        ],
        'subject-case': [2, 'always', 'lower-case'],
        'subject-empty': [2, 'never'],
        'type-empty': [2, 'never'],
    },
};

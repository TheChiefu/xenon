export type MessagePart = 
    | {kind: 'text'; content: string;
        bold?: boolean;
        italic?: boolean;
        strikethrough?: boolean;
    }
    | {kind: 'code'; content: string}
    | {kind: 'block'; content: string}
    | {kind: 'link'; href: string}
    | {kind: 'mention'; userId: string};

// Regex Definitions
const LINK: RegExp = /https?:\/\/[^\s<>"']+/g;
const MENTION: RegExp = /@([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/gi;
const BOLD_ITALIC: RegExp = /\*\*\*([^*]+)\*\*\*|(?<!\w)___([^_]+?)___(?!\w)/g;
const BOLD: RegExp = /\*\*([^*]+)\*\*|(?<!\w)__([^_]+?)__(?!\w)/g;
const ITALIC: RegExp = /\*([^*\s][^*]*?)\*|(?<!\w)_([^_\s][^_]*?)_(?!\w)/g;
const STRIKETHROUGH: RegExp = /~~([^~]+)~~/g;
const BLOCK_CODE: RegExp = /```([\s\S]*?)```/g;
const INLINE_CODE: RegExp = /`([^`\n]+)`/g;

const PATTERNS: { regex: RegExp; toPart: (match: RegExpMatchArray) => MessagePart }[] = [
    { regex: BLOCK_CODE,     toPart: (m) => ({ kind: 'block', content: m[1] }) },
    { regex: INLINE_CODE,    toPart: (m) => ({ kind: 'code', content: m[1] }) },
    { regex: STRIKETHROUGH,  toPart: (m) => ({ kind: 'text', strikethrough: true, content: m[1] }) },
    { regex: LINK,           toPart: (m) => ({ kind: 'link', href: m[0] }) },
    { regex: MENTION,        toPart: (m) => ({ kind: 'mention', userId: m[1] }) },
    { regex: BOLD_ITALIC,    toPart: (m) => ({ kind: 'text', bold: true, italic: true, content: m[1] ?? m[2] }) },
    { regex: BOLD,           toPart: (m) => ({ kind: 'text', bold: true, content: m[1] ?? m[2] }) },
    { regex: ITALIC,         toPart: (m) => ({ kind: 'text', italic: true, content: m[1] ?? m[2] }) },
];

function parse(body: string): MessagePart[] {
    type Match = {
        start: number;
        end: number;
        part: MessagePart
    };

    const matches: Match[] = [];

    for (const pattern of PATTERNS) {
        const found = body.matchAll(pattern.regex);
        for (const match of found) {
            const part = pattern.toPart(match);
            const start = match.index;
            const end = start + match[0].length;
            matches.push({ start: start, end: end, part: part });
        }
    }

    return matches;
}
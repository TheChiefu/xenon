// Message Parts //

interface Text {
    kind: 'text';
    content: string;
    bold?: boolean;
    italic?: boolean;
    strikethrough?: boolean
}

interface Code {
    kind: 'code';
    content: string
}

interface Block {
    kind: 'block';
    content: string
}

interface Link {
    kind: 'link';
    href: string
}

interface Mention {
    kind: 'mention';
    userId: string
}

export type MessagePart = Text | Code | Block | Link | Mention;


// Regex Definitions //
const LINK: RegExp = /https?:\/\/[^\s<>"']+/y;
const MENTION: RegExp = /@([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/iy;
const BOLD_ITALIC: RegExp = /\*\*\*([^*]+)\*\*\*|(?<!\w)___([^_]+?)___(?!\w)/y;
const BOLD: RegExp = /\*\*([^*]+)\*\*|(?<!\w)__([^_]+?)__(?!\w)/y;
const ITALIC: RegExp = /\*([^*\s][^*]*?)\*|(?<!\w)_([^_\s][^_]*?)_(?!\w)/y;
const STRIKETHROUGH: RegExp = /~~([^~]+)~~/y;
const BLOCK_CODE: RegExp = /```([\s\S]*?)```/y;
const INLINE_CODE: RegExp = /`([^`\n]+)`/y;

// Extract the matched pattern and return a MessagePart object
// For cases that have 2 capture groups, 1st is preferred, 2nd is fallback
// For cases that have 1 capture group, the whole group is returned as the content
function patternToMsgPart(pattern: RegExp, match: RegExpExecArray): MessagePart {
    switch (pattern) {
        case BLOCK_CODE: return {
            kind: 'block',
            content: match[1] // ```code block```
        };
        case INLINE_CODE: return {
            kind: 'code',
            content: match[1] // `code`
        };
        case LINK: return {
            kind: 'link',
            href: match[0] // http(s)
        };
        case MENTION: return {
            kind: 'mention',
            userId: match[1] // @
        };
        case BOLD_ITALIC: return {
            kind: 'text',
            bold: true,
            italic: true,
            content: match[1] ?? match[2] // *** OR ___
        };
        case BOLD: return {
            kind: 'text',
            bold: true,
            content: match[1] ?? match[2] // ** OR __
        };
        case ITALIC: return {
            kind: 'text',
            italic: true,
            content: match[1] ?? match[2] // * OR _
        };
        case STRIKETHROUGH: return {
            kind: 'text',
            strikethrough: true,
            content: match[1]
        };
        default: throw new Error('Unreachable: unrecognized pattern');
    }
}

// List of all regex patterns to match against the message body
const CONSTRUCTS: RegExp[] = [BLOCK_CODE, INLINE_CODE, LINK, MENTION, BOLD_ITALIC, BOLD, ITALIC, STRIKETHROUGH];

// Parses a message body into an array of MessagePart objects
export function parse(body: string): MessagePart[] {

    // Initialize an empty array to hold the parsed message parts
    const parts: MessagePart[] = [];
    let i = 0;
    let textStart = 0;

    // Iterate through the message body by character
    while (i < body.length) {
        let matched = false;

        // Iterate through each regex pattern to find a match in the message body
        for (const pattern of CONSTRUCTS) {

            // Reset the lastIndex of the regex pattern to the current index
            pattern.lastIndex = i;
            const match = pattern.exec(body);
            if (!match) continue;

            // If a match is found, push the text before the match as a text part
            if (i > textStart) {
                parts.push({
                    kind: 'text',
                    content: body.slice(textStart, i)
                });
            }

            // Push the matched pattern as a MessagePart object
            parts.push(patternToMsgPart(pattern, match));

            // Update index/textStart to continue parsing after matched pattern
            i = textStart = i + match[0].length;
            matched = true;
            break;
        }

        // If no match is found, increment the index to continue parsing
        if (!matched) i++;
    }

    // If there is any remaining text after the last match, push it as a text part
    if (i > textStart) {
        parts.push({
            kind: 'text',
            content: body.slice(textStart, i)
        });
    }

    return parts;
}

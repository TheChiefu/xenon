const EMBED_ORIGIN: string = "https://www.youtube-nocookie.com/embed/";

const VIDEO_ID: RegExp = /^[A-Za-z0-9_-]{11}$/;

const TIMESTAMP: RegExp = /^(?:(\d+)h)?(?:(\d+)m)?(?:(\d+)s?)?$/;

const HOSTS: string[] = [
    "youtube.com",
    "www.youtube.com",
    "m.youtube.com",
    "music.youtube.com",
    "youtube-nocookie.com",
    "www.youtube-nocookie.com",
];

const SHORT_HOSTS: string[] = ["youtu.be", "www.youtu.be"];

const PATH_PREFIXES: string[] = ["embed", "shorts", "live", "v"];

function videoId(url: URL): string | null {
    const host = url.hostname.toLowerCase();
    const segments = url.pathname.split("/").filter(segment => segment.length > 0);

    if (SHORT_HOSTS.includes(host)) {
        return segments[0] ?? null;
    }

    if (!HOSTS.includes(host)) {
        return null;
    }

    if (segments[0] === "watch") {
        return url.searchParams.get("v");
    }

    if (PATH_PREFIXES.includes(segments[0])) {
        return segments[1] ?? null;
    }

    return null;
}

function startSeconds(url: URL): number {
    const value = url.searchParams.get("t") ?? url.searchParams.get("start");
    if (!value) {
        return 0;
    }

    const match = TIMESTAMP.exec(value);
    if (!match) {
        return 0;
    }

    const [, hours, minutes, seconds] = match;
    return Number(hours ?? 0) * 3600 + Number(minutes ?? 0) * 60 + Number(seconds ?? 0);
}

export function embedUrl(href: string): string | null {
    let url: URL;
    try {
        url = new URL(href);
    } catch {
        return null;
    }

    const id = videoId(url);
    if (!id || !VIDEO_ID.test(id)) {
        return null;
    }

    const start = startSeconds(url);
    return start > 0 ? `${EMBED_ORIGIN}${id}?start=${start}` : `${EMBED_ORIGIN}${id}`;
}

import { api_url } from "./fetcher";
import { Language } from "./state";

const SAVE_DELAY_MS = 2500;

type Pending = { problem: number, language: Language, code: string | null };

// Signed-out visitors keep their drafts in the browser only; stop asking after the first refusal.
let signedOut = false;

function send({ problem, language, code }: Pending) {
    if (signedOut) return;
    // keepalive lets the last save finish while the page is closing.
    fetch(api_url(`/problems/${problem}/draft`), {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        keepalive: true,
        body: JSON.stringify({ language, code }),
    }).then((response) => { if (response.status === 401) signedOut = true; }).catch(() => undefined);
}

/// Saves drafts to the account shortly after typing stops, one request per pause.
/// The browser's own copy stays the immediate source; this makes it follow the user.
export function createDraftSaver() {
    let pending: Pending | null = null;
    let timer: ReturnType<typeof setTimeout> | null = null;

    const flush = () => {
        if (timer) clearTimeout(timer);
        timer = null;
        if (pending) send(pending);
        pending = null;
    };

    return {
        schedule(problem: number, language: Language, code: string) {
            // A draft for another problem or language must not be dropped.
            if (pending && (pending.problem !== problem || pending.language !== language)) flush();
            pending = { problem, language, code };
            if (timer) clearTimeout(timer);
            timer = setTimeout(flush, SAVE_DELAY_MS);
        },
        discard(problem: number, language: Language) {
            if (timer) clearTimeout(timer);
            timer = null;
            pending = null;
            send({ problem, language, code: null });
        },
        flush,
    };
}

export async function loadDraft(problem: number, language: Language): Promise<string | null> {
    const response = await fetch(api_url(`/problems/${problem}/draft?language=${language}`), { credentials: "include" });
    if (!response.ok) return null;
    const draft = await response.json();
    return typeof draft?.code === "string" ? draft.code : null;
}

import dynamic from "next/dynamic";
import Head from "next/head";
import Link from "next/link";
import { createContext, useEffect, useRef } from "react";
import useSWR from "swr";
import { Competition } from "../../pages/competitions";
import { api_url, fetcher } from "../../utils/fetcher";
import { createDraftSaver, loadDraft } from "../../utils/drafts";
import { getProblemImpl, Language, useStore } from "../../utils/state";
import Tabbed from "../tabbed";
import CodeRunner from "./code-runner";
import Description from "./description";
import ProblemLeaderboard from "./leaderboard";
import SubmissionHistory from "./submission/history";
import Timer from "./timer";
import TestContainer from "./submission/tests/container";
const Editor = dynamic(import("../../components/editor"), { ssr: false });

export const ProblemContext = createContext<Problem | undefined>(undefined);
export const ProblemIDContext = createContext<number | undefined>(undefined);

type ProblemViewProps = {
    id?: number;
    competitionId?: number;
    hideEditorScrollbars?: boolean;
};

type Problem = {
    id: number;
    title: string;
    description: string;
    template: string;
    /// "stdio" when the answer is a complete C++ program instead of a function.
    runner?: string;
    competition_id?: number;
};

async function rustTemplate(url: string, name = "Rust"): Promise<string> {
    const response = await fetch(url, { credentials: "include" });
    if (!response.ok) {
        const body = await response.json().catch(() => null);
        throw new Error(
            body?.message ??
                body?.error ??
                `Could not load the ${name} template.`,
        );
    }
    return response.json();
}

// Rust and Python starter code is generated from the problem's tests.
const generatedTemplates = { rust: "Rust", python: "Python" } as const;

function ProblemEditorWrapper({
    id,
    template,
    program,
    hideEditorScrollbars,
}: {
    id?: number;
    template?: string;
    /// Whole-program problems read standard input, which only C++ supports.
    program: boolean;
    hideEditorScrollbars?: boolean;
}) {
    const chosen = useStore((state) =>
        id ? (state.problemLanguages[id] ?? "cpp") : "cpp",
    );
    const language: Language = program ? "cpp" : chosen;
    const saved = useStore((state) =>
        id ? getProblemImpl(state, id, language) : undefined,
    );
    const setProblemImpl = useStore((state) => state.setProblemImpl);
    const setLanguage = useStore((state) => state.setProblemLanguage);
    const resetProblemImpl = useStore((state) => state.resetProblemImpl);
    const saver = useRef(createDraftSaver()).current;
    const edited = useRef(false);

    // Bring in the draft saved to the account, unless typing has already started here.
    useEffect(() => {
        edited.current = false;
        if (!id) return;
        let current = true;
        loadDraft(id, language)
            .then((code) => {
                if (!current || edited.current || code === null) return;
                if (code !== getProblemImpl(useStore.getState(), id, language))
                    setProblemImpl(id, code, language);
            })
            .catch(() => undefined);
        return () => {
            current = false;
            saver.flush();
        };
    }, [id, language, saver, setProblemImpl]);

    // Submissions and custom runs read the stored language, so keep it on C++.
    useEffect(() => {
        if (id && program && chosen !== "cpp") setLanguage(id, "cpp");
    }, [id, program, chosen, setLanguage]);

    useEffect(() => {
        window.addEventListener("pagehide", saver.flush);
        return () => window.removeEventListener("pagehide", saver.flush);
    }, [saver]);

    const resetCode = () => {
        if (!id || !window.confirm("Discard your code for this language and start again from the template?")) return;
        edited.current = true;
        resetProblemImpl(id, language);
        saver.discard(id, language);
    };
    const generated = language === "cpp" ? undefined : language;
    const { data, error } = useSWR<string>(
        id && generated
            ? api_url(`/problems/${id}/${generated}-template`)
            : null,
        (url: string) =>
            rustTemplate(url, generated && generatedTemplates[generated]),
        { shouldRetryOnError: false },
    );
    const content = saved ?? (generated ? data : template) ?? "";
    return (
        <div className="bg-white dark:bg-neutral-900 h-full min-h-0 flex flex-col ring-1 md:ring-0 ring-neutral-300 dark:ring-neutral-700">
            <div className="p-2 border-b border-neutral-300 dark:border-neutral-700 flex flex-wrap items-center gap-x-4 gap-y-1 text-sm sm:text-base">
            <label>
                Language{" "}
                <select
                    aria-label="Submission language"
                    value={language}
                    onChange={(event) =>
                        id && setLanguage(id, event.target.value as Language)
                    }
                    className="bg-white dark:bg-neutral-900"
                >
                    <option value="cpp">C++</option>
                    {!program && <option value="rust">Rust</option>}
                    {!program && <option value="python">Python</option>}
                </select>
            </label>
                <button
                    className="px-2 py-1 rounded hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
                    onClick={resetCode}
                >
                    Reset code
                </button>
                <span className="ml-auto">{id && <Timer id={id} />}</span>
            </div>
            {program && (
                <p className="px-2 text-sm">
                    Write a complete C++ program with <code>main</code>. Read
                    the input from standard input (<code>cin</code>) and print
                    the answer to standard output (<code>cout</code>).
                </p>
            )}
            {language === "rust" && (
                <p className="px-2 text-sm">
                    Rust standard library only. Cargo crates are not available.
                </p>
            )}
            {language === "python" && (
                <p className="px-2 text-sm">
                    Python 3.14 standard library only. Packages such as NumPy
                    are not available.
                </p>
            )}
            {error && (
                <p role="alert" className="p-2 text-red-600">
                    {error.message}
                </p>
            )}
            <div className="flex-1 min-h-0">
                <Editor
                    key={`${id}:${language}`}
                    diagnosticProblem={id}
                    language={language}
                    value={content}
                    hideScrollbars={hideEditorScrollbars}
                    onChange={(text) => {
                        if (!id) return;
                        edited.current = true;
                        setProblemImpl(id, text, language);
                        saver.schedule(id, language, text);
                    }}
                />
            </div>
        </div>
    );
}

export default function ProblemView({
    id,
    hideEditorScrollbars,
}: ProblemViewProps): JSX.Element {
    const { data, error } = useSWR<Problem>(
        id ? api_url(`/problems/${id}`) : null,
        fetcher,
    );

    function BackToCompetition({
        competitionId,
    }: {
        competitionId?: number;
    }): JSX.Element {
        const { data, error } = useSWR<Competition>(
            competitionId ? api_url(`/competitions/${competitionId}`) : null,
            fetcher,
        );

        if (!competitionId || error || !data) return <></>;

        return (
            <div className="border-neutral-300 dark:border-neutral-700 border-b p-2">
                <h1 className="font-bold text-lg">{data.name}</h1>

                <Link href={`/competitions/${competitionId}`}>
                    <a className="text-blue-700 hover:underline dark:text-blue-500">
                        ‹ Back to overview.
                    </a>
                </Link>
            </div>
        );
    }

    if (error) {
        return (
            <div className="h-full flex flex-col items-center justify-center gap-2 bg-white dark:bg-black p-8 text-center">
                <h1 className="text-2xl font-bold">Could not load problem</h1>
                <p className="max-w-md text-neutral-600 dark:text-neutral-400">
                    Check that the API is running and that this problem is still
                    available.
                </p>
            </div>
        );
    }

    return (
        <ProblemIDContext.Provider value={id}>
            <ProblemContext.Provider value={data}>
                <Head>
                    <title>{data?.title}</title>
                </Head>

                <div className="grid grid-rows-[min-content_min-content_40vh_minmax(0,1fr)] md:grid-cols-[400px_minmax(0,1fr)] lg:grid-cols-[500px_minmax(0,1fr)] md:grid-rows-full-min h-full">
                    <div className="md:border-r border-neutral-300 dark:border-neutral-700 row-span-2 flex flex-col border-b md:border-b-0">
                        {data?.competition_id && (
                            <BackToCompetition
                                competitionId={data?.competition_id}
                            />
                        )}

                        <TestContainer />

                        <Tabbed
                            titles={["Description", "History", "Leaderboard"]}
                            className="h-full overflow-y-auto"
                        >
                            <Description />
                            <SubmissionHistory />
                            <ProblemLeaderboard />
                        </Tabbed>
                    </div>

                    <ProblemEditorWrapper
                        id={id}
                        template={data?.template}
                        program={data?.runner === "stdio"}
                        hideEditorScrollbars={hideEditorScrollbars}
                    />
                    <CodeRunner />
                </div>
            </ProblemContext.Provider>
        </ProblemIDContext.Provider>
    );
}

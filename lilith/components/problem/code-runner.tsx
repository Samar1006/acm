import { useContext, useEffect, useState } from "react";
import useSWR, { useSWRConfig } from "swr";
import { ProblemIDContext } from ".";
import { api_url, fetcher } from "../../utils/fetcher";
import { JobStatus, monitorJob } from "../../utils/job";
import { getProblemImpl, Submission, useSession, useStore } from "../../utils/state";
import EditorPreferences from "../editor-preferences";
import LoadingButton from "../loading-button";
import Modal from "../modal";
import QueueStatus from "../queue-status";
import { ProblemMeta } from "./examples";
import InputTester from "./input-tester";
import { DiagnosticsDisplay } from "./submission";
import { TestResult } from "./submission/tests";
import { TestResultInner } from "./submission/tests/test_result_view";

type ExampleResults = { tests: TestResult[], passed: boolean };

/// Whether the code now in the editor is exactly the code that passed the examples.
function useExamplesPassed(id: number): boolean {
    const implementation = useStore((state) => id ? getProblemImpl(state, id) : undefined);
    const language = useStore(state => state.problemLanguages[id] ?? "cpp");
    const passed = useSession((session) => session.examplesPassed);
    return !!passed && passed.problem === id && passed.language === language && passed.source === implementation;
}

export default function CodeRunner(): JSX.Element {
    const [dockerShown, setDockerShown] = useState(false);
    const [settingsShown, setSettingsShown] = useState(false);
    const setSubmissionShown = useSession((session) => session.setSubmissionShown);
    const [exampleResults, setExampleResults] = useState<ExampleResults | null>(null);
    const [exampleError, setExampleError] = useState<string | null>(null);
    const problemId = useContext(ProblemIDContext);

    // Results belong to one problem; never show another problem's.
    useEffect(() => {
        setExampleResults(null);
        setExampleError(null);
    }, [problemId]);

    function SubmitButton(): JSX.Element {
        const id = useContext(ProblemIDContext)!;
        const implementation = useStore(
            (state) => id ? getProblemImpl(state, id) : undefined
        );
        const language = useStore(state => state.problemLanguages[id] ?? "cpp");
        const setError = useSession((session) => session.setError);
        const [loading, setLoading] = useState(false);
        const [queuePosition, setQueuePosition] = useState(0);
        const { mutate } = useSWRConfig();
        const { data: meta } = useSWR<ProblemMeta>(id ? api_url(`/problems/${id}/meta`) : null, fetcher);
        const tested = useExamplesPassed(id);
        // The server enforces the same rule; this only explains it up front.
        const locked = !!meta?.require_tests && !tested;

        const submitProblem = async () => {
            if (!implementation) {
                setError("You must modify the answer before submitting.", true);
                return;
            }

            const request = useSession.getState().beginCompilerCheck(id, language, implementation);
            let compilerError = useSession.getState().compilerError;
            setLoading(true);

            try {
                let res = await fetch(api_url("/run/submit"), {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    credentials: "include",
                    body: JSON.stringify({
                        problem_id: id,
                        implementation,
                        language,
                    }),
                });

                if (!res.ok) {
                    const error = await res.json().catch(() => null);
                    setError(error?.message ?? error?.error ?? "Submission was rejected.", true);
                    return;
                }
                let job: JobStatus<Submission, string> = await res.json();
                console.log(job);
                let [data, err] = await monitorJob(job, (n) => setQueuePosition(n));

                compilerError = err ?? data?.error ?? null;

                if (data) {
                    setTimeout(() => {
                        setSubmissionShown(true);
                        mutate(api_url(`/problems/${id}/recent-submission`));
                    }, 0);
                }

                if (err) {
                    console.log(`${err}`)
                    setError(err, true);
                }
            }
            catch (e) {
                setError("Network error.", true);
            }
            finally {
                useSession.getState().finishCompilerCheck(request, compilerError);
                setLoading(false);
            }
        };

        return (
            <div className="flex items-center">
                {loading &&
                    <QueueStatus className="mr-4" queuePosition={queuePosition} />
                }
                <LoadingButton
                    onClick={() => {
                        if (locked) setError("Run Tests and pass every example with this code before submitting.", true);
                        else submitProblem();
                    }}
                    loading={loading}
                    disabled={locked}
                    title={locked ? "Pass the example tests with this code to enable Submit" : undefined}
                    className={`p-4 border-l h-full border-neutral-300 dark:border-neutral-700 transition-colors text-white ${locked
                        ? "bg-neutral-400 dark:bg-neutral-700 cursor-not-allowed"
                        : "bg-green-500 dark:bg-green-600 hover:bg-green-400 dark:hover:bg-green-500"}`}
                >
                    Submit
                </LoadingButton>
            </div>
        );
    }

    function TestsButton(): JSX.Element | null {
        const id = useContext(ProblemIDContext)!;
        const implementation = useStore((state) => id ? getProblemImpl(state, id) : undefined);
        const language = useStore(state => state.problemLanguages[id] ?? "cpp");
        const setError = useSession((session) => session.setError);
        const { data: meta } = useSWR<ProblemMeta>(id ? api_url(`/problems/${id}/meta`) : null, fetcher);
        const [loading, setLoading] = useState(false);
        const [queuePosition, setQueuePosition] = useState(0);

        if (!meta || meta.example_count === 0) return null;

        const runExamples = async () => {
            if (loading) return;
            if (!implementation) {
                setError("You must modify the answer before testing.", true);
                return;
            }

            const request = useSession.getState().beginCompilerCheck(id, language, implementation);
            let compilerError = useSession.getState().compilerError;
            setLoading(true);
            try {
                const response = await fetch(api_url("/run/tests"), {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    credentials: "include",
                    body: JSON.stringify({ problem_id: id, implementation, language }),
                });
                if (!response.ok) {
                    const error = await response.json().catch(() => null);
                    setError(error?.message ?? error?.error ?? "Tests were rejected.", true);
                    return;
                }
                const job: JobStatus<ExampleResults, string> = await response.json();
                const [data, err] = await monitorJob(job, (n) => setQueuePosition(n));
                compilerError = err ?? null;
                setExampleResults(data ?? null);
                setExampleError(err ?? null);
                // Any later edit or language change makes this stop matching.
                useSession.getState().setExamplesPassed(
                    data?.passed ? { problem: id, language, source: implementation } : null);
            }
            catch (e) {
                setError("Network error.", true);
            }
            finally {
                useSession.getState().finishCompilerCheck(request, compilerError);
                setLoading(false);
            }
        };

        return (
            <div className="flex items-center">
                {loading && <QueueStatus className="mr-4" queuePosition={queuePosition} />}
                <LoadingButton
                    onClick={runExamples}
                    loading={loading}
                    className="p-4 border-l h-full border-neutral-300 dark:border-neutral-700 bg-blue-600 hover:bg-blue-500 transition-colors text-white"
                >
                    Tests
                </LoadingButton>
            </div>
        );
    }

    function ExampleResultsPanel(): JSX.Element | null {
        if (!exampleResults && !exampleError) return null;

        const passed = exampleResults?.tests.filter((test) => test.success).length ?? 0;

        return (
            <div className="border-t border-neutral-300 dark:border-neutral-700 bg-white dark:bg-black max-h-[40vh] overflow-y-auto" role="region" aria-label="Example test results">
                <div className="flex items-center p-4 pb-0">
                    <b>
                        {exampleError
                            ? "Tests could not run"
                            : `Tests: ${passed} of ${exampleResults!.tests.length} examples passed`}
                    </b>
                    <button
                        className="ml-auto px-3 py-1 rounded-full bg-neutral-200 dark:bg-neutral-800 hover:bg-neutral-300 dark:hover:bg-neutral-700 transition-colors text-sm"
                        onClick={() => { setExampleResults(null); setExampleError(null); }}
                    >
                        Close
                    </button>
                </div>
                {exampleError &&
                    <div className="m-4 bg-red-500 dark:bg-red-700 text-red-50">
                        <DiagnosticsDisplay error={exampleError} />
                    </div>}
                {exampleResults &&
                    <div className="grid gap-4 p-4 lg:grid-cols-2">
                        {exampleResults.tests.map((test) =>
                            <TestResultInner key={test.index} {...test} index={test.index + 1} />)}
                    </div>}
            </div>
        );
    }

    return (
        <div className="sticky md:static bottom-0">
            <Modal shown={settingsShown} onClose={() => setSettingsShown(false)}>
                <EditorPreferences />
            </Modal>
            {dockerShown && <InputTester />}
            <ExampleResultsPanel />

            <div className="flex bg-white dark:bg-black border-t border-neutral-300 dark:border-neutral-700 overflow-x-auto whitespace-nowrap">
                <button
                    className="p-4 border-r border-neutral-300 dark:border-neutral-700 hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
                    onClick={() => setDockerShown(!dockerShown)}
                >
                    {dockerShown ? "Hide console" : "Show console"}
                </button>

                <button
                    className="mr-auto p-4 border-r border-neutral-300 dark:border-neutral-700 hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
                    onClick={() => setSettingsShown(!settingsShown)}
                >
                    Settings
                </button>

                <TestsButton />
                <SubmitButton />
            </div>
        </div>
    );
}

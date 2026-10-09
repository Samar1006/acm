import { useContext, useState } from "react";
import useSWR from "swr";
import { ProblemIDContext } from ".";
import { api_url, fetcher } from "../../utils/fetcher";
import { JobStatus, monitorJob } from "../../utils/job";
import { CustomCase, getProblemImpl, useSession, useStore } from "../../utils/state";
import LoadingButton from "../loading-button";
import QueueStatus from "../queue-status";
import { TestEditor } from "../test-editor";
import { formatValue } from "./examples";
import { DiagnosticsDisplay } from "./submission";
import { isRunnerError } from "./submission/error";
import { Test, TestResult } from "./submission/tests";
import { TestResultInner } from "./submission/tests/test_result_view";

type CustomInputResponse = {
    result: TestResult;
    output: string;
};

type CaseOutcome = { result?: CustomInputResponse, error?: string };

const normalize = (text: string) => text.replace(/\s+/g, "");

/// Runs the current code on the user's own inputs. Each case is compared with the
/// reference solution, and with the user's expected output when one is given.
/// These runs are never stored as submissions.
export default function InputTester() {
    const [running, setRunning] = useState<number | null>(null);
    const [queuePosition, setQueuePosition] = useState(0);
    const [outcomes, setOutcomes] = useState<{ [key: number]: CaseOutcome }>({});

    const setError = useSession((state) => state.setError);
    const problem_id = useContext(ProblemIDContext);
    const implementation = useStore((state) =>
        problem_id ? getProblemImpl(state, problem_id) : undefined
    );
    const language = useStore(state => problem_id ? state.problemLanguages[problem_id] ?? "cpp" : "cpp");
    const saved = useStore(state => problem_id ? state.customCases[problem_id] : undefined);
    const setCustomCases = useStore(state => state.setCustomCases);

    let { data, error } = useSWR<Test>(problem_id ? api_url(`/problems/${problem_id}/tests/0`) : null, fetcher);

    if (!data || error || !problem_id) return null;

    const format = data.input;
    // Saved cases from an older version of the problem may no longer fit its arguments.
    const usable = saved?.filter((item) => item.args.length === format.arguments.length);
    const cases: CustomCase[] = usable && usable.length > 0
        ? usable
        : [{ key: 1, args: format.arguments, expected: "" }];

    const update = (next: CustomCase[]) => setCustomCases(problem_id, next);
    const change = (key: number, patch: Partial<CustomCase>) =>
        update(cases.map((item) => item.key === key ? { ...item, ...patch } : item));
    const setOutcome = (key: number, outcome: CaseOutcome) =>
        setOutcomes((previous) => ({ ...previous, [key]: outcome }));

    // Resolves to the compiler error, null when the code compiled, or undefined when the run never got that far.
    const runCase = async (item: CustomCase): Promise<string | null | undefined> => {
        const input = { ...format, arguments: item.args };
        let res = await fetch(api_url("/run/custom"), {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            credentials: "include",
            body: JSON.stringify({
                problem_id,
                input,
                implementation,
                language,
            }),
        });

        if (!res.ok) {
            const error = await res.json().catch(() => null);
            setOutcome(item.key, { error: error?.message ?? error?.error ?? "Custom input was rejected." });
            return undefined;
        }
        let job: JobStatus<CustomInputResponse, string> = await res.json();

        let [data, err] = await monitorJob(job, (n) => setQueuePosition(n));

        setOutcome(item.key, err ? { error: err } : { result: data });
        return err ?? null;
    };

    const run = async (selected: CustomCase[]) => {
        if (running !== null) return;
        if (!implementation) {
            setError("You must modify the answer before submitting.", true);
            return;
        }

        const request = useSession.getState().beginCompilerCheck(problem_id, language, implementation);
        let compilerError = useSession.getState().compilerError;
        try {
            for (const item of selected) {
                setRunning(item.key);
                const outcome = await runCase(item);
                if (outcome !== undefined) compilerError = outcome;
                // The same code fails to compile for every case.
                if (outcome) break;
            }
        }
        catch (e) {
            console.log(e);
            setError("Network error.", true);
        }
        finally {
            useSession.getState().finishCompilerCheck(request, compilerError);
            setRunning(null);
        }
    };

    const button = "px-4 py-2 rounded-full transition-colors text-sm";
    const secondary = `${button} bg-neutral-200 dark:bg-neutral-800 hover:bg-neutral-300 dark:hover:bg-neutral-700`;

    return (
        <div className="border-t max-h-[40vh] lg:max-h-96 border-neutral-300 dark:border-neutral-700 bg-white dark:bg-black overflow-y-auto">
            <div className="flex flex-wrap items-center gap-2 p-4 pb-0">
                <b className="mr-auto">Custom tests</b>
                {running !== null && <QueueStatus queuePosition={queuePosition} />}
                <button
                    className={secondary}
                    onClick={() => update([...cases, {
                        key: Math.max(...cases.map((item) => item.key)) + 1,
                        args: cases[cases.length - 1].args,
                        expected: "",
                    }])}
                >
                    Add case
                </button>
                {cases.length > 1 &&
                    <LoadingButton
                        className={`${button} bg-blue-700 hover:bg-blue-500 text-blue-100`}
                        loading={running !== null}
                        onClick={() => run(cases)}
                    >
                        Run all
                    </LoadingButton>}
            </div>

            {cases.map((item, i) => {
                const outcome = outcomes[item.key];
                const output = outcome?.result?.result?.output;
                const wanted = normalize(item.expected);
                return (
                    <div key={item.key} className="flex flex-col lg:flex-row gap-4 p-4 border-b last:border-b-0 border-neutral-300 dark:border-neutral-700">
                        <div className="flex flex-col gap-2 lg:w-96 shrink-0">
                            <b>Case {i + 1} input</b>

                            <TestEditor key={item.key} baseArgs={item.args} onChange={(args) => change(item.key, { args })} />

                            <label className="flex flex-col gap-1 text-sm">
                                Expected output (optional)
                                <input
                                    className="border-neutral-300 dark:border-neutral-700 border rounded p-2 bg-neutral-50 dark:bg-neutral-900 font-mono"
                                    value={item.expected}
                                    placeholder={`e.g. ${formatValue(data!.expected_output)}`}
                                    onChange={(event) => change(item.key, { expected: event.target.value })}
                                />
                            </label>

                            <div className="flex items-center gap-2">
                                <LoadingButton
                                    className={`${button} bg-blue-700 hover:bg-blue-500 text-blue-100`}
                                    loading={running === item.key}
                                    onClick={() => run([item])}
                                >
                                    Run
                                </LoadingButton>
                                {cases.length > 1 &&
                                    <button className={secondary} onClick={() => update(cases.filter((other) => other.key !== item.key))}>
                                        Remove
                                    </button>}
                            </div>
                        </div>

                        <div className="flex flex-col gap-2 min-w-0 lg:w-96">
                            {outcome?.error &&
                                <div className="bg-red-500 dark:bg-red-700 text-red-50 border-x border-red-700">
                                    <h1 className="text-2xl font-bold p-4">Error</h1>
                                    <DiagnosticsDisplay error={outcome.error} />
                                </div>}

                            {outcome?.result && <>
                                {output && wanted !== "" &&
                                    <p className={normalize(formatValue(output)) === wanted ? "text-green-600" : "text-red-600"}>
                                        {normalize(formatValue(output)) === wanted
                                            ? "Matches your expected output."
                                            : `Differs from your expected output: got ${formatValue(output)}.`}
                                    </p>}
                                <div className="m-1">
                                    <TestResultInner {...outcome.result.result} index={i + 1} />
                                </div>
                            </>}
                        </div>

                        {outcome?.result && !isRunnerError(outcome.result) && outcome.result.output && outcome.result.output.length > 0 &&
                            <div className="flex flex-col gap-2 min-w-0 lg:w-96">
                                <span className="font-bold">Debug Output</span>
                                <pre
                                    className="rounded-md bg-blue-50 dark:bg-stone-900 p-2 overflow-auto border border-blue-200 dark:border-slate-700"
                                >
                                    {outcome.result.output}
                                </pre>
                            </div>}
                    </div>
                );
            })}
        </div>
    );
}

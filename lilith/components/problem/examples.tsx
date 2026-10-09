import useSWR from "swr";
import { api_url, fetcher } from "../../utils/fetcher";
import { FunctionValue, WasmFunctionCall } from "./submission/tests";

export type Example = {
    index: number;
    input: WasmFunctionCall;
    expected_output: FunctionValue;
    explanation: string;
};

export type ProblemExamples = {
    parameter_names: string[];
    examples: Example[];
};

export type ProblemMeta = {
    tags: string[];
    require_tests: boolean;
    require_tests_setting: "default" | "on" | "off";
    example_count: number;
    bookmarked: boolean;
    solved: boolean;
};

/// Writes a test value the way it would appear in code: 5, "abc", [1, 2], [[1], [2]].
export function formatValue(value: FunctionValue): string {
    const [[type, variant]] = Object.entries(value) as [string, any][];
    const text = type === "String" || type === "Char";
    const one = (item: unknown) => text ? JSON.stringify(item) : String(item);
    const list = (items: unknown[]) => `[${items.map(one).join(", ")}]`;
    if ("Single" in variant) return one(variant.Single);
    if ("List" in variant) return list(variant.List);
    const rows: unknown[][] = variant.Grid ?? variant.Graph ?? [];
    return `[${rows.map(list).join(", ")}]`;
}

export function formatInput(input: WasmFunctionCall, names: string[]): string {
    return input.arguments
        .map((argument, i) => `${names[i] ?? `arg${i}`} = ${formatValue(argument)}`)
        .join(", ");
}

/// The worked examples under a problem description. The Tests button runs these same cases.
export default function Examples({ id }: { id: number }): JSX.Element | null {
    const { data } = useSWR<ProblemExamples>(api_url(`/problems/${id}/examples`), fetcher);

    if (!data || data.examples.length === 0) return null;

    return (
        <section className="mt-6 flex flex-col gap-4" aria-label="Examples">
            {data.examples.map((example, i) => (
                <div key={example.index}>
                    <h3 className="font-bold mb-1">Example {i + 1}</h3>
                    <div className="rounded-md bg-neutral-100 dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-700 p-3 flex flex-col gap-1 overflow-x-auto">
                        <div><b>Input:</b> <code className="break-words">{formatInput(example.input, data.parameter_names)}</code></div>
                        <div><b>Output:</b> <code className="break-words">{formatValue(example.expected_output)}</code></div>
                        {example.explanation && <div><b>Explanation:</b> {example.explanation}</div>}
                    </div>
                </div>
            ))}
        </section>
    );
}

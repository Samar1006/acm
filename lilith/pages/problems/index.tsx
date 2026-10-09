import type { NextPage } from "next";
import Navbar from "../../components/navbar";
import useSWRInfinite from "swr/infinite";
import { marked } from "marked";
import Link from "next/link";
import { api_url, fetcher } from "../../utils/fetcher";
import { User, useStore } from "../../utils/state";
import { useContext, useEffect, useRef, useState } from "react";
import renderLatex from "../../utils/latex";
import { CompetitionIDContext } from "../competitions/[id]";
import useSWR from "swr";
import { useRouter } from "next/router";
import LoadingButton from "../../components/loading-button";
import Head from "next/head";
import ErrorBox from "../../components/error-box";

function ProblemLoading(): JSX.Element {
    return (
        <div className="animate-fade-in sm:rounded-md border-neutral-300 dark:border-neutral-700 border-y sm:border sm:mx-2 md:m-0 bg-white dark:bg-black dark:hover:bg-neutral-800 p-4 hover:shadow-md max-h-52 hover:max-h-64 overflow-hidden transition-all">
            <div className="animate-pulse">
                <h1 className="rounded bg-neutral-200 w-64 text-2xl h-6 mb-4"></h1>
                <p className="rounded bg-neutral-200 w-full h-3 mb-2"></p>
                <p className="rounded bg-neutral-200 w-full h-3 mb-2"></p>
                <p className="rounded bg-neutral-200 w-full h-3"></p>
            </div>
        </div>
    );
}

export type Problem = {
    id: number;
    title: string;

    // markdown formatted
    description: string;
    runtime_multiplier: number;

    template: string;

    visible: boolean;
    difficulty?: string;

    tags?: string[];
    solved?: boolean;
    bookmarked?: boolean;
};

type ProblemStatus = "Complete" | "InProgress" | "NotStarted";

function ProblemTeamStatus({ problem_id }: { problem_id: number }): JSX.Element {

    const competition_id = useContext(CompetitionIDContext);

    const { data, error } = useSWR<ProblemStatus>(
        api_url(`/competitions/${competition_id}/problem-status/${problem_id}`),
        fetcher
    );

    if (!data || error) return <></>;

    if (data == "InProgress") {
        return (
            <div className="ml-auto bg-blue-700 text-blue-50 rounded-full px-4 py-2 text-sm">
                In Progress
            </div>
        );
    } else if (data == "Complete") {
        return (
            <div className="ml-auto bg-green-700 text-blue-50 rounded-full px-4 py-2 text-sm">
                Completed
            </div>
        );
    } else {
        return <></>;
    }
}

interface ProblemListingProps extends Problem {
    show_team_status?: boolean;
};

function DifficultyBadge({ difficulty }: { difficulty: string }): JSX.Element {
    if (difficulty == "Easy") {
        return <span className="bg-green-600 text-green-50 rounded-full px-4 py-2 text-sm">Easy</span>;
    }

    if (difficulty == "Medium") {
        return <span className="bg-yellow-600 text-yellow-50 rounded-full px-4 py-2 text-sm">Medium</span>;
    }

    if (difficulty == "Hard") {
        return <span className="bg-red-600 text-red-50 rounded-full px-4 py-2 text-sm">Hard</span>;
    }

    return <>{difficulty}</>;
}

function ProblemListing({ id, title, description, show_team_status, difficulty, tags, solved, bookmarked }: ProblemListingProps): JSX.Element {
    let desc = marked.parse(description);
    const content = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (content.current) {
            renderLatex(content.current);
        }
    });

    return (
        <Link href={`/problems/${id}`}>
            <a className="sm:rounded-md border-neutral-300 dark:border-neutral-700 border-y sm:border sm:mx-2 md:m-0 bg-white dark:bg-black dark:hover:bg-neutral-800 p-4 hover:shadow-md max-h-52 hover:max-h-64 overflow-hidden transition-all">
                <div className="flex items-center mb-4">
                    <h1 className="text-2xl font-extrabold">
                        {bookmarked && <span title="Bookmarked" aria-label="Bookmarked">★ </span>}
                        {title}
                    </h1>

                    <div className="flex ml-auto gap-4 items-center">
                        {solved && <span className="text-green-600 dark:text-green-400 text-sm font-bold">Solved</span>}
                        {difficulty !== undefined && <DifficultyBadge difficulty={difficulty} />}
                        {show_team_status && <ProblemTeamStatus problem_id={id} />}
                    </div>
                </div>

                {tags && tags.length > 0 && <div className="flex flex-wrap gap-2 mb-4 text-sm">
                    {tags.map((tag) => <span key={tag} className="rounded-full bg-neutral-100 dark:bg-neutral-800 px-3 py-1">{tag}</span>)}
                </div>}

                <div
                    ref={content}
                    className="prose prose-neutral dark:prose-invert"
                    dangerouslySetInnerHTML={{ __html: desc }}
                />
            </a>
        </Link>
    );
}

function ListLoading(): JSX.Element {
    return (
        <>
            <ProblemLoading />
            <ProblemLoading />
            <ProblemLoading />
            <ProblemLoading />
        </>
    );
}

export function ProblemList({ problems, show_team_status, show_difficulty }: { problems: Problem[], show_team_status?: boolean, show_difficulty?: boolean }): JSX.Element {
    return (
        <>
            {problems.map(({ id, title, description, difficulty, visible, tags, solved, bookmarked }) => (
                <ProblemListing
                    key={id}
                    id={id}
                    title={title}
                    template={""}
                    runtime_multiplier={0.0}
                    description={description}
                    show_team_status={show_team_status}
                    difficulty={(show_difficulty ?? true) ? difficulty : undefined}
                    visible={visible}
                    tags={tags}
                    solved={solved}
                    bookmarked={bookmarked}
                />
            ))}
        </>
    );
}

type Filters = {
    query: string;
    difficulty: number;
    tags: string[];
    status: "" | "solved" | "unsolved";
    bookmarked: boolean;
    showCompetitionProblems: boolean;
    sortBy: string;
};

export const noFilters: Filters = {
    query: "",
    difficulty: 0,
    tags: [],
    status: "",
    bookmarked: false,
    showCompetitionProblems: false,
    sortBy: "Newest",
};

export function filterParams({ query, difficulty, tags, status, bookmarked, showCompetitionProblems, sortBy }: Filters): string {
    const params = new URLSearchParams({ difficulty: difficulty.toString(), sort_by: sortBy });
    if (query.trim() !== "") params.set("query", query.trim());
    if (tags.length > 0) params.set("tags", tags.join(","));
    if (status !== "") params.set("status", status);
    if (bookmarked) params.set("bookmarked", "true");
    if (showCompetitionProblems) params.set("show_competition_problems", "true");
    return params.toString();
}

export function ProblemResults({ filters }: { filters: Filters }) {
    const params = filterParams(filters);
    const { data, error, isValidating, size, setSize } = useSWRInfinite<Problem[]>(
        (pageIndex, previousProblems) => {
            if (previousProblems && !previousProblems.length) return null;

            return api_url(`/problems?offset=${7 * pageIndex}&count=7&${params}`);
        },
        fetcher
    );

    if (error) return (
        <ErrorBox>
            Could not fetch problems
        </ErrorBox>
    );

    if (data?.[0]?.length === 0) return <p>No problems found.</p>;

    return <>
        {!data ? <ListLoading /> : data.map((problems, i) => <ProblemList key={i} problems={problems} />)}

        {data?.[data.length - 1]?.length !== 0 && <LoadingButton
            loading={isValidating}
            className="rounded-full bg-neutral-200 hover:bg-neutral-300 px-6 py-3 transition-colors mx-auto dark:hover:bg-neutral-700 dark:bg-neutral-800"
            onClick={() => setSize(size + 1)}
        >Load more</LoadingButton>}
    </>;
}

// Returns plain elements, so labels stay directly tied to their inputs.
function choice(id: string, type: "checkbox" | "radio", checked: boolean, onChange: () => void, label: string, name?: string) {
    return (
        <div className="flex items-center gap-2">
            <input id={id} type={type} name={name} checked={checked} onChange={onChange} />
            <label htmlFor={id}>{label}</label>
        </div>
    );
}

const ProblemListPage: NextPage = () => {
    const [filters, setFilters] = useState<Filters>(noFilters);
    const [isComponentMounted, setIsComponentMounted] = useState(false);
    const [showFilters, setShowFilters] = useState(false);
    const [randomLoading, setRandomLoading] = useState(false);
    const [randomMessage, setRandomMessage] = useState("");
    const router = useRouter();
    const set = (patch: Partial<Filters>) => setFilters((current) => ({ ...current, ...patch }));
    const { difficulty, tags, status, bookmarked, showCompetitionProblems, sortBy, query } = filters;

    const { data: user } = useSWR<User>(
        api_url("/user/me"),
        fetcher, {
        shouldRetryOnError: false,
    });

    const { data: allTags } = useSWR<{ name: string, count: number }[]>(api_url("/problems/tags"), fetcher);

    useEffect(() => setIsComponentMounted(true), []);

    // A tag link from a problem page opens the list already filtered.
    useEffect(() => {
        if (!router.isReady || typeof router.query.tags !== "string") return;
        setFilters((current) => ({ ...current, tags: (router.query.tags as string).split(",").filter(Boolean) }));
        setShowFilters(true);
    }, [router.isReady, router.query.tags]);

    const loggedIn = isComponentMounted && !!user;
    const activeFilters = filterParams(filters) !== filterParams(noFilters);

    const openRandom = async () => {
        setRandomLoading(true);
        setRandomMessage("");
        try {
            // Keeps the difficulty and tag filters; skips problems already solved.
            const params = filterParams({ ...filters, query: "", status: "unsolved", sortBy: "Random" });
            const found: Problem[] = await fetcher(api_url(`/problems?count=1&${params}`));
            if (found.length > 0) await router.push(`/problems/${found[0].id}`);
            else setRandomMessage(loggedIn ? "No unsolved problems match these filters." : "No problems match these filters.");
        } catch {
            setRandomMessage("Could not pick a problem. Try again.");
        } finally {
            setRandomLoading(false);
        }
    };

    return (
        <>
            <Navbar />

            <Head>
                <title>Problems</title>
            </Head>

            <div className="max-w-screen-md mx-auto my-4 flex flex-col gap-4">
                <div className="grid grid-cols-5 gap-4 items-center justify-center">
                    <button onClick={() => setShowFilters(!showFilters)} className="col-end-2 mr-auto ml-4 md:ml-0 bg-neutral-100 dark:bg-neutral-800 dark:hover:bg-neutral-700 hover:bg-neutral-200 px-4 py-2 rounded-full transition-colors">
                        Filters
                    </button>

                    <div className="col-start-2 col-end-5 focus-within:outline focus-within:outline-2 focus-within:border-neutral-200 focus-within:outline-neutral-200 border-neutral-100 dark:border-neutral-800 border rounded-full bg-white dark:bg-black dark:text-white overflow-hidden h-10 flex">
                        <input aria-label="Search problems" placeholder="Search by title" className="outline-0 w-full ml-4 h-full dark:bg-black" value={query} onChange={e => set({ query: e.target.value })} />

                        <div className="h-10 aspect-square inline-flex items-center justify-center">
                            <svg className="w-4 dark:fill-white dark:stroke-white" enableBackground="new 0 0 32 32" id="Glyph" version="1.1" viewBox="0 0 32 32"><path d="M27.414,24.586l-5.077-5.077C23.386,17.928,24,16.035,24,14c0-5.514-4.486-10-10-10S4,8.486,4,14  s4.486,10,10,10c2.035,0,3.928-0.614,5.509-1.663l5.077,5.077c0.78,0.781,2.048,0.781,2.828,0  C28.195,26.633,28.195,25.367,27.414,24.586z M7,14c0-3.86,3.14-7,7-7s7,3.14,7,7s-3.14,7-7,7S7,17.86,7,14z" id="XMLID_223_" /></svg>
                        </div>
                    </div>

                    {isComponentMounted && user && (user.auth === "OFFICER" || user.auth === "ADMIN") && (
                        <Link href="/problems/new">
                            <a className="col-start-5 ml-auto text-green-50 text-sm font-bold rounded-full bg-green-700 hover:bg-green-500 transition-colors px-4 py-2 mr-4 md:mr-0">
                                New
                            </a>
                        </Link>
                    )}
                </div>

                <div className="flex flex-wrap items-center gap-2 mx-4 md:mx-0">
                    <LoadingButton
                        loading={randomLoading}
                        onClick={openRandom}
                        className="bg-neutral-100 dark:bg-neutral-800 dark:hover:bg-neutral-700 hover:bg-neutral-200 px-4 py-2 rounded-full transition-colors"
                    >
                        {loggedIn ? "Random unsolved problem" : "Random problem"}
                    </LoadingButton>
                    {activeFilters && <button
                        onClick={() => setFilters(noFilters)}
                        className="bg-neutral-100 dark:bg-neutral-800 dark:hover:bg-neutral-700 hover:bg-neutral-200 px-4 py-2 rounded-full transition-colors"
                    >
                        Clear all filters
                    </button>}
                    {randomMessage && <span role="status" className="text-sm">{randomMessage}</span>}
                </div>

                {showFilters && <div className="bg-neutral-100 dark:bg-neutral-800 sm:rounded-xl p-4 flex flex-col gap-4">
                    <div className="flex flex-col gap-4 sm:flex-row sm:gap-16">
                        <div className="flex flex-col">
                            <span className="font-bold mb-2">Difficulty</span>
                            {choice("easy", "checkbox", (difficulty & 1) !== 0, () => set({ difficulty: difficulty ^ 1 }), "Easy")}
                            {choice("medium", "checkbox", (difficulty & 2) !== 0, () => set({ difficulty: difficulty ^ 2 }), "Medium")}
                            {choice("hard", "checkbox", (difficulty & 4) !== 0, () => set({ difficulty: difficulty ^ 4 }), "Hard")}
                        </div>

                        {loggedIn && <div className="flex flex-col">
                            <span className="font-bold mb-2">Status</span>
                            {choice("status-all", "radio", status === "", () => set({ status: "" }), "All", "status")}
                            {choice("status-solved", "radio", status === "solved", () => set({ status: "solved" }), "Solved", "status")}
                            {choice("status-unsolved", "radio", status === "unsolved", () => set({ status: "unsolved" }), "Unsolved", "status")}
                            {choice("bookmarked", "checkbox", bookmarked, () => set({ bookmarked: !bookmarked }), "Bookmarked")}
                        </div>}

                        <div className="flex flex-col">
                            <span className="font-bold mb-2">Sort By</span>
                            {choice("newest", "radio", sortBy == "Newest", () => set({ sortBy: "Newest" }), "Newest", "sort-by")}
                            {choice("oldest", "radio", sortBy == "Oldest", () => set({ sortBy: "Oldest" }), "Oldest", "sort-by")}
                            {choice("by-difficulty", "radio", sortBy == "Difficulty", () => set({ sortBy: "Difficulty" }), "Difficulty", "sort-by")}
                            {choice("by-title", "radio", sortBy == "Title", () => set({ sortBy: "Title" }), "A to Z", "sort-by")}
                        </div>

                        <div className="flex flex-col">
                            <span className="font-bold mb-2">Misc</span>
                            {choice("competition-problems", "checkbox", showCompetitionProblems, () => set({ showCompetitionProblems: !showCompetitionProblems }), "Show competition problems")}
                        </div>
                    </div>

                    {allTags && allTags.length > 0 && <div>
                        <span className="font-bold">Tags</span>
                        <span className="text-sm"> (problems must have every selected tag)</span>

                        <div className="flex flex-wrap gap-2 mt-2">
                            {allTags.map(({ name, count }) => {
                                const selected = tags.includes(name);
                                return (
                                    <button
                                        key={name}
                                        aria-pressed={selected}
                                        onClick={() => set({ tags: selected ? tags.filter((tag) => tag !== name) : [...tags, name] })}
                                        className={`rounded-full px-3 py-1 text-sm transition-colors ${selected
                                            ? "bg-blue-600 text-blue-50 hover:bg-blue-500"
                                            : "bg-white dark:bg-black hover:bg-neutral-200 dark:hover:bg-neutral-700"}`}
                                    >
                                        {name} <span className="opacity-60">{count}</span>
                                    </button>
                                );
                            })}
                        </div>
                    </div>}
                </div>}

                <ProblemResults filters={filters} />
            </div>
        </>
    );
};

export default ProblemListPage;

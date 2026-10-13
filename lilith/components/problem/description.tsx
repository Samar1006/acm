import { marked } from "marked";
import { useContext, useEffect, useRef } from "react";
import { ProblemContext } from ".";
import renderMathInElement from "katex/contrib/auto-render";
import renderLatex from "../../utils/latex";
import Link from "next/link";
import useSWR, { useSWRConfig } from "swr";
import { User } from "../../utils/state";
import { api_url, fetcher } from "../../utils/fetcher";
import Examples, { ProblemMeta } from "./examples";

function TagsAndBookmark({ id, loggedIn }: { id: number, loggedIn: boolean }): JSX.Element | null {
  const key = api_url(`/problems/${id}/meta`);
  const { data } = useSWR<ProblemMeta>(key, fetcher);
  const { mutate } = useSWRConfig();

  if (!data) return null;

  const toggle = async () => {
    const bookmarked = !data.bookmarked;
    // Show the change at once; the next fetch confirms it.
    mutate(key, { ...data, bookmarked }, false);
    await fetch(api_url(`/problems/${id}/bookmark`), {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify({ bookmarked }),
    }).catch(() => undefined);
    mutate(key);
  };

  return (
    <div className="flex flex-wrap items-center gap-2 mb-4 text-sm">
      {data.solved && <span className="rounded-full bg-green-600 text-green-50 px-3 py-1">Solved</span>}
      {data.tags.map((tag) => (
        <Link key={tag} href={`/problems?tags=${encodeURIComponent(tag)}`}>
          <a className="rounded-full bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 px-3 py-1 transition-colors">{tag}</a>
        </Link>
      ))}
      {loggedIn && (
        <button
          onClick={toggle}
          aria-pressed={data.bookmarked}
          className="rounded-full bg-neutral-100 dark:bg-neutral-800 hover:bg-neutral-200 dark:hover:bg-neutral-700 px-3 py-1 transition-colors"
        >
          {data.bookmarked ? "★ Bookmarked" : "☆ Bookmark"}
        </button>
      )}
    </div>
  );
}

export default function Description(): JSX.Element {
  const problem = useContext(ProblemContext);
  const content = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (content.current) {
      renderLatex(content.current);
    }
  });

  const { data: user } = useSWR<User>(
    api_url("/user/me"),
    fetcher, {
    shouldRetryOnError: false,
  });

  if (!problem) {
    return (
      <div className="animate-pulse p-4">
        <h1 className="rounded bg-neutral-200 w-64 h-10 mb-4"></h1>
        <p className="rounded bg-neutral-200 w-full h-3 mb-2"></p>
        <p className="rounded bg-neutral-200 w-full h-3 mb-2"></p>
        <p className="rounded bg-neutral-200 w-full h-3 mb-2"></p>
        <p className="rounded bg-neutral-200 w-[70%] h-3"></p>
      </div>
    );
  }

  let description = marked.parse(problem.description);

  return (
    <div className="grow bg-white dark:bg-black p-4 h-full max-h-full overflow-y-auto">
      <h1 className="text-4xl font-extrabold mb-4">{problem.title}</h1>


      {(user?.auth == "OFFICER" || user?.auth == "ADMIN") && <div className="mb-4"><Link href={`./${problem.id}/edit`}>
        <a className="rounded-full bg-neutral-100 dark:bg-neutral-800 dark:hover:bg-neutral-700 px-4 py-2 hover:bg-neutral-200 transition-colors">Edit</a>
      </Link></div>}

      <TagsAndBookmark id={problem.id} loggedIn={!!user} />

      <div
        ref={content}
        className="prose prose-neutral dark:prose-invert"
        dangerouslySetInnerHTML={{ __html: description }}
      />

      <Examples id={problem.id} />
    </div>
  );
}

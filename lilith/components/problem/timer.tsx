import { useEffect, useState } from "react";
import { ProblemTimer, useStore } from "../../utils/state";

const idle: ProblemTimer = { elapsed: 0, started: null };

function format(milliseconds: number): string {
    const minutes = Math.floor(milliseconds / 60000);
    const pad = (value: number) => value.toString().padStart(2, "0");
    return `${pad(Math.floor(minutes / 60))}:${pad(minutes % 60)}`;
}

/// An optional per-problem stopwatch. It only reads the clock, so it cannot delay a run or a submission.
export default function Timer({ id }: { id: number }): JSX.Element {
    const timer = useStore((state) => state.timers[id] ?? idle);
    const hidden = useStore((state) => state.timerHidden);
    const setTimer = useStore((state) => state.setTimer);
    const setHidden = useStore((state) => state.setTimerHidden);
    const [now, setNow] = useState(() => Date.now());

    useEffect(() => {
        if (timer.started === null || hidden) return;
        setNow(Date.now());
        const tick = setInterval(() => setNow(Date.now()), 1000);
        return () => clearInterval(tick);
    }, [timer.started, hidden]);

    const button = "px-2 py-1 rounded hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors";

    if (hidden) {
        return <button className={button} onClick={() => setHidden(false)}>Show timer</button>;
    }

    const running = timer.started !== null;
    const elapsed = timer.elapsed + (running ? Math.max(0, now - timer.started!) : 0);

    return (
        <span className="inline-flex items-center gap-1" aria-label="Coding timer">
            <span className="font-mono tabular-nums px-1" title="Hours and minutes spent on this problem">{format(elapsed)}</span>
            {running
                ? <button className={button} onClick={() => setTimer(id, { elapsed, started: null })}>Pause</button>
                : <button className={button} onClick={() => setTimer(id, { elapsed: timer.elapsed, started: Date.now() })}>
                    {timer.elapsed > 0 ? "Resume" : "Start"}
                </button>}
            <button className={button} onClick={() => setTimer(id, idle)}>Reset</button>
            <button className={button} onClick={() => setHidden(true)}>Hide</button>
        </span>
    );
}

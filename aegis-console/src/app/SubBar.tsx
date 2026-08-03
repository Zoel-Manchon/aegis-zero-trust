/* SubBar — the 38px strip under the header. Left: where you are. Right: the
 * state of the event stream and the two controls that change it.
 *
 * Every authenticated screen renders one, so the operator never loses sight of
 * whether telemetry is still arriving — including while they're on Account. */

export interface StreamControls {
    /** "open" while the SSE/WS streams are connected */
    live: boolean;
    reconnecting?: boolean;
    paused: boolean;
    onTogglePause: () => void;
    muted: boolean;
    onToggleMute: () => void;
}

export function SubBar({ label, stream }: { label: string; stream?: StreamControls }) {
    const state = stream
        ? stream.paused
            ? "Paused"
            : stream.reconnecting
              ? "Reconnecting"
              : stream.live
                ? "Live"
                : "Offline"
        : null;
    const hot = state === "Live";

    return (
        <div className="flex h-[38px] items-center justify-between gap-4 border-b-2 border-line px-4">
            <span className="text-[10px] uppercase tracking-[0.2em] text-fg-dim">{label}</span>
            {stream && (
                <div className="flex items-center gap-2.5 text-[10px] uppercase tracking-[0.14em]">
                    <span className={`flex items-center gap-1.5 ${hot ? "text-accent" : "text-fg-dim"}`}>
                        <span
                            className={`h-2 w-2 ${hot ? "bg-accent pulse" : "bg-neutral-500"}`}
                            aria-hidden
                        />
                        {state}
                    </span>
                    <button
                        onClick={stream.onToggleMute}
                        className="btn btn-secondary btn-micro"
                        title={stream.muted ? "Alert sound off" : "Alert sound on"}
                    >
                        {stream.muted ? "Alerts off" : "Alerts on"}
                    </button>
                    <button
                        onClick={stream.onTogglePause}
                        className={`btn btn-micro ${stream.paused ? "btn-primary" : "btn-secondary"}`}
                    >
                        {stream.paused ? "■ Resume stream" : "● Streaming"}
                    </button>
                </div>
            )}
        </div>
    );
}

import type { InputHTMLAttributes } from "react";

interface FieldProps extends InputHTMLAttributes<HTMLInputElement> {
    label: string;
    hint?: string;
}

export function Field({ label, hint, id, ...rest }: FieldProps) {
    const inputId = id ?? `f-${label.toLowerCase().replace(/\s+/g, "-")}`;
    return (
        <label className="block" htmlFor={inputId}>
            <span className="block text-[10px] uppercase tracking-[1.5px] text-fg-dim">{label}</span>
            <input
                id={inputId}
                {...rest}
                className="mt-1 w-full border border-line bg-bg px-2 py-1.5 text-xs text-fg outline-none focus:border-accent"
            />
            {hint && <span className="mt-1 block text-[9px] text-fg-dim">{hint}</span>}
        </label>
    );
}

import type { InputHTMLAttributes } from "react";

interface FieldProps extends InputHTMLAttributes<HTMLInputElement> {
    label: string;
    hint?: string;
}

export function Field({ label, hint, id, className = "", ...rest }: FieldProps) {
    const inputId = id ?? `f-${label.toLowerCase().replace(/\s+/g, "-")}`;
    return (
        <div className="field">
            <label htmlFor={inputId}>{label}</label>
            <input id={inputId} {...rest} className={`input ${className}`} />
            {hint && <span className="mt-1 block text-[11px] text-fg-dim">{hint}</span>}
        </div>
    );
}

import type { ButtonHTMLAttributes } from "react";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: "primary" | "ghost" | "danger";
}

const variants: Record<NonNullable<ButtonProps["variant"]>, string> = {
    primary: "border-accent bg-panel-hi text-accent hover:bg-accent/10",
    ghost: "border-line bg-transparent text-fg-dim hover:text-fg hover:border-fg-dim",
    danger: "border-sev-critical/50 bg-sev-critical/5 text-sev-critical hover:bg-sev-critical/15",
};

export function Button({ variant = "primary", className = "", ...rest }: ButtonProps) {
    return (
        <button
            {...rest}
            className={`border px-2 py-2 text-[11px] uppercase tracking-[1.5px] transition-colors disabled:cursor-wait disabled:opacity-60 ${variants[variant]} ${className}`}
        />
    );
}

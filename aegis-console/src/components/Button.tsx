import type { ButtonHTMLAttributes } from "react";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
    /** primary = filled vermilion · secondary = outlined · ghost = inline text action */
    variant?: "primary" | "secondary" | "ghost";
    block?: boolean;
}

const variants: Record<NonNullable<ButtonProps["variant"]>, string> = {
    primary: "btn-primary",
    secondary: "btn-secondary",
    ghost: "btn-ghost",
};

export function Button({ variant = "primary", block = false, className = "", ...rest }: ButtonProps) {
    return (
        <button
            {...rest}
            className={`btn ${variants[variant]} ${block ? "btn-block" : ""} text-[12px] uppercase tracking-[0.1em] ${className}`}
        />
    );
}

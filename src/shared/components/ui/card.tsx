import type { HTMLAttributes, ReactNode } from "react";

interface CardProps extends HTMLAttributes<HTMLElement> {
  children: ReactNode;
}

export function Card({ children, className = "", ...props }: CardProps) {
  return (
    <article className={`border border-voicebox-border bg-white p-4 ${className}`} {...props}>
      {children}
    </article>
  );
}

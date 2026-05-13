import type { InputHTMLAttributes } from "react";

export function Input(props: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      className="h-11 border-2 border-voicebox-black bg-white px-3 text-sm text-voicebox-black focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2"
      {...props}
    />
  );
}

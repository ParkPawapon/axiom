interface ErrorPanelProps {
  message: string;
}

export function ErrorPanel({ message }: ErrorPanelProps) {
  return (
    <div className="border-2 border-voicebox-red bg-white p-4 text-sm text-voicebox-red">
      {message}
    </div>
  );
}

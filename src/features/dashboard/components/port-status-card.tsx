import { Chip } from "../../../shared/components/ui/chip";

interface PortStatusCardProps {
  ports: number[];
}

export function PortStatusCard({ ports }: PortStatusCardProps) {
  const uniquePorts = Array.from(new Set(ports)).sort((left, right) => left - right);

  return (
    <article className="border-2 border-voicebox-black bg-white p-5">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="font-mono text-xs uppercase text-voicebox-secondary">Bound Ports</p>
          <p className="mt-2 font-display text-5xl uppercase leading-none text-voicebox-black">
            {uniquePorts.length}
          </p>
        </div>
        <Chip tone={uniquePorts.length > 0 ? "success" : "neutral"}>127.0.0.1</Chip>
      </div>
      <p className="mt-5 break-words font-mono text-xs text-voicebox-secondary">
        {uniquePorts.length > 0
          ? uniquePorts.map((port) => `:${port}`).join(" ")
          : "No project PHP process is currently bound."}
      </p>
    </article>
  );
}

export const voiceboxTokens = {
  color: {
    primaryBlack: "#0A0A0A",
    accentRed: "#EF4444",
    background: "#FAFAFA",
    surface: "#F5F5F5",
    surfaceRaised: "#E5E5E5",
    textPrimary: "#0A0A0A",
    textSecondary: "#525252",
    textTertiary: "#A3A3A3",
    borderSubtle: "#E5E5E5",
    borderMedium: "#D4D4D4",
    borderStrong: "#0A0A0A",
    success: "#16A34A",
    warning: "#CA8A04",
    error: "#EF4444",
    info: "#0A0A0A",
  },
  typography: {
    display: '"Archivo Black", Impact, "Arial Black", sans-serif',
    body: '"Work Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
    mono: '"Space Mono", "Courier New", Consolas, monospace',
  },
  radius: {
    default: 0,
  },
  border: {
    control: "2px solid #0A0A0A",
    subtle: "1px solid #E5E5E5",
    medium: "1px solid #D4D4D4",
  },
  shadow: {
    default: "none",
  },
} as const;

export type VoiceboxTokens = typeof voiceboxTokens;

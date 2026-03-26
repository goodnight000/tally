import claudeLogo from "../../assets/claude-logo.svg";
import codexLogo from "../../assets/codex-color.svg";
import clineLogo from "../../assets/cline-logo.png";
import kiloLogo from "../../assets/kilo-logo.png";
import rooLogo from "../../assets/roo-logo.png";
import opencodeLogo from "../../assets/opencode-logo.svg";
import openclawLogo from "../../assets/openclaw-logo.svg";
import { TOOL_COLORS } from "../../lib/constants";

const LOGO_MAP: Record<string, string> = {
  claude: claudeLogo,
  codex: codexLogo,
  cline: clineLogo,
  kilo: kiloLogo,
  roo: rooLogo,
  opencode: opencodeLogo,
  openclaw: openclawLogo,
};

const DISPLAY_NAMES: Record<string, string> = {
  claude: "Claude Code",
  codex: "Codex CLI",
  cline: "Cline",
  kilo: "Kilo Code",
  roo: "Roo Code",
  opencode: "OpenCode",
  openclaw: "OpenClaw",
};

interface Props {
  tool: string;
  size?: number;
}

export function ToolIcon({ tool, size = 18 }: Props) {
  if (tool in LOGO_MAP) {
    return (
      <img
        src={LOGO_MAP[tool]}
        alt={DISPLAY_NAMES[tool] || tool}
        width={size}
        height={size}
        className="shrink-0"
      />
    );
  }

  // Colored initial fallback for tools without SVG logos
  const color = TOOL_COLORS[tool] || "#8D8D83";
  const initial = (DISPLAY_NAMES[tool] || tool)[0].toUpperCase();

  return (
    <div
      className="shrink-0 rounded flex items-center justify-center text-white font-bold"
      style={{
        width: size,
        height: size,
        backgroundColor: color,
        fontSize: size * 0.55,
        lineHeight: 1,
      }}
    >
      {initial}
    </div>
  );
}

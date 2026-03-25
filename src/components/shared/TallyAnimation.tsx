/**
 * Cascading token squares animation — small squares rain down gently
 * and stack into organized rows at the bottom, like a visual abacus
 * counting tokens. Terracotta and periwinkle alternate. Rows fill up,
 * dissolve, and restart.
 */
export function TallyAnimation({ size = 200 }: { size?: number }) {
  // Generate token squares with staggered positions and delays
  const columns = 8;
  const rows = 4;
  const squareSize = 10;
  const gap = 3;
  const gridWidth = columns * (squareSize + gap) - gap;
  const gridLeft = (200 - gridWidth) / 2;
  const stackBase = 160;

  // Falling tokens
  const fallingTokens = Array.from({ length: 16 }, (_, i) => {
    const col = i % columns;
    const row = Math.floor(i / columns);
    const isClaud = (col + row) % 2 === 0;
    const x = gridLeft + col * (squareSize + gap);
    const targetY = stackBase - row * (squareSize + gap);
    const startY = -20 - Math.random() * 40;
    const delay = row * 1.2 + col * 0.15 + Math.random() * 0.3;

    return { x, targetY, startY, isClaud, delay, col, row };
  });

  return (
    <div
      style={{ width: size, height: size }}
      className="flex items-center justify-center"
    >
      <svg
        width={size}
        height={size}
        viewBox="0 0 200 200"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <style>{`
          @keyframes tokenFall {
            0% {
              opacity: 0;
              transform: translateY(var(--start-y));
            }
            8% {
              opacity: 0.8;
            }
            30% {
              opacity: 1;
              transform: translateY(0px);
            }
            70% {
              opacity: 1;
              transform: translateY(0px);
            }
            85% {
              opacity: 0;
              transform: translateY(0px);
            }
            100% {
              opacity: 0;
              transform: translateY(var(--start-y));
            }
          }
          @keyframes shimmer {
            0%, 100% { opacity: 0.03; }
            50% { opacity: 0.08; }
          }
          @keyframes counterPulse {
            0%, 100% { opacity: 0.6; }
            50% { opacity: 1; }
          }
          @keyframes gentleBob {
            0%, 100% { transform: translateY(0); }
            50% { transform: translateY(-3px); }
          }
          .token-square {
            animation: tokenFall 5s ease-out infinite;
            rx: 2;
          }
        `}</style>

        {/* Subtle background grid lines */}
        {Array.from({ length: columns + 1 }, (_, i) => (
          <line
            key={`vg${i}`}
            x1={gridLeft + i * (squareSize + gap) - gap / 2}
            y1={stackBase - rows * (squareSize + gap) + squareSize}
            x2={gridLeft + i * (squareSize + gap) - gap / 2}
            y2={stackBase + squareSize}
            stroke="#E8E8E0"
            strokeWidth="0.5"
            opacity="0.5"
          />
        ))}
        {Array.from({ length: rows + 1 }, (_, i) => (
          <line
            key={`hg${i}`}
            x1={gridLeft - gap / 2}
            y1={stackBase - i * (squareSize + gap) + squareSize}
            x2={gridLeft + gridWidth + gap / 2}
            y2={stackBase - i * (squareSize + gap) + squareSize}
            stroke="#E8E8E0"
            strokeWidth="0.5"
            opacity="0.5"
          />
        ))}

        {/* Background shimmer */}
        <rect
          x={gridLeft - 8}
          y={stackBase - rows * (squareSize + gap)}
          width={gridWidth + 16}
          height={rows * (squareSize + gap) + squareSize + 8}
          rx="6"
          fill="#CC785C"
          style={{ animation: "shimmer 5s ease-in-out infinite" }}
        />

        {/* Falling + stacking token squares */}
        {fallingTokens.map((token, i) => (
          <rect
            key={i}
            x={token.x}
            y={token.targetY}
            width={squareSize}
            height={squareSize}
            fill={token.isClaud ? "#CC785C" : "#7B8CEA"}
            className="token-square"
            style={{
              "--start-y": `${token.startY - token.targetY}px`,
              animationDelay: `${token.delay}s`,
            } as React.CSSProperties}
          />
        ))}

        {/* Floating accent particles above the grid */}
        {[
          { cx: 55, cy: 55, r: 2, color: "#CC785C", delay: "0.5s" },
          { cx: 145, cy: 48, r: 1.8, color: "#7B8CEA", delay: "1.2s" },
          { cx: 80, cy: 40, r: 1.5, color: "#7B8CEA", delay: "2.0s" },
          { cx: 120, cy: 52, r: 2, color: "#CC785C", delay: "0.8s" },
          { cx: 100, cy: 35, r: 1.5, color: "#CC785C", delay: "1.8s" },
          { cx: 65, cy: 45, r: 1.8, color: "#7B8CEA", delay: "2.5s" },
        ].map((dot, i) => (
          <circle
            key={`p${i}`}
            cx={dot.cx}
            cy={dot.cy}
            r={dot.r}
            fill={dot.color}
            opacity="0"
          >
            <animate
              attributeName="opacity"
              values="0;0.5;0"
              dur="3s"
              begin={dot.delay}
              repeatCount="indefinite"
            />
            <animate
              attributeName="cy"
              values={`${dot.cy + 8};${dot.cy - 8};${dot.cy + 8}`}
              dur="3s"
              begin={dot.delay}
              repeatCount="indefinite"
            />
          </circle>
        ))}

        {/* Counter text at top */}
        <text
          x="100"
          y="24"
          textAnchor="middle"
          fill="#1A1A1A"
          fontSize="11"
          fontFamily="var(--font-serif, Georgia)"
          style={{ animation: "counterPulse 2s ease-in-out infinite" }}
        >
          counting tokens...
        </text>
      </svg>
    </div>
  );
}

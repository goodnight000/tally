import { useEffect, useRef, useState } from "react";

interface Props {
  value: number;
  duration?: number;
  formatter?: (n: number) => string;
  className?: string;
}

export function AnimatedNumber({
  value,
  duration = 500,
  formatter = (n) => n.toLocaleString(),
  className,
}: Props) {
  const [display, setDisplay] = useState(0);
  const rafRef = useRef<number>(0);
  const startRef = useRef<number>(0);
  const startValid = useRef(false);
  const fromRef = useRef(0);

  useEffect(() => {
    fromRef.current = display;
    startValid.current = false;

    const animate = (timestamp: number) => {
      if (!startValid.current) {
        startRef.current = timestamp;
        startValid.current = true;
      }
      const elapsed = timestamp - startRef.current;
      const progress = Math.min(elapsed / duration, 1);
      // Ease-out cubic
      const eased = 1 - Math.pow(1 - progress, 3);
      const current = Math.round(fromRef.current + (value - fromRef.current) * eased);
      setDisplay(current);

      if (progress < 1) {
        rafRef.current = requestAnimationFrame(animate);
      }
    };

    rafRef.current = requestAnimationFrame(animate);
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
    };
  }, [value, duration]);

  return <span className={className}>{formatter(display)}</span>;
}

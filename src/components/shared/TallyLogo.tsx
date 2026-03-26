import tallyLogo from "../../assets/tally-logo-flow-horizontal.svg";

interface Props {
  size?: number;
  className?: string;
}

export function TallyLogo({ size = 128, className = "" }: Props) {
  return (
    <img
      src={tallyLogo}
      alt="Tally logo"
      width={size}
      height={size}
      className={className}
    />
  );
}

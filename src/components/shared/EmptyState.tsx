interface Props {
  message: string;
  className?: string;
}

export function EmptyState({ message, className = "" }: Props) {
  return (
    <div
      className={`flex items-center justify-center py-12 text-text-secondary text-sm ${className}`}
    >
      {message}
    </div>
  );
}

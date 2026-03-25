import { ReactNode } from "react";
import { Sidebar } from "./Sidebar";

export function AppLayout({ children }: { children: ReactNode }) {
  return (
    <div className="flex h-screen bg-cream">
      <Sidebar />
      <main className="flex-1 overflow-auto p-(--spacing-content-padding)">
        {children}
      </main>
    </div>
  );
}

import Link from "next/link";
import { cn } from "@/lib/utils";

type BrandLogoProps = {
  href?: string | null;
  className?: string;
  markOnly?: boolean;
  inverted?: boolean;
  wordmark?: string;
};

export function BrandMark({ className }: { className?: string }) {
  return (
    <span
      className={cn(
        "inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-[#14B8A6] text-[11px] font-semibold tracking-wide text-white shadow-sm ring-1 ring-black/10",
        className
      )}
      style={{ fontFamily: "var(--font-serif), Georgia, serif" }}
    >
      ETE
    </span>
  );
}

/** ChatBridge lockup: teal ETE emblem, serif wordmark, API badge. PNG lives in /logo-chatbridge.png. */
export function BrandLogo({
  href = "/",
  className,
  markOnly = false,
  inverted = false,
  wordmark = "ChatBridge",
}: BrandLogoProps) {
  const inner = (
    <span className={cn("inline-flex items-center gap-2", className)}>
      <BrandMark />
      {!markOnly && (
        <span className="relative inline-flex items-start leading-none">
          <span
            className={cn(
              "text-[17px] font-semibold tracking-tight",
              inverted ? "text-white" : "text-[#0F3D3A] dark:text-white"
            )}
            style={{ fontFamily: "var(--font-serif), Georgia, serif" }}
          >
            {wordmark}
          </span>
          <span
            className={cn(
              "ml-1 mt-px rounded px-1 py-px text-[8px] font-semibold uppercase tracking-wider",
              inverted ? "bg-white/15 text-white" : "bg-[#042F2E] text-white"
            )}
          >
            API
          </span>
        </span>
      )}
    </span>
  );

  if (!href) return inner;
  return (
    <Link href={href} className="inline-flex items-center" aria-label={`${wordmark} API`}>
      {inner}
    </Link>
  );
}

export function BrandLogoImage({ className, alt = "ChatBridge" }: { className?: string; alt?: string }) {
  return (
    // eslint-disable-next-line @next/next/no-img-element
    <img src="/logo-chatbridge.png" alt={alt} className={cn("h-10 w-auto object-contain", className)} />
  );
}

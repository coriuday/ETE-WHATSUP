import Link from "next/link";
import { Button } from "@/components/ui/button";

export function CtaBanner({ authenticated }: { authenticated: boolean }) {
  return (
    <section className="px-6 py-16">
      <div className="mx-auto max-w-6xl overflow-hidden rounded-3xl bg-gradient-to-r from-[#14B8A6] to-[#042F2E] px-8 py-14 text-center text-white">
        <h2 className="text-3xl font-semibold tracking-tight">Ready to run inbox and campaigns in one place?</h2>
        <p className="mx-auto mt-3 max-w-lg text-sm text-white/80">
          Start on the mock WhatsApp provider. Switch to Meta when credentials are ready.
        </p>
        <div className="mt-8 flex flex-wrap justify-center gap-3">
          <Button asChild className="rounded-xl bg-white text-[#042F2E] hover:bg-white/90">
            <Link href={authenticated ? "/dashboard" : "/register"}>
              {authenticated ? "Open dashboard" : "Start Free Trial"}
            </Link>
          </Button>
          <Button asChild variant="outline" className="rounded-xl border-white/40 bg-transparent text-white hover:bg-white/10">
            <Link href={authenticated ? "/inbox" : "/login"}>{authenticated ? "Open inbox" : "Log in"}</Link>
          </Button>
        </div>
      </div>
    </section>
  );
}

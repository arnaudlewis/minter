import type { NfrInfo } from "@/types"
import { CheckCircle2, XCircle } from "lucide-react"

export function NfrStatusIcon({ nfr, size = "size-4" }: { nfr: NfrInfo; size?: string }) {
  const isInvalid =
    typeof nfr.validation_status === "object" && "Invalid" in nfr.validation_status
  if (isInvalid) {
    return <XCircle className={`${size} shrink-0 text-red-400`} />
  }
  return <CheckCircle2 className={`${size} shrink-0 text-emerald-400`} />
}

import { redirect } from "next/navigation";

/**
 * Redirects template detail routes to the live marketplace catalog until a dedicated
 * deep-linked detail experience is implemented.
 */
export default function MarketplaceTemplatePage() {
  redirect("/marketplace");
}

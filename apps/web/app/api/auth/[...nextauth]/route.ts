/**
 * Handles auth requests proxied through the Next.js application.
 *
 * Replace this placeholder with the real auth adapter once the identity provider is selected.
 */
export async function GET() {
  return Response.json({ message: "Auth route scaffold." }, { status: 501 });
}

/**
 * Handles auth POST requests proxied through the Next.js application.
 *
 * Keep request validation and provider-specific behavior delegated to a shared auth module.
 */
export async function POST() {
  return Response.json({ message: "Auth route scaffold." }, { status: 501 });
}

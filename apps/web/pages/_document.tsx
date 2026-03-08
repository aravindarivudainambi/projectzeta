import { Head, Html, Main, NextScript } from "next/document";

/**
 * Provides a minimal Document fallback for production builds that still expect
 * the legacy pages-router document entrypoint.
 */
export default function Document() {
  return (
    <Html lang="en">
      <Head />
      <body>
        <Main />
        <NextScript />
      </body>
    </Html>
  );
}

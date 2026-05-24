import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["__tests__/**/*.test.ts"],
    typecheck: {
      enabled: false,
    },
    // Set TNDM_INTEGRATION_TEST=1 via shell to enable integration tests:
    //   TNDM_INTEGRATION_TEST=1 pnpm exec vitest run __tests__/integration.test.ts
    env: {
      ...(process.env.TNDM_INTEGRATION_TEST && {
        TNDM_INTEGRATION_TEST: process.env.TNDM_INTEGRATION_TEST,
      }),
    },
  },
});

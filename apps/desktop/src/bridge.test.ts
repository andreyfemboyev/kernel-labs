import { expect, it, vi } from "vitest";
import { inspect, workspaces } from "./bridge";
it("sends ABI arguments as structured data and returns the backend report", async () => {
  const result = { state: "potentially_incompatible" };
  const call = vi.fn().mockResolvedValue(result);
  await expect(inspect(workspaces[6], " a/Module.symvers ", "b/Module.symvers", call)).resolves.toEqual(result);
  expect(call).toHaveBeenCalledWith("inspect", { request: { kind: "abi", path: "a/Module.symvers", secondary: "b/Module.symvers" } });
});

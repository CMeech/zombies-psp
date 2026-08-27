import { existsSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { $ } from "bun";
import { MAP_TOOL_ARCHIVE_SHA256, MAP_TOOL_REVISION } from "./slice-map-lib";

const root = resolve(import.meta.dir, "..");
const source = resolve(root, "local/tools/ericw-tools", MAP_TOOL_REVISION);
const build = resolve(source, "build");
// The immutable GitHub source archive was independently hashed when this pin
// was accepted. Keep the checksum visible even though recursive git checkout is
// used here because the build also needs the repository's pinned submodules.
console.log(`ericw-tools ${MAP_TOOL_REVISION} archive sha256 ${MAP_TOOL_ARCHIVE_SHA256}`);
if (!existsSync(resolve(source, ".git"))) {
  mkdirSync(resolve(source, ".."), { recursive: true });
  await $`git clone --filter=blob:none https://github.com/ericwa/ericw-tools.git ${source}`;
  await $`git -C ${source} checkout --detach ${MAP_TOOL_REVISION}`;
  await $`git -C ${source} submodule update --init --recursive`;
}
const actual = (await $`git -C ${source} rev-parse HEAD`.text()).trim();
if (actual !== MAP_TOOL_REVISION) throw new Error(`map compiler revision mismatch: ${actual}`);
mkdirSync(build, { recursive: true });
await $`cmake -S ${source} -B ${build} -DCMAKE_BUILD_TYPE=Release -DDISABLE_TESTS=ON -DDISABLE_DOCS=ON -DENABLE_LIGHTPREVIEW=OFF -DCMAKE_PREFIX_PATH=${"/opt/homebrew/opt/embree;/opt/homebrew/opt/tbb"}`;
await $`cmake --build ${build} --target qbsp vis light --parallel`;
console.log(`map tools ready at ${build}`);

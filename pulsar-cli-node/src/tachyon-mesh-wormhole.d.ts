declare module "@tachyon-mesh/wormhole" {
  import type { WormholeFactory } from "./wormhole.js";

  export const Wormhole: {
    create: WormholeFactory;
  };
}
